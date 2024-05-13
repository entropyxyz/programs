//! This is an allow list where the allowed addresses are hashed to improve privacy
//! It is still possible for anyone to check whether a given address is in the list, using the
//! on-chain bytecode. But you cannot just read the allowed addresses from it.
#![no_std]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use blake2::{Blake2s256, Digest};
use entropy_programs::{
    arch::evm::NameOrAddress,
    core::{bindgen::*, export_program, prelude::*, TryParse},
    programs::acl::*,
};
use serde::{Deserialize, Serialize};

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {}

pub struct PrivateTransactionAcl;

include!(concat!(env!("OUT_DIR"), "/addresses.rs"));

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Program for PrivateTransactionAcl {
    /// Allow any address given in the pre-defined list (addresses.txt)
    // #[no_mangle]
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<u8>>,
    ) -> Result<(), CoreError> {
        // parse the raw tx into some type
        let parsed_tx = <Evm as Architecture>::TransactionRequest::try_parse(
            signature_request.message.as_slice(),
        )?;

        let name_or_address: NameOrAddress = parsed_tx.to.ok_or(Error::Evaluation(
            "No recipient given in transaction".to_string(),
        ))?;

        match name_or_address {
            NameOrAddress::Name(_) => Err(Error::Evaluation("ENS names not supported".to_string())),
            NameOrAddress::Address(address) => {
                let hashed_address = {
                    let mut hasher = Blake2s256::new();
                    hasher.update(&address.0);
                    hasher.finalize().into()
                };
                if ADDRESSES.contains(&hashed_address) {
                    Ok(())
                } else {
                    Err(Error::Evaluation("Address not in allow list".to_string()))
                }
            }
        }
    }

    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(PrivateTransactionAcl);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_evaluate() {
        let signature_request = SignatureRequest {
            // `data` is an RLP serialized ETH transaction with the recipient set to `0x772b9a9e8aa1c9db861c6611a82d251db4fac990`
            message: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(PrivateTransactionAcl::evaluate(signature_request, None, None).is_ok());
    }

    #[test]
    fn test_start_fail() {
        let signature_request = SignatureRequest {
            // `data` is the same as previous test, but recipient address ends in `1` instead of `0`, so it should fail
            message: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac991019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(PrivateTransactionAcl::evaluate(signature_request, None, None).is_err());
    }
}
