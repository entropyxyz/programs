//! This example shows how to write a contrieved and basic program: checking the length of the data to be signed.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};
use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};
use serde::{Deserialize, Serialize};

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {}

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct BarebonesProgram;

impl Program for BarebonesProgram {
    /// This is the only function required by the program runtime. `message` is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<Vec<u8>>>,
    ) -> Result<(), Error> {
        let message: Vec<u8> = signature_request.message;

        // our program just checks that the length of the message is greater than 10
        if message.len() < 10 {
            return Err(Error::Evaluation(
                "Length of message is too short.".to_string(),
            ));
        }

        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(BarebonesProgram);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = SignatureRequest {
            message: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(BarebonesProgram::evaluate(signature_request, None, None).is_ok());
    }

    #[test]
    fn test_should_error() {
        // data being checked is under 10 bytes in length
        let signature_request = SignatureRequest {
            message: "under10".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(BarebonesProgram::evaluate(signature_request, None, None).is_err());
    }
}
