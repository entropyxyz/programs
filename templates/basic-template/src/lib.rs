//! No-op program

#![no_std]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct {{project-name | upper_camel_case}};

impl Program for {{project-name | upper_camel_case}} {
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
    ) -> Result<(), Error> {
        if signature_request.message.is_empty() {
            return Err(Error::Evaluation(
                "Message must have a length greater than zero".to_string(),
            ));
        }
        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!({{project-name | upper_camel_case}});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = SignatureRequest {
            message: b"some_message".to_vec(),
            auxilary_data: None,
        };

        assert!({{project-name | upper_camel_case}}::evaluate(signature_request, None).is_ok());
    }

    #[test]
    fn test_should_fail() {
        let signature_request = SignatureRequest {
            message: Vec::new(),
            auxilary_data: None,
        };

        assert!({{project-name | upper_camel_case}}::evaluate(signature_request, None).is_err());
    }
}
