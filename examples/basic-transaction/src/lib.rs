#![no_main]
#![no_std]

extern crate alloc;

use ec_constraints::{
    constraints::acl::*,
    core::{bindgen::*, export_program, prelude::*, SatisfiableForArchitecture, TryParse},
};

use alloc::vec;

pub struct BasicTransaction;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Program for BasicTransaction {
    /// This is the function that the constraints engine will runtime esecute. signature_request is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    // #[no_mangle]
    fn evaluate(state: InitialState) -> Result<(), CoreError> {
        // parse the raw tx into some type
        let parsed_tx =
            <Evm as Architecture>::TransactionRequest::try_parse(state.data.as_slice())?;

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
}

export_program!(BasicTransaction);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let signature_request = InitialState {
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };

        match BasicTransaction::evaluate(signature_request) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e)
            }
        }
    }

    #[test]
    fn test_start_fail() {
        let signature_request = InitialState {
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };

        match BasicTransaction::evaluate(signature_request) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}
