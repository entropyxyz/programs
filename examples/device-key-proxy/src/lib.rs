#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{format, string::{String, ToString}, vec::Vec};

use entropy_programs_core::{bindgen::*, export_program, prelude::*, Error};

use serde::{Serialize, Deserialize};
use base64::{prelude::BASE64_STANDARD, Engine};
use k256::ecdsa::{signature::Verifier, VerifyingKey as EcdsaPublicKey, Signature as EcdsaSignature};
use schnorrkel::{PublicKey as Sr25519PublicKey, Signature as Sr25519Signature, signing_context};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
/// Note how this uses JSON-native types only.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ConfigJson {
    /// base64-encoded compressed point (33-byte) ECDSA public keys, (eg. "A572dqoue5OywY/48dtytQimL9WO0dpSObaFbAxoEWW9")
    pub ecdsa_public_keys: Option<Vec<String>>,
    pub sr25519_public_keys: Option<Vec<String>>,
}

/// Used by the program to verify signatures
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Config {
    pub ecdsa_public_keys: Vec<EcdsaPublicKey>,
    pub sr25519_public_keys: Vec<Sr25519PublicKey>,
}

/// JSON representation of the auxiliary data
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxDataJson {
    /// "ecdsa" or "sr25519"
    pub public_key_type: String,
    /// base64-encoded public key
    pub public_key: String,
    /// base64-encoded signature
    pub signature: String,
}

pub struct AuxData {
    pub verification_parameters: VerificationParameters,
}

/// Everything needed to verify a message against a signature/pubkey pair
pub enum VerificationParameters {
    Ecdsa(EcdsaPublicKey, EcdsaSignature),
    Sr25519(Sr25519PublicKey, Sr25519Signature),
}

pub struct DeviceKeyProxy;

impl Program for DeviceKeyProxy {
    fn evaluate(signature_request: SignatureRequest, raw_config: Option<Vec<u8>>) -> Result<(), Error> {
        let config_json = serde_json::from_slice::<ConfigJson>(
            raw_config.ok_or(Error::Evaluation("No config provided.".to_string()))?.as_slice()
        ).map_err(|e| Error::Evaluation(format!("Failed to parse config: {}", e)))?;
        let aux_data_json = serde_json::from_slice::<AuxDataJson>(
            signature_request.auxilary_data.ok_or(Error::InvalidSignatureRequest("No auxilary_data provided".to_string()))?.as_slice()
        ).map_err(|e| Error::InvalidSignatureRequest(format!("Failed to parse auxilary_data: {}", e)))?;

        let config = Config::from(config_json);
        let aux_data = AuxData::from(aux_data_json);

        // assert that the key in the aux data is in the config, and verify signature
        match aux_data.verification_parameters {
            VerificationParameters::Ecdsa(public_key, signature) => {
                if !config.ecdsa_public_keys.contains(&public_key) {
                    return Err(Error::InvalidSignatureRequest("Public key not in config".to_string()));
                }
                if public_key.verify(signature_request.message.as_slice(), &signature).is_err() {
                    return Err(Error::InvalidSignatureRequest("Invalid signature".to_string()));
                }
            }
            VerificationParameters::Sr25519(public_key, signature) => {
                if !config.sr25519_public_keys.contains(&public_key) {
                    return Err(Error::InvalidSignatureRequest("Public key not in config".to_string()));
                }
                // `context` is required for sr25519 signature verification
                let context = signing_context(b"");
                if public_key.verify(context.bytes(signature_request.message.as_slice()), &signature).is_err() {
                    return Err(Error::InvalidSignatureRequest("Invalid signature".to_string()));
                }
            }
        }


        Ok(())
    }

    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

impl From<ConfigJson> for Config {
    fn from(config_json: ConfigJson) -> Config {
        let mut config = Config::default();

        if let Some(ecdsa_pub_keys) = config_json.ecdsa_public_keys {
            for encoded_key in ecdsa_pub_keys {
                let key = BASE64_STANDARD.decode(encoded_key.as_bytes()).unwrap();
                let public_key = EcdsaPublicKey::from_sec1_bytes(key.as_slice()).unwrap();
                config.ecdsa_public_keys.push(public_key);
            }
        }

        if let Some(sr25519_pub_keys) = config_json.sr25519_public_keys {
            for encoded_key in sr25519_pub_keys {
                let key = BASE64_STANDARD.decode(encoded_key.as_bytes()).unwrap();
                let public_key = Sr25519PublicKey::from_bytes(key.as_slice()).unwrap();
                config.sr25519_public_keys.push(public_key);
            }
        }

        config
    }
}

impl From<Config> for ConfigJson {
    fn from(config: Config) -> ConfigJson {
        let ecdsa_public_keys = config.ecdsa_public_keys.iter().map(|key| {
            let encoded_key = BASE64_STANDARD.encode(key.to_encoded_point(true).as_bytes());
            encoded_key
        }).collect();
        let sr25519_public_keys = config.sr25519_public_keys.iter().map(|key| {
            let encoded_key = BASE64_STANDARD.encode(key);
            encoded_key
        }).collect();

        ConfigJson {
            ecdsa_public_keys: Some(ecdsa_public_keys),
            sr25519_public_keys: Some(sr25519_public_keys),
        }
    }
}

impl From<AuxDataJson> for AuxData {
    fn from(aux_data_json: AuxDataJson) -> AuxData {
        let AuxDataJson { public_key_type, public_key, signature } = aux_data_json;

        let verification_parameters = match public_key_type.as_str() {
            "ecdsa" => {
                let decoded_signature = BASE64_STANDARD.decode(signature.as_bytes()).unwrap();
                let decoded_public_key = BASE64_STANDARD.decode(public_key.as_bytes()).unwrap();
                let public_key = EcdsaPublicKey::from_sec1_bytes(decoded_public_key.as_slice()).unwrap();
                let signature = EcdsaSignature::from_slice(decoded_signature.as_slice()).unwrap();
                VerificationParameters::Ecdsa(public_key, signature)
            }
            "sr25519" => {
                let decoded_signature = BASE64_STANDARD.decode(signature.as_bytes()).unwrap();
                let decoded_public_key = BASE64_STANDARD.decode(public_key.as_bytes()).unwrap();
                let public_key = Sr25519PublicKey::from_bytes(decoded_public_key.as_slice()).unwrap();
                let signature = Sr25519Signature::from_bytes(decoded_signature.as_slice()).unwrap();
                VerificationParameters::Sr25519(public_key, signature)
            }
            _ => panic!("Invalid public key type"),
        };

        AuxData {
            verification_parameters
        }
    }
}

export_program!(DeviceKeyProxy);

#[cfg(test)]
mod tests {
    use super::*;

    use k256::ecdsa::{SigningKey as EcdsaKeypair, Signature as EcdsaSignature, signature::Signer};
    use schnorrkel::{Keypair as Sr25519Keypair, Signature as Sr25519Signature};
    use rand_core::OsRng;

    struct TestKeys {
        ecdsa_keys: Vec<EcdsaKeypair>,
        sr25519_keys: Vec<Sr25519Keypair>
    }

    #[test]
    fn test_ok_for_only_device_key_signatures() {
        let device_keys = generate_test_keys();
        let non_device_keys = generate_test_keys();

        let config = Config {
            ecdsa_public_keys: device_keys.ecdsa_keys.iter().map(|key| EcdsaPublicKey::from(key)).collect(),
            sr25519_public_keys: device_keys.sr25519_keys.iter().map(|key| key.public).collect(),
        };
        let json_config = ConfigJson::from(config.clone());
        dbg!(json_config.clone());

        let message: &str = "this is some message that we want to sign if its from a valid device key";

        // constrtuct signature request from device key (for positive test)
        let ecdsa_device_key_signature: EcdsaSignature = device_keys.ecdsa_keys[0].try_sign(message.as_bytes()).unwrap();
        let device_key_aux_data_json = AuxDataJson {
            public_key_type: "ecdsa".to_string(),
            public_key: BASE64_STANDARD.encode(device_keys.ecdsa_keys[0].verifying_key().to_encoded_point(true).as_bytes()),
            signature: BASE64_STANDARD.encode(ecdsa_device_key_signature.to_bytes()),
        };
        let request_from_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: Some(serde_json::to_string(&device_key_aux_data_json).unwrap().into_bytes())
        };
        // construct signature request from non-device key (for negative test)
        let ecdsa_non_device_key_signature: EcdsaSignature = non_device_keys.ecdsa_keys[0].try_sign(message.as_bytes()).unwrap();
        let non_device_key_aux_data_json = AuxDataJson {
            public_key_type: "ecdsa".to_string(),
            public_key: BASE64_STANDARD.encode(non_device_keys.ecdsa_keys[0].verifying_key().to_encoded_point(true).as_bytes()),
            signature: BASE64_STANDARD.encode(ecdsa_non_device_key_signature.to_bytes()),
        };
        let request_from_non_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: Some(serde_json::to_string(&non_device_key_aux_data_json).unwrap().into_bytes())
        };

        let config_bytes = serde_json::to_vec(&json_config).unwrap();
        // positive
        assert!(DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes.clone())).is_ok());
        // negative
        assert!(DeviceKeyProxy::evaluate(request_from_non_device_key, Some(config_bytes)).is_err());
    }

    // #[test]
    // fn test_fails_with_empty_aux_data() {
    //     let (device_keys, _)= generate_test_keys();

    //     let config = ConfigJson {
    //         ecdsa_public_keys: Some(device_keys.iter().map(|key| {
    //             let public_key = EcdsaPublicKey::from(key);
    //             let encoded_key = BASE64_STANDARD.encode(public_key.to_encoded_point(true).as_bytes());
    //             encoded_key
    //         }).collect()),
    //     };
    //     let config_bytes = serde_json::to_vec(&config).unwrap();

    //     let message = "this is some message that we want to sign if its from a valid device key";
    //     let _device_key_signature: EcdsaSignature = device_keys[0].try_sign(message.as_bytes()).unwrap();

    //     let request_from_device_key = SignatureRequest {
    //         message: message.to_string().into_bytes(),
    //         auxilary_data: None,
    //     };

    //     assert!(DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes)).is_err());
    // }

    /// Generates keys that can be used for testing
    fn generate_test_keys() -> TestKeys {
        let ecdsa_keys: Vec<EcdsaKeypair> = (0..3)
            .map(|_| EcdsaKeypair::random(&mut OsRng))
            .collect();
        let sr25519_keys: Vec<Sr25519Keypair> = (0..3)
            .map(|_| Sr25519Keypair::generate_with(&mut OsRng))
            .collect();

        TestKeys {
            ecdsa_keys,
            sr25519_keys 
        }
    }
}
