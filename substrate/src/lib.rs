use codec::Decode;
use core::str::FromStr;
use entropy_programs_core::{bindgen::SignatureRequest, Error};
use serde::{de::DeserializeOwned, Deserialize};
pub use subxt::{
    dynamic::tx,
    ext::scale_value::Value,
    utils::{AccountId32, H256},
    Metadata, OfflineClient, PolkadotConfig,
};

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

pub trait HasFieldsAux {
    type SpecVersionType;
    type TransactionVersionType;
    type StringAccountIdType;
    type AmountType;

    fn spec_version(&self) -> &u32;
    fn transaction_version(&self) -> &u32;
    fn string_account_id(&self) -> &String;
    fn amount(&self) -> &u128;
}

#[derive(Deserialize)]
struct AuxDataStruct {
    spec_version: u32,
    transaction_version: u32,
    string_account_id: String,
    amount: u128,
}

impl HasFieldsAux for AuxDataStruct {
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

pub trait HasFieldsConfig {
    type GenesisHashType;

    fn genesis_hash(&self) -> &String;
}

#[derive(Deserialize)]
struct UserConfigStruct {
    genesis_hash: String,
}

impl HasFieldsConfig for UserConfigStruct {
    type GenesisHashType = String;

    fn genesis_hash(&self) -> &String {
        &self.genesis_hash
    }
}

pub fn check_message_against_transaction<AuxData, UserConfig>(
    signature_request: SignatureRequest,
    config: Option<Vec<u8>>,
) -> Result<(AuxData, UserConfig, OfflineClient<PolkadotConfig>), Error>
where
    AuxData: DeserializeOwned + HasFieldsAux,
    UserConfig: DeserializeOwned + HasFieldsConfig,
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

    let typed_config = serde_json::from_slice::<UserConfig>(
        config
            .ok_or(Error::Evaluation("No config provided.".to_string()))?
            .as_slice(),
    )
    .map_err(|e| Error::Evaluation(format!("Failed to parse config: {}", e)))?;

    let api = get_offline_api(
        typed_config.genesis_hash().clone().to_string(),
        *aux_data_json.spec_version(),
        *aux_data_json.transaction_version(),
    )?;
    // TODO: generalize this
    let account_id = AccountId32::from_str(aux_data_json.string_account_id())
        .map_err(|e| Error::InvalidSignatureRequest(format!("account id issue: {}", e)))?;

    let balance_transfer_tx = tx(
        "Balances",
        "transfer_allow_death",
        vec![
            Value::unnamed_variant("Id", vec![Value::from_bytes(account_id)]),
            Value::u128(*aux_data_json.amount()),
        ],
    );

    let call_data = api.tx().call_data(&balance_transfer_tx).unwrap();

    let hex_message = hex::encode(message);
    let hex_call_data = hex::encode(call_data);
    let hex_genesis_hash = hex::encode(typed_config.genesis_hash());

    if !&hex_message.contains(&hex_call_data) && !&hex_message.contains(&hex_genesis_hash) {
        return Err(Error::Evaluation(format!(
            "Signatures don't match, message: {:?}, calldata: {:?}, genesis_hash: {:?}",
            hex_message, hex_call_data, hex_genesis_hash,
        )));
    }

    Ok((aux_data_json, typed_config, api))
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
