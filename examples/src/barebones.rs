//! This example shows how to write a contrieved and basic constraint: checking the length of the data to be signed.

#![no_main]
#![no_std]

extern crate alloc;

use alloc::{vec, string::ToString};

use ec_constraints::{
    core::{prelude::*, bindgen::*, bindgen::Error as CoreError, export_constraint,
    }
};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct Program;

impl Constraint for Program {
/// This is the function that the constraints engine will runtime execute. signature_request is the preimage of the curve element to be 
/// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(signature_request: EvaluationState) -> Result<(), CoreError> {
        let data: vec::Vec<u8> = signature_request.data;

        // our constraint just checks that the length of the signature request is greater than 10
        if data.len() < 10 {
            return Err(CoreError::Evaluation("Length of data is too short.".to_string()));
        }

        Ok(())
    }
}

export_constraint!(Program);

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