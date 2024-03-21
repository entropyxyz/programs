#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{format, string::{String, ToString}, vec::Vec};

use entropy_programs_core::{bindgen::*, export_program, prelude::*, Error};

use serde::{Serialize, Deserialize};
use base64::{prelude::BASE64_STANDARD, Engine};
use k256::ecdsa::{signature::Verifier, VerifyingKey as EcdsaPublicKey, Signature as EcdsaSignature};
use ed25519_dalek::{VerifyingKey as Ed25519PublicKey, Signature as Ed25519Signature};
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
    pub ed25519_public_keys: Option<Vec<String>>,
}

/// Used by the program to verify signatures
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Config {
    pub ecdsa_public_keys: Vec<EcdsaPublicKey>,
    pub sr25519_public_keys: Vec<Sr25519PublicKey>,
    pub ed25519_public_keys: Vec<Ed25519PublicKey>,
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

trait DeviceKey {
    type PublicKey;
    type Signature;
    fn verify_signature(&self, message: &[u8]) -> Result<(), Error>;
    fn from_base64(public_key: &[u8], signature: &[u8]) -> Result<Self, Error> where Self: Sized;
    fn pub_key_from_base64(public_key: &[u8]) -> Result<Self::PublicKey, Error> where Self: Sized;
    fn to_base64(&self) -> (String, String);
    // Checks that the public key is included in the config
    fn confirm_in_config(&self, config: &Config) -> Result<(), Error>;
}

struct VerificationParameters<P, S> {
    pub_key: P,
    signature: S,
}

type Ecdsa = VerificationParameters<EcdsaPublicKey, EcdsaSignature>;
type Sr25519 = VerificationParameters<Sr25519PublicKey, Sr25519Signature>;
type Ed25519 = VerificationParameters<Ed25519PublicKey, Ed25519Signature>;

impl DeviceKey for Ecdsa {
    type PublicKey = EcdsaPublicKey;
    type Signature = EcdsaSignature;

    fn verify_signature(&self, message: &[u8]) -> Result<(), Error> {
        self.pub_key.verify(message, &self.signature).map_err(|_| Error::InvalidSignatureRequest("Unable to verify ecdsa signature".to_string()))
    }

    fn from_base64(pub_key_encoded: &[u8], signature_encoded: &[u8]) -> Result<Self, Error> {
        let pub_key = Ecdsa::pub_key_from_base64(pub_key_encoded)?;
        let signature = EcdsaSignature::from_slice(BASE64_STANDARD.decode(signature_encoded).unwrap().as_slice()).map_err(|_| Error::InvalidSignatureRequest("Invalid ecdsa signature".to_string()))?;
        Ok(Ecdsa {
            pub_key,
            signature
        })
    }

    fn pub_key_from_base64(pub_key_encoded: &[u8]) -> Result<Self::PublicKey, Error> {
        let pub_key = EcdsaPublicKey::from_sec1_bytes(BASE64_STANDARD.decode(pub_key_encoded).unwrap().as_slice()).map_err(|_| Error::InvalidSignatureRequest("Invalid ecdsa public key".to_string()))?;
        Ok(pub_key)
    }

    fn to_base64(&self) -> (String, String) {
        let pub_key_encoded = BASE64_STANDARD.encode(self.pub_key.to_encoded_point(true));
        let signature_encoded = BASE64_STANDARD.encode(self.signature.to_bytes());
        (pub_key_encoded, signature_encoded)
    }

    fn confirm_in_config(&self, config: &Config) -> Result<(), Error> {
        if !config.ecdsa_public_keys.contains(&self.pub_key) {
            return Err(Error::InvalidSignatureRequest("ECDSA Public key not in config".to_string()));
        }
        Ok(())
    }
}

impl DeviceKey for Ed25519 {
    type PublicKey = Ed25519PublicKey;
    type Signature = Ed25519Signature;

    fn verify_signature(&self, message: &[u8]) -> Result<(), Error> {
        self.pub_key.verify(message, &self.signature).map_err(|_| Error::InvalidSignatureRequest("Unable to verify ed25519 signature".to_string()))
    }

    fn pub_key_from_base64(public_key: &[u8]) -> Result<Self::PublicKey, Error> where Self: Sized {
        let pub_key = Ed25519PublicKey::try_from(BASE64_STANDARD.decode(public_key).unwrap().as_slice()).map_err(|_| Error::InvalidSignatureRequest("Invalid ed25519 public key".to_string()))?;
        Ok(pub_key)
    }

    fn from_base64(pub_key_encoded: &[u8], signature_encoded: &[u8]) -> Result<Self, Error> {
        let pub_key = Ed25519::pub_key_from_base64(pub_key_encoded)?;
        let signature = Ed25519Signature::try_from(BASE64_STANDARD.decode(signature_encoded).unwrap().as_slice()).unwrap();
        Ok(Ed25519 {
            pub_key,
            signature
        })
    }

    fn to_base64(&self) -> (String, String) {
        let pub_key_encoded = BASE64_STANDARD.encode(self.pub_key.to_bytes());
        let signature_encoded = BASE64_STANDARD.encode(self.signature.to_bytes());
        (pub_key_encoded, signature_encoded)
    }

    fn confirm_in_config(&self, config: &Config) -> Result<(), Error> {
        if !config.ed25519_public_keys.contains(&self.pub_key) {
            return Err(Error::InvalidSignatureRequest("Ed25519 Public key not in config".to_string()));
        }
        Ok(())
    }

}

impl DeviceKey for Sr25519 {
    type PublicKey = Sr25519PublicKey;
    type Signature = Sr25519Signature;

    fn verify_signature(&self, message: &[u8]) -> Result<(), Error> {
        let context = signing_context(b"");
        self.pub_key.verify(context.bytes(message), &self.signature).map_err(|_| Error::InvalidSignatureRequest("Unable to verify sr25519 signature".to_string()))
    }

    fn from_base64(pub_key_encoded: &[u8], signature_encoded: &[u8]) -> Result<Self, Error> {
        let pub_key = Sr25519::pub_key_from_base64(pub_key_encoded)?;
        let signature = Sr25519Signature::from_bytes(signature_encoded).map_err(|_| Error::InvalidSignatureRequest("Invalid sr25519 signature".to_string()))?;
        Ok(Sr25519 {
            pub_key,
            signature
        })
    }

    fn pub_key_from_base64(pub_key_encoded: &[u8]) -> Result<Self::PublicKey, Error> {
        let pub_key = Sr25519PublicKey::from_bytes(BASE64_STANDARD.decode(pub_key_encoded).unwrap().as_slice()).map_err(|_| Error::InvalidSignatureRequest("Invalid sr25519 public key".to_string()))?;
        Ok(pub_key)
    }

    fn to_base64(&self) -> (String, String) {
        let pub_key_encoded = BASE64_STANDARD.encode(self.pub_key);
        let signature_encoded = BASE64_STANDARD.encode(self.signature.to_bytes());
        (pub_key_encoded, signature_encoded)
    }

    fn confirm_in_config(&self, config: &Config) -> Result<(), Error> {
        if !config.sr25519_public_keys.contains(&self.pub_key) {
            return Err(Error::InvalidSignatureRequest("Sr25519 Public key not in config".to_string()));
        }
        Ok(())
    }
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

        // assert that the key in the aux data is in the config, and verify signature
        match aux_data_json.public_key_type.as_str() {
            "ecdsa" => {
                let verification_parameters = Ecdsa::from_base64(aux_data_json.public_key.as_bytes(), aux_data_json.signature.as_bytes())?;
                verification_parameters.confirm_in_config(&config)?;
                verification_parameters.verify_signature(signature_request.message.as_slice())?;
            }
            "sr25519" => {
                let verification_parameters = Sr25519::from_base64(aux_data_json.public_key.as_bytes(), aux_data_json.signature.as_bytes())?;
                verification_parameters.confirm_in_config(&config)?;
                verification_parameters.verify_signature(signature_request.message.as_slice())?;
            }
            "ed25519" => {
                let verification_parameters = Ed25519::from_base64(aux_data_json.public_key.as_bytes(), aux_data_json.signature.as_bytes())?;
                verification_parameters.confirm_in_config(&config)?;
                verification_parameters.verify_signature(signature_request.message.as_slice())?;
            }
            _ => return Err(Error::InvalidSignatureRequest("Invalid public key type".to_string())),
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
                // let key = EcdsaPublicKey::from_base64(encoded_key.as_bytes()).unwrap();
                config.ecdsa_public_keys.push(Ecdsa::pub_key_from_base64(encoded_key.as_bytes()).unwrap());
            }
        }

        if let Some(sr25519_pub_keys) = config_json.sr25519_public_keys {
            for encoded_key in sr25519_pub_keys {
                let public_key = Sr25519::pub_key_from_base64(encoded_key.as_bytes()).unwrap();
                config.sr25519_public_keys.push(public_key);
            }
        }

        if let Some(ed25519_pub_keys) = config_json.ed25519_public_keys {
            for encoded_key in ed25519_pub_keys {
                let public_key = Ed25519::pub_key_from_base64(encoded_key.as_bytes()).unwrap();
                config.ed25519_public_keys.push(public_key);
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
        let ed25519_public_keys = config.ed25519_public_keys.iter().map(|key| {
            let encoded_key = BASE64_STANDARD.encode(key.as_bytes());
            encoded_key
        }).collect();

        ConfigJson {
            ecdsa_public_keys: Some(ecdsa_public_keys),
            sr25519_public_keys: Some(sr25519_public_keys),
            ed25519_public_keys: Some(ed25519_public_keys),
        }
    }
}

export_program!(DeviceKeyProxy);

#[cfg(test)]
mod tests {
    use super::*;

    use k256::ecdsa::{SigningKey as EcdsaKeypair, Signature as EcdsaSignature, signature::Signer};
    use schnorrkel::{Keypair as Sr25519Keypair, Signature as Sr25519Signature, signing_context};
    use ed25519_dalek::{SigningKey as Ed25519Keypair, Signature as Ed25519Signature};
    use rand_core::OsRng;

    struct TestKeys {
        ecdsa_keys: Vec<EcdsaKeypair>,
        sr25519_keys: Vec<Sr25519Keypair>,
        ed25519_keys: Vec<Ed25519Keypair>,
    }

    #[test]
    fn test_ok_for_only_device_key_signatures() {
        let device_keys = generate_test_keys();
        let non_device_keys = generate_test_keys();

        let config = Config {
            ecdsa_public_keys: device_keys.ecdsa_keys.iter().map(|key| EcdsaPublicKey::from(key)).collect(),
            sr25519_public_keys: device_keys.sr25519_keys.iter().map(|key| key.public).collect(),
            ed25519_public_keys: device_keys.ed25519_keys.iter().map(|key| key.verifying_key()).collect(),

        };
        let json_config = ConfigJson::from(config.clone());

        let message: &str = "this is some message that we want to sign if its from a valid device key";

        // constrtuct signature request from device key (for positive test)
        let ecdsa_device_key_signature: EcdsaSignature = device_keys.ecdsa_keys[0].try_sign(message.as_bytes()).unwrap();
        let device_key_aux_data_json_edcsa = AuxDataJson {
            public_key_type: "ecdsa".to_string(),
            public_key: BASE64_STANDARD.encode(device_keys.ecdsa_keys[0].verifying_key().to_encoded_point(true).as_bytes()),
            signature: BASE64_STANDARD.encode(ecdsa_device_key_signature.to_bytes()),
        };
        let mut request_from_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: Some(serde_json::to_string(&device_key_aux_data_json_edcsa).unwrap().into_bytes())
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
        // positive for edcsa
        assert!(DeviceKeyProxy::evaluate(request_from_device_key.clone(), Some(config_bytes.clone())).is_ok());
        // let context = signing_context(b"");
        // #[cfg(feature = "getrandom")]
        // let sr25519_device_key_signature: Sr25519Signature = device_keys.sr25519_keys[0].sign(context.bytes(message.as_bytes()));
        // let device_key_aux_data_json_sr25519 = AuxDataJson {
        //     public_key_type: "sr25519".to_string(),
        //     public_key: BASE64_STANDARD.encode(device_keys.sr25519_keys[0].public),
        //     signature: BASE64_STANDARD.encode(sr25519_device_key_signature.to_bytes()),
        // };
        // request_from_device_key.auxilary_data = Some(serde_json::to_string(&device_key_aux_data_json_sr25519).unwrap().into_bytes());
        // // positive for sr25519
        // assert!(DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes.clone())).is_ok());
        // // positive for ed25519
        let ed25519_device_key_signature: Ed25519Signature = device_keys.ed25519_keys[0].sign(message.as_bytes());
        let device_key_aux_data_json_ed25519 = AuxDataJson {
            public_key_type: "ed25519".to_string(),
            public_key: BASE64_STANDARD.encode(device_keys.ed25519_keys[0].verifying_key()),
            signature: BASE64_STANDARD.encode(ed25519_device_key_signature.to_bytes()),
        };
        request_from_device_key.auxilary_data = Some(serde_json::to_string(&device_key_aux_data_json_ed25519).unwrap().into_bytes());
        DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes.clone())).unwrap();

        // negative
        assert!(DeviceKeyProxy::evaluate(request_from_non_device_key, Some(config_bytes)).is_err());
    }

    #[test]
    fn test_fails_with_empty_aux_data() {
        let device_keys = generate_test_keys();

        let config = ConfigJson {
            ecdsa_public_keys: Some(device_keys.ecdsa_keys.iter().map(|key| {
                let public_key = EcdsaPublicKey::from(key);
                let encoded_key = BASE64_STANDARD.encode(public_key.to_encoded_point(true).as_bytes());
                encoded_key
            }).collect()),
            sr25519_public_keys: None,
            ed25519_public_keys: None,
        };
        let config_bytes = serde_json::to_vec(&config).unwrap();

        let message = "this is some message that we want to sign if its from a valid device key";
        let _device_key_signature: EcdsaSignature = device_keys.ecdsa_keys[0].try_sign(message.as_bytes()).unwrap();

        let request_from_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes)).is_err());
    }

    /// Generates keys that can be used for testing
    fn generate_test_keys() -> TestKeys {
        let ecdsa_keys: Vec<EcdsaKeypair> = (0..3)
            .map(|_| EcdsaKeypair::random(&mut OsRng))
            .collect();
        let sr25519_keys: Vec<Sr25519Keypair> = (0..3)
            .map(|_| Sr25519Keypair::generate_with(&mut OsRng))
            .collect();
        let ed25519_keys: Vec<Ed25519Keypair> = (0..3)
            .map(|_| Ed25519Keypair::generate(&mut OsRng))
            .collect();

        TestKeys {
            ecdsa_keys,
            sr25519_keys,
            ed25519_keys
        }
    }
}
