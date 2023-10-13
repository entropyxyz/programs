//! This example shows having an allow list defining who is able to submit signature requests
#![no_std]

extern crate alloc;

use alloc::string::ToString;

use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct PermissionedProgram;

/// The list of substrate account IDs of users who may use this program
const ALLOWED_AUTHORS: [[u8; 32]; 2] = [[1; 32], [2; 32]];

impl Program for PermissionedProgram {
    fn evaluate(signature_request: InitialState) -> Result<(), Error> {
        let signature_request_key: [u8; 32] = signature_request
            .signature_request_key
            .try_into()
            .map_err(|_| Error::Evaluation("Singature request key must be 32 bytes".to_string()))?;
        if ALLOWED_AUTHORS.contains(&signature_request_key) {
            Ok(())
        } else {
            Err(Error::Evaluation("Author not in allow list".to_string()))
        }
    }
}

export_program!(PermissionedProgram);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = InitialState {
            data: b"some_data".to_vec(),
            signature_request_key: [1; 32].to_vec(),
        };

        assert!(PermissionedProgram::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_should_error() {
        let signature_request = InitialState {
            data: b"some_data".to_vec(),
            signature_request_key: [0; 32].to_vec(),
        };

        assert!(PermissionedProgram::evaluate(signature_request).is_err());
    }
}
