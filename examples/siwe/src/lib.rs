//! Allows only valid Sign-in with Ethereum messages (EIP 4361)
#![no_std]

extern crate alloc;

use alloc::{string::String, string::ToString, vec};

use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};
use siwe::Message;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct Siwe;

impl Program for Siwe {
    fn evaluate(signature_request: InitialState) -> Result<(), Error> {
        let data: vec::Vec<u8> = signature_request.data;
        let string_message =
            String::from_utf8(data).map_err(|err| Error::Evaluation(err.to_string()))?;
        if string_message.parse::<Message>().is_err() {
            return Err(Error::Evaluation(
                "Not a valid Sign-in with Ethereum message".to_string(),
            ));
        };

        Ok(())
    }
}

export_program!(Siwe);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = InitialState {
            data: "localhost wants you to sign in with your Ethereum account:
0x6Ee9894c677EFa1c56392e5E7533DE76004C8D94

This is a test statement.

URI: https://localhost/login
Version: 1
Chain ID: 1
Nonce: oNCEHm5jzQU2WvuBB
Issued At: 2022-01-28T23:28:16.013Z"
                .to_string()
                .into_bytes(),
        };

        assert!(Siwe::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_should_not_sign() {
        let signature_request = InitialState {
            data: "localhost does not want you to sign in with your Ethereum account:
0x6Ee9894c677EFa1c56392e5E7533DE76004C8D94

This is a test statement.

URI: https://localhost/login
Version: 1
Chain ID: 1
Nonce: oNCEHm5jzQU2WvuBB
Issued At: 2022-01-28T23:28:16.013Z"
                .to_string()
                .into_bytes(),
        };

        assert!(Siwe::evaluate(signature_request).is_err());
    }
}
