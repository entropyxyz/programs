//! This example shows how to write a contrieved and basic constraint: checking the length of the data to be signed.

#![no_std]

extern crate alloc;

use alloc::{string::ToString, vec};

use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct BarebonesProgram;

impl Program for BarebonesProgram {
    /// This is the only function required by the program runtime. `signature_request` is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(signature_request: InitialState) -> Result<(), Error> {
        let data: vec::Vec<u8> = signature_request.preimage;

        // our constraint just checks that the length of the signature request is greater than 10
        if data.len() < 10 {
            return Err(Error::Evaluation(
                "Length of data is too short.".to_string(),
            ));
        }

        Ok(())
    }
}

export_program!(BarebonesProgram);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = InitialState {
            data: "some_data_longer_than_10_bytes".to_string().into_bytes(),
        };

        assert!(BarebonesProgram::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_should_error() {
        // data being checked is under 10 bytes in length
        let signature_request = InitialState {
            data: "under10".to_string().into_bytes(),
        };

        assert!(BarebonesProgram::evaluate(signature_request).is_err());
    }
}
