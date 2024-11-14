//! This example shows how to write a contrieved and basic program: checking the length of the data to be signed.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};
use codec::Decode;
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

pub struct OracleExample;

impl Program for OracleExample {
    fn evaluate(
        _signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        oracle_data: Option<Vec<Vec<u8>>>,
    ) -> Result<(), Error> {
        let data = oracle_data.ok_or(Error::Evaluation("No oracle data provided.".to_string()))?;
        let block_number = u32::decode(&mut data[0].as_ref())
            .map_err(|_| Error::Evaluation("Unable to decode oracle data".to_string()))?;
        // our program just checks that the block number is greater than 100
        if block_number > 100 {
            return Err(Error::Evaluation("Block Number too large".to_string()));
        }

        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(OracleExample);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use codec::Encode;

    #[test]
    fn test_should_sign() {
        let signature_request = SignatureRequest {
            message: "".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(
            OracleExample::evaluate(signature_request, None, Some(vec![99u32.encode()])).is_ok()
        );
    }

    #[test]
    fn test_should_error() {
        // data being checked is under 10 bytes in length
        let signature_request = SignatureRequest {
            message: "".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert_eq!(
            OracleExample::evaluate(signature_request, None, Some(vec![101u32.encode()]))
                .unwrap_err()
                .to_string(),
            "Error::Evaluation(\"Block Number too large\")"
        );
    }
}
