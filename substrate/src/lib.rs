use codec::Decode;
use core::str::FromStr;
use entropy_programs_core::{bindgen::SignatureRequest, Error};
use serde::{de::DeserializeOwned, Deserialize};
pub use subxt::{
    dynamic::tx,
    ext::scale_value::{self, Composite, Value},
    utils::{AccountId32, H256},
    Metadata, OfflineClient, PolkadotConfig,
};
#[cfg(test)]
mod tests;
#[cfg(test)]
use serde::Serialize;

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

pub trait HasFieldsAux {
    fn genesis_hash(&self) -> &String;
    fn spec_version(&self) -> &u32;
    fn transaction_version(&self) -> &u32;
    fn pallet(&self) -> &String;
    fn function(&self) -> &String;
    fn values(&self) -> &String;
}

/// Info needed in AuxData to use these substrate helpers
#[cfg_attr(test, derive(Serialize, Debug, PartialEq))]
#[derive(Deserialize)]
pub struct AuxDataStruct {
    /// Genesis hash of the chain you are trying to talk to
    genesis_hash: String,
    /// Spec version of the chain to call
    spec_version: u32,
    /// Transaction version of the chain to call
    transaction_version: u32,
    /// Pallet name to call (ex: Balances)
    pallet: String,
    /// Function name to call (ex: transfer_allow_death)
    function: String,
    /// Values string encoded in a tuple Vec<(type_string, value_string)> (ex: vec![("account", "5x5......"), ("amount", "100")];)
    values: String,
}

impl HasFieldsAux for AuxDataStruct {
    fn spec_version(&self) -> &u32 {
        &self.spec_version
    }
    fn transaction_version(&self) -> &u32 {
        &self.transaction_version
    }
    fn pallet(&self) -> &String {
        &self.pallet
    }
    fn function(&self) -> &String {
        &self.function
    }
    fn values(&self) -> &String {
        &self.values
    }
    fn genesis_hash(&self) -> &String {
        &self.genesis_hash
    }
}
/// Checks message request against passed info to make sure they match
pub fn check_message_against_transaction<AuxData>(
    signature_request: SignatureRequest,
) -> Result<(AuxData, OfflineClient<PolkadotConfig>), Error>
where
    AuxData: DeserializeOwned + HasFieldsAux,
{
    let SignatureRequest {
        message,
        auxilary_data,
    } = signature_request;

    let aux_data_json = serde_json::from_slice::<AuxData>(
        auxilary_data
            .ok_or(Error::InvalidSignatureRequest(
                "No auxilary_data provided".to_string(),
            ))?
            .as_slice(),
    )
    .map_err(|e| Error::InvalidSignatureRequest(format!("Failed to parse auxilary_data: {}", e)))?;

    let api = get_offline_api(
        aux_data_json.genesis_hash().clone().to_string(),
        *aux_data_json.spec_version(),
        *aux_data_json.transaction_version(),
    )?;

    let deserialized: Vec<(&str, &str)> = serde_json::from_str(&aux_data_json.values())
        .map_err(|e| Error::Evaluation(format!("Failed to parse values: {}", e)))?;
    let encoding = handle_encoding(deserialized.clone())?;

    let balance_transfer_tx = tx(aux_data_json.pallet(), aux_data_json.function(), encoding);

    let call_data = api
        .tx()
        .call_data(&balance_transfer_tx)
        .map_err(|e| Error::Evaluation(format!("Failed to create transaction: {}", e)))?;

    let hex_message = hex::encode(message);
    let hex_call_data = hex::encode(call_data);
    let hex_genesis_hash = hex::encode(aux_data_json.genesis_hash());

    if !&hex_message.contains(&hex_call_data) && !&hex_message.contains(&hex_genesis_hash) {
        return Err(Error::Evaluation(format!(
            "Signatures don't match, message: {:?}, calldata: {:?}, genesis_hash: {:?}",
            hex_message, hex_call_data, hex_genesis_hash,
        )));
    }

    Ok((aux_data_json, api))
}

/// Creates an offline api instance
/// Chain endpoint set on launch
pub fn get_offline_api(
    hash: String,
    spec_version: u32,
    transaction_version: u32,
) -> Result<OfflineClient<PolkadotConfig>, Error> {
    let genesis_hash = {
        let bytes = hex::decode(hash)
            .map_err(|e| Error::InvalidSignatureRequest(format!("Failed to parse bytes: {}", e)))?;
        H256::from_slice(&bytes)
    };

    // 2. A runtime version (system_version constant on a Substrate node has these):
    let runtime_version = subxt::backend::RuntimeVersion {
        spec_version,
        transaction_version,
    };

    // Metadata comes from metadata.rs, which is a &[u8] representation of the metadata
    // It takes a lot of space and is clunky.....I am very open to better ideas
    let metadata = Metadata::decode(&mut &*METADATA)
        .map_err(|e| Error::InvalidSignatureRequest(format!("Failed to parse metadata: {}", e)))?;

    // Create an offline client using the details obtained above:
    Ok(OfflineClient::<PolkadotConfig>::new(
        genesis_hash,
        runtime_version,
        metadata,
    ))
}

/// Hacky way to handle encoding takes a tuple of (string_type, string) see TODO: Make Issue
pub fn handle_encoding(encodings: Vec<(&str, &str)>) -> Result<Vec<Value>, Error> {
    let mut values: Vec<Value> = vec![];
    for encoding in encodings {
        let value = match encoding.0 {
            "account" => {
                let account_id = AccountId32::from_str(&encoding.1).map_err(|e| {
                    Error::InvalidSignatureRequest(format!("account id issue: {}", e))
                })?;
                Ok(Value::unnamed_variant(
                    "Id",
                    vec![Value::from_bytes(account_id)],
                ))
            }
            "true" => Ok(Value::bool(true)),
            "false" => Ok(Value::bool(false)),
            "string" => Ok(Value::string(encoding.1.to_string())),
            "amount" => {
                let number: u128 = encoding.1.parse().map_err(|e| {
                    Error::InvalidSignatureRequest(format!("parse number issue: {}", e))
                })?;
                Ok(Value::u128(number))
            }
            _ => Err(Error::InvalidSignatureRequest(
                "Incorrect value heading".to_string(),
            )),
        }?;
        values.push(value);
    }
    Ok(values)
}
