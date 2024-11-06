//! This example demonstrates a contrieved program that can include auxilary data. Note, only the data in `message` will be signed by Entropy; `auxilary_data` is used to provide additional data (eg an additional signature or a zkp related to the preimage) that the user requires during program evaluation.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

use serde::{Deserialize, Serialize};

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {}

pub struct BarebonesWithAuxilary;

impl Program for BarebonesWithAuxilary {
    /// This is the only function required by the program runtime. `signature_request` includes the message to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<Vec<u8>>>,
    ) -> Result<(), Error> {
        let SignatureRequest {
            message,
            auxilary_data,
        } = signature_request;

        // our program just checks that the length of the signature request is greater than 10
        if message.len() < 10 {
            return Err(Error::Evaluation(
                "Length of message is too short.".to_string(),
            ));
        }

        // Just check and make sure the `auxilary_data` field is not empty.
        auxilary_data.ok_or(Error::Evaluation(
            "This program requires that `auxilary_data` be `Some`.".to_string(),
        ))?;

        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(BarebonesWithAuxilary);

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_message_length_is_valid() {
        let signature_request = SignatureRequest {
            message: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            auxilary_data: Some(vec![0x00]),
        };

        assert!(BarebonesWithAuxilary::evaluate(signature_request, None, None).is_ok());
    }

    /// Note, the program is written s.t. if `message` is less than 10 bytes, the program will error.
    #[test]
    fn test_message_length_is_invalid() {
        let signature_request = SignatureRequest {
            // should error since preimage is less than 10 bytes
            message: "under10".to_string().into_bytes(),
            auxilary_data: Some(vec![0x00]),
        };

        assert!(BarebonesWithAuxilary::evaluate(signature_request, None, None).is_err());
    }

    /// Note, the program is written s.t. if `auxilary_data` is `None`, the program will error.
    #[test]
    fn test_error_when_auxilary_field_is_none() {
        let signature_request = SignatureRequest {
            message: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            // should error since auxilary_data field is None
            auxilary_data: None,
        };

        assert!(BarebonesWithAuxilary::evaluate(signature_request, None, None).is_err());
    }
}
