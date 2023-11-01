//! This example demonstrates a contrieved program that can include extra data. Note, only the data in `preimage` will be signed by Entropy; `extra` is used to provide additional data (eg an additional signature or a zkp related to the preimage) that the user requires during program evaluation.

#![no_std]

extern crate alloc;

use alloc::string::ToString;

use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct BarebonesWithExtra;

impl Program for BarebonesWithExtra {
    /// This is the only function required by the program runtime. `signature_request` is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(signature_request: InitialState) -> Result<(), Error> {
        let InitialState { preimage, extra} = signature_request;
        
        // our constraint just checks that the length of the signature request is greater than 10
        if preimage.len() < 10 {
            return Err(Error::Evaluation(
                "Length of data is too short.".to_string(),
            ));
        }

        // Just check and make sure the extra field is not empty.
        extra.ok_or(Error::Evaluation("This program requires that `extra` be `Some`.".to_string()))?;

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
        let signature_request = InitialState {
            preimage: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            extra: Some(vec![0x00]) 
        };

        assert!(BarebonesWithExtra::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_preimage_length_is_invalid() {
        let signature_request = InitialState {
            // should error since preimage is less than 10 bytes
            preimage: "under10".to_string().into_bytes(),
            extra: Some(vec![0x00]) 
        };

        assert!(BarebonesWithExtra::evaluate(signature_request).is_err());
    }

    #[test]
    fn test_error_when_extra_field_is_none() {
        let signature_request = InitialState {
            preimage: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            // should error since extra field is None
            extra: None
        };

        assert!(BarebonesWithExtra::evaluate(signature_request).is_err());
    }

}
