//! This example demonstrates a contrieved program that can include extra data. Note, only the data in `preimage` will be signed by Entropy; `extra` is used to provide additional data (eg an additional signature or a zkp related to the preimage) that the user requires during program evaluation.

#![no_std]

extern crate alloc;

use alloc::string::ToString;

use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct BarebonesWithExtra;

impl Program for BarebonesWithExtra {
    /// This is the only function required by the program runtime. `signature_request` includes the message to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(signature_request: SignatureRequest) -> Result<(), Error> {
        let SignatureRequest { message, auxilary_data } = signature_request;

        // our constraint just checks that the length of the signature request is greater than 10
        if message.len() < 10 {
            return Err(Error::Evaluation(
                "Length of message is too short.".to_string(),
            ));
        }

        // Just check and make sure the extra field is not empty.
        auxilary_data.ok_or(Error::Evaluation(
            "This program requires that `auxilary_data` be `Some`.".to_string(),
        ))?;

        Ok(())
    }
}

export_program!(BarebonesWithExtra);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_preimage_length_is_valid() {
        let signature_request = SignatureRequest {
            message: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            auxilary_data: Some(vec![0x00]),
        };

        assert!(BarebonesWithExtra::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_preimage_length_is_invalid() {
        let signature_request = SignatureRequest {
            // should error since preimage is less than 10 bytes
            message: "under10".to_string().into_bytes(),
            auxilary_data: Some(vec![0x00]),
        };

        assert!(BarebonesWithExtra::evaluate(signature_request).is_err());
    }

    #[test]
    fn test_error_when_extra_field_is_none() {
        let signature_request = SignatureRequest {
            message: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            // should error since auxilary_data field is None
            auxilary_data: None,
        };

        assert!(BarebonesWithExtra::evaluate(signature_request).is_err());
    }
}
