#![no_std]

extern crate alloc;

use entropy_programs::{
    core::{bindgen::*, export_program, prelude::*, SatisfiableForArchitecture, TryParse},
    programs::acl::*,
};

use alloc::{vec, vec::Vec};

pub struct BasicTransaction;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Program for BasicTransaction {
    /// This is the function that the programs engine will runtime esecute. signature_request is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    // #[no_mangle]
    fn evaluate(state: SignatureRequest) -> Result<(), CoreError> {
        // parse the raw tx into some type
        let parsed_tx =
            <Evm as Architecture>::TransactionRequest::try_parse(state.message.as_slice())?;

        // construct a whitelist ACL
        // TODO can we just use Address instead of AddressRaw?
        let whitelisted_address: <Evm as Architecture>::AddressRaw =
            hex::decode("772b9a9e8aa1c9db861c6611a82d251db4fac990")
                .unwrap()
                .try_into()
                .unwrap();
        let allowlisted_acl = Acl::<<Evm as Architecture>::AddressRaw> {
            addresses: vec![whitelisted_address],
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
    use alloc::string::ToString;

    #[test]
    fn test_evaluate() {
        let signature_request = SignatureRequest {
            // `data` is an RLP serialized ETH transaction with the recipient set to `0x772b9a9e8aa1c9db861c6611a82d251db4fac990`
            message: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
            auxilary_data: None
        };

        assert!(BasicTransaction::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_start_fail() {
        let signature_request = SignatureRequest {
            // `data` is the same as previous test, but recipient address ends in `1` instead of `0`, so it should fail
            message: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac991019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
            auxilary_data: None
        };

        assert!(BasicTransaction::evaluate(signature_request).is_err());
    }
}
