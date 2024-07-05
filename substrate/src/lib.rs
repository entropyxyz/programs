use codec::Decode;
use entropy_programs_core::{bindgen::SignatureRequest, Error};
use serde::{de::DeserializeOwned, Deserialize};
pub use subxt::{utils::H256, Metadata, OfflineClient, PolkadotConfig};

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

pub trait HasFieldsAux {
    type SpecVersionType;
    type TransactionVersionType;

    fn spec_version(&self) -> &u32;
    fn transaction_version(&self) -> &u32;
}

#[derive(Deserialize)]
struct AuxDataStruct {
    spec_version: u32,
    transaction_version: u32,
}

impl HasFieldsAux for AuxDataStruct {
    type SpecVersionType = u32;
    type TransactionVersionType = u32;

    fn spec_version(&self) -> &u32 {
        &self.spec_version
    }

    fn transaction_version(&self) -> &u32 {
        &self.transaction_version
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

pub fn check_message_against_transaction<AuxData, ConfigData, UserConfig>(
    signature_request: SignatureRequest,
    config: Option<Vec<u8>>,
) -> Result<(), Error>
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

    Ok(())
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
