#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use entropy_programs::core::{bindgen::*, export_program, prelude::*};
use entropy_programs_substrate::{
    check_message_against_transaction, HasFieldsAux, HasFieldsConfig,
};

use serde::{Deserialize, Serialize};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    genesis_hash: String,
}

impl HasFieldsConfig for UserConfig {
    type GenesisHashType = String;

    fn genesis_hash(&self) -> &String {
        &self.genesis_hash
    }
}
/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {
    pub spec_version: u32,
    pub transaction_version: u32,
    pub string_account_id: String,
    pub amount: u128,
}

impl HasFieldsAux for AuxData {
    type SpecVersionType = u32;
    type TransactionVersionType = u32;
    type StringAccountIdType = String;
    type AmountType = u128;

    fn spec_version(&self) -> &u32 {
        &self.spec_version
    }

    fn transaction_version(&self) -> &u32 {
        &self.transaction_version
    }
    fn string_account_id(&self) -> &String {
        &self.string_account_id
    }
    fn amount(&self) -> &u128 {
        &self.amount
    }
}

pub struct SubstrateProgram;

impl Program for SubstrateProgram {
    fn evaluate(
        signature_request: SignatureRequest,
        config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<u8>>,
    ) -> Result<(), Error> {
        let (_aux_data, _user_config, _api) = check_message_against_transaction::<
            AuxData,
            UserConfig,
        >(signature_request, config)
        .map_err(|e| {
            Error::InvalidSignatureRequest(format!("Error comparing tx request and message: {}", e))
        })?;

        // can now use aux data and user configto apply constraints
        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(SubstrateProgram);
