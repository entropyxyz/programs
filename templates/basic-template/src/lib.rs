//! No-op program

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {
}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {
}

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
