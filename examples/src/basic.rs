#![no_main]
// #![no_std]

// wit_bindgen::generate!("constraint");
extern crate alloc;

// use wit_bindgen::rt::{vec, string::ToString};

use ec_constraints::{
    constraints::acl::*,
    core::{SatisfiableForArchitecture, TryParse,
        prelude::*, bindgen::*, export_constraint
    }
};

// Generates the bindings to EvaluationState
use alloc::{vec};

pub struct Program;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Constraint for Program {
    /// This is the function that the constraints engine will runtime esecute. signature_request is the preimage of the curve element to be 
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    // #[no_mangle]
    fn evaluate(state: EvaluationState) -> Result<(), CoreError> {
        // parse the raw tx into some type
        let parsed_tx = <Evm as Architecture>::TransactionRequest::try_parse(state.data.as_slice())?;

        // construct a whitelist ACL
        // TODO can we just use Address instead of AddressRaw?
        let whitelisted_address: <Evm as Architecture>::AddressRaw = hex::decode("772b9a9e8aa1c9db861c6611a82d251db4fac990").unwrap().try_into().unwrap();
        let allowlisted_acl = Acl::<<Evm as Architecture>::AddressRaw> {
            addresses: vec![whitelisted_address],
            ..Default::default()
        };

        // check that the parsed tx is allowed by the ACL
        allowlisted_acl.is_satisfied_by(&parsed_tx)?;

        Ok(())
    }
}

export_constraint!(Program);


// fn main() {
//     // This is an RLP encoded Ethereum transaction request
//     // Its recipient address is : "772b9a9e8aa1c9db861c6611a82d251db4fac990"
//     let raw_unsigned_tx = "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string();

//     match evaluate(raw_unsigned_tx.as_bytes()) {
//         Ok(_) => return,
//         Err(e) => panic!("{}", e),
//     }
// }

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let signature_request = EvaluationState {
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };
        
        match Program::evaluate(signature_request) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e)
            },
        }
    }

    #[test]
    fn test_start_fail() {
        let signature_request = EvaluationState {
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };

        match Program::evaluate(signature_request) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e)
            },
        }
    }
}