//! A template program which allows only valid Sign-in with Ethereum messages (EIP 4361)
//! which sign-in to a given service.
//!
//! This could be used to share an account for a specific service whilst not allowing transactions
//! to be signed.
#![no_std]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};
use serde::{Deserialize, Serialize};
use siwe::Message;

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

// The domain we allow a user to sign-in to. Change this to the desired service
const ALLOWED_DOMAIN: &str = "localhost";

pub struct Siwe;

impl Program for Siwe {
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<Vec<u8>>>,
    ) -> Result<(), Error> {
        let string_message = String::from_utf8(signature_request.message)
            .map_err(|err| Error::Evaluation(err.to_string()))?;
        let siwe_message = string_message
            .parse::<Message>()
            .map_err(|err| Error::Evaluation(err.to_string()))?;

        if siwe_message.domain == ALLOWED_DOMAIN {
            Ok(())
        } else {
            Err(Error::Evaluation(
                "You may not sign-in to this domain".to_string(),
            ))
        }
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(Siwe);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = SignatureRequest {
            message: "localhost wants you to sign in with your Ethereum account:
0x6Ee9894c677EFa1c56392e5E7533DE76004C8D94

This is a test statement.

URI: https://localhost/login
Version: 1
Chain ID: 1
Nonce: oNCEHm5jzQU2WvuBB
Issued At: 2022-01-28T23:28:16.013Z"
                .to_string()
                .into_bytes(),
            auxilary_data: None,
        };

        assert!(Siwe::evaluate(signature_request, None, None).is_ok());
    }

    #[test]
    fn test_bad_siwe_message() {
        let signature_request = SignatureRequest {
            message: "localhost does not want you to sign in with your Ethereum account:
0x6Ee9894c677EFa1c56392e5E7533DE76004C8D94

This is a test statement.

URI: https://localhost/login
Version: 1
Chain ID: 1
Nonce: oNCEHm5jzQU2WvuBB
Issued At: 2022-01-28T23:28:16.013Z"
                .to_string()
                .into_bytes(),
            auxilary_data: None,
        };

        assert!(Siwe::evaluate(signature_request, None, None).is_err());
    }

    #[test]
    fn test_bad_domain() {
        let signature_request = SignatureRequest {
            message: "google.com does not want you to sign in with your Ethereum account:
0x6Ee9894c677EFa1c56392e5E7533DE76004C8D94

This is a test statement.

URI: https://google.com/login
Version: 1
Chain ID: 1
Nonce: oNCEHm5jzQU2WvuBB
Issued At: 2022-01-28T23:28:16.013Z"
                .to_string()
                .into_bytes(),
            auxilary_data: None,
        };

        assert!(Siwe::evaluate(signature_request, None, None).is_err());
    }
}
