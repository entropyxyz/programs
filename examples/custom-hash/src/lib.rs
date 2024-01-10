//! This example shows how to use a non-standardized or custom hash function in a program.

#![no_std]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

use blake3;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct CustomHashExample;

impl Program for CustomHashExample {
    fn evaluate(signature_request: SignatureRequest) -> Result<(), Error> {
        if signature_request.message.len() < 1 {
            return Err(Error::Evaluation(
                "You need to give me SOME data to sign!".to_string(),
            ));
        }
        // By immediately returning Ok, we sign any data that is passed to us.
        Ok(())
    }

    fn custom_hash(data: Vec<u8>) -> Option<Vec<u8>> {
        // We can use any hash function we want here, as long as it returns a 32 byte Vec.
        Some(blake3::hash(&data).as_bytes().to_vec())
    }
}

export_program!(CustomHashExample);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// We are just going to test that the custom hash function works WITHOUT calling evaluate
    fn test_custom_hash() {
        let message = "some_data_to_be_hashed".to_string().into_bytes();

        let expected_hash = blake3::hash(&message).as_bytes().to_vec();
        let actual_hash = CustomHashExample::custom_hash(message).unwrap();

        assert_eq!(actual_hash, expected_hash);
        assert!(actual_hash.len() == 32);
    }
}
