#![no_main]
#![no_std]

extern crate alloc;

use ec_constraints::{
    constraints::acl::*,
    core::{bindgen::*, export_program, prelude::*, SatisfiableForArchitecture, TryParse},
};

use alloc::{string::ToString, vec};

pub struct NotATransaction;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Program for NotATransaction {
    /// Passes only if the given message is not a valid EVM transaction
    // #[no_mangle]
    fn evaluate(state: InitialState) -> Result<(), CoreError> {
        // parse the raw tx into some type
        match <Evm as Architecture>::TransactionRequest::try_parse(state.data.as_slice()) {
            Ok(_) => Err(Error::Evaluation("Parses as a transaction".to_string())),
            Err(_) => Ok(()),
        }
    }
}

export_program!(NotATransaction);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let signature_request = InitialState {
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };

        match NotATransaction::evaluate(signature_request) {
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

        match NotATransaction::evaluate(signature_request) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}
