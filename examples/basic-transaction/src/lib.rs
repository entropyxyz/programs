#![no_std]

extern crate alloc;

use entropy_programs::{
    core::{bindgen::*, export_program, prelude::*, SatisfiableForArchitecture, TryParse},
    programs::acl::*,
};

use alloc::{vec::Vec, string::{String, ToString}, format};

use serde_json;
use serde::{Serialize, Deserialize};

pub struct BasicTransaction;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BasicTransactionConfig {
    pub allowlisted_addresses: Vec<String>,
}

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Program for BasicTransaction {
    /// This is the function that the programs engine will runtime esecute. signature_request is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    // #[no_mangle]
    fn evaluate(signature_request: SignatureRequest, config: Option<Vec<u8>>) -> Result<(), CoreError> {
        // parse the raw tx into some type supported by the Acl check
        let parsed_tx =
            <Evm as Architecture>::TransactionRequest::try_parse(signature_request.message.as_slice())?;

        // construct a allowlist ACL from the config
        let typed_config = serde_json::from_slice::<BasicTransactionConfig>(
                config.ok_or(CoreError::Evaluation("No config provided.".to_string()))?.as_slice()
            ).map_err(|e| CoreError::Evaluation(format!("Failed to parse config: {}", e)))?;

        let addresses: Vec<<Evm as Architecture>::AddressRaw> =
                typed_config
                .allowlisted_addresses
                .iter()
                .map(|a| hex::decode(a).unwrap().try_into().unwrap())
                .collect();

        let allowlisted_acl = Acl::<<Evm as Architecture>::AddressRaw> {
            addresses,
            ..Default::default()
        };

        // check that the parsed tx is allowed by the ACL
        allowlisted_acl.is_satisfied_by(&parsed_tx)?;

        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(BasicTransaction);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    const EVM_TX_WITH_ALLOWLISTED_RECIPIENT: &[u8] = b"0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080";
    const EVM_TX_WITH_NONALLOWLISTED_RECIPIENT: &[u8] = b"0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac991019243726561746564204f6e20456e74726f7079018080";
    const CONFIG: &[u8] = r#"
        {
            "allowlisted_addresses": [
                "772b9a9e8aa1c9db861c6611a82d251db4fac990"
            ]
        }
    "#.as_bytes();

    #[test]
    fn test_evaluate() {
        let signature_request = SignatureRequest {
            // `data` is an RLP serialized ETH transaction with the recipient set to `0x772b9a9e8aa1c9db861c6611a82d251db4fac990`
            message: EVM_TX_WITH_ALLOWLISTED_RECIPIENT.to_vec(),
            auxilary_data: None
        };

        assert!(BasicTransaction::evaluate(signature_request, Some(CONFIG.to_vec())).is_ok());
    }

    #[test]
    fn test_start_fail() {
        let signature_request = SignatureRequest {
            // `data` is the same as previous test, but recipient address ends in `1` instead of `0`, so it should fail
            message: EVM_TX_WITH_NONALLOWLISTED_RECIPIENT.to_vec(),
            auxilary_data: None
        };

        assert!(BasicTransaction::evaluate(signature_request, Some(CONFIG.to_vec())).is_err());
    }
}
