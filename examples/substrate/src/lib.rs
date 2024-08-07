#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use entropy_programs::core::{bindgen::*, export_program, prelude::*};
use entropy_programs_substrate::{check_message_against_transaction, HasFieldsAux};

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {
    pub spec_version: u32,
    pub transaction_version: u32,
    pub values: String,
    pub pallet: String,
    pub function: String,
    pub genesis_hash: String,
}

impl HasFieldsAux for AuxData {
    fn genesis_hash(&self) -> &String {
        &self.genesis_hash
    }
    fn spec_version(&self) -> &u32 {
        &self.spec_version
    }

    fn transaction_version(&self) -> &u32 {
        &self.transaction_version
    }
    fn values(&self) -> &String {
        &self.values
    }
    fn pallet(&self) -> &String {
        &self.pallet
    }
    fn function(&self) -> &String {
        &self.function
    }
}

pub struct SubstrateProgram;

impl Program for SubstrateProgram {
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<u8>>,
    ) -> Result<(), Error> {
        let (_aux_data, _api) = check_message_against_transaction::<AuxData>(signature_request)
            .map_err(|e| {
                Error::InvalidSignatureRequest(format!(
                    "Error comparing tx request and message: {}",
                    e
                ))
            })?;

        // can now use aux data and user config to apply constraints
        // Ex: balance limit, genesis hash (locks it to a chain or specific chains)
        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(SubstrateProgram);
