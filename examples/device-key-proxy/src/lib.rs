#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use entropy_programs_core::{bindgen::*, export_program, prelude::*, Error};

use base64::{prelude::BASE64_STANDARD, Engine};
use ed25519_dalek::{Signature as Ed25519Signature, VerifyingKey as Ed25519PublicKey};
use k256::ecdsa::{
    signature::Verifier, Signature as EcdsaSignature, VerifyingKey as EcdsaPublicKey,
};
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
/// Note how this uses JSON-native types only.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {
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
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {
    /// "ecdsa", "ed25519", "sr25519"
    pub public_key_type: String,
    /// base64-encoded public key
    pub public_key: String,
    /// base64-encoded signature
    pub signature: String,
    /// The context for the signature only needed in sr25519 signature type
    pub context: String,
}

trait DeviceKey {
    type PublicKey;
    type Signature;
    fn verify_signature(&self, message: &[u8], context: &[u8]) -> Result<(), Error>;
    fn from_base64(public_key: &[u8], signature: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
    fn pub_key_from_base64(public_key: &[u8]) -> Result<Self::PublicKey, Error>
    where
        Self: Sized;
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

    fn verify_signature(&self, message: &[u8], _context: &[u8]) -> Result<(), Error> {
        self.pub_key.verify(message, &self.signature).map_err(|_| {
            Error::InvalidSignatureRequest("Unable to verify ecdsa signature".to_string())
        })
    }

    fn from_base64(pub_key_encoded: &[u8], signature_encoded: &[u8]) -> Result<Self, Error> {
        let pub_key = Ecdsa::pub_key_from_base64(pub_key_encoded)?;
        let signature = EcdsaSignature::from_slice(
            BASE64_STANDARD
                .decode(signature_encoded)
                .map_err(|_| Error::InvalidSignatureRequest("ecdsa from_base64 error".to_string()))?
                .as_slice(),
        )
        .map_err(|_| Error::InvalidSignatureRequest("Invalid ecdsa signature".to_string()))?;
        Ok(Ecdsa { pub_key, signature })
    }

    fn pub_key_from_base64(pub_key_encoded: &[u8]) -> Result<Self::PublicKey, Error> {
        let pub_key = EcdsaPublicKey::from_sec1_bytes(
            BASE64_STANDARD
                .decode(pub_key_encoded)
                .map_err(|_| {
                    Error::InvalidSignatureRequest("ecdsa pub_key_from_base64 error".to_string())
                })?
                .as_slice(),
        )
        .map_err(|_| Error::InvalidSignatureRequest("Invalid ecdsa public key".to_string()))?;
        Ok(pub_key)
    }

    fn to_base64(&self) -> (String, String) {
        let pub_key_encoded = BASE64_STANDARD.encode(self.pub_key.to_encoded_point(true));
        let signature_encoded = BASE64_STANDARD.encode(self.signature.to_bytes());
        (pub_key_encoded, signature_encoded)
    }

    fn confirm_in_config(&self, config: &Config) -> Result<(), Error> {
        if !config.ecdsa_public_keys.contains(&self.pub_key) {
            return Err(Error::InvalidSignatureRequest(
                "ECDSA Public key not in config".to_string(),
            ));
        }
        Ok(())
    }
}

impl DeviceKey for Ed25519 {
    type PublicKey = Ed25519PublicKey;
    type Signature = Ed25519Signature;

    fn verify_signature(&self, message: &[u8], _context: &[u8]) -> Result<(), Error> {
        self.pub_key.verify(message, &self.signature).map_err(|_| {
            Error::InvalidSignatureRequest("Unable to verify ed25519 signature".to_string())
        })
    }

    fn pub_key_from_base64(public_key: &[u8]) -> Result<Self::PublicKey, Error>
    where
        Self: Sized,
    {
        let pub_key = Ed25519PublicKey::try_from(
            BASE64_STANDARD
                .decode(public_key)
                .map_err(|_| {
                    Error::InvalidSignatureRequest("ed25519 pub_key_from_base64 error".to_string())
                })?
                .as_slice(),
        )
        .map_err(|_| Error::InvalidSignatureRequest("Invalid ed25519 public key".to_string()))?;
        Ok(pub_key)
    }

    fn from_base64(pub_key_encoded: &[u8], signature_encoded: &[u8]) -> Result<Self, Error> {
        let pub_key = Ed25519::pub_key_from_base64(pub_key_encoded)?;
        let signature = Ed25519Signature::try_from(
            BASE64_STANDARD
                .decode(signature_encoded)
                .map_err(|_| {
                    Error::InvalidSignatureRequest("ed25519 from_base64 error".to_string())
                })?
                .as_slice(),
        )
        .unwrap();
        Ok(Ed25519 { pub_key, signature })
    }

    fn to_base64(&self) -> (String, String) {
        let pub_key_encoded = BASE64_STANDARD.encode(self.pub_key.to_bytes());
        let signature_encoded = BASE64_STANDARD.encode(self.signature.to_bytes());
        (pub_key_encoded, signature_encoded)
    }

    fn confirm_in_config(&self, config: &Config) -> Result<(), Error> {
        if !config.ed25519_public_keys.contains(&self.pub_key) {
            return Err(Error::InvalidSignatureRequest(
                "Ed25519 Public key not in config".to_string(),
            ));
        }
        Ok(())
    }
}

impl DeviceKey for Sr25519 {
    type PublicKey = Sr25519PublicKey;
    type Signature = Sr25519Signature;

    fn verify_signature(&self, message: &[u8], context: &[u8]) -> Result<(), Error> {
        let context = signing_context(context);
        self.pub_key
            .verify(context.bytes(message), &self.signature)
            .map_err(|_| {
                Error::InvalidSignatureRequest("Unable to verify sr25519 signature".to_string())
            })
    }

    fn from_base64(pub_key_encoded: &[u8], signature_encoded: &[u8]) -> Result<Self, Error> {
        let pub_key = Sr25519::pub_key_from_base64(pub_key_encoded)?;
        let signature = Sr25519Signature::from_bytes(
            BASE64_STANDARD
                .decode(signature_encoded)
                .map_err(|_| {
                    Error::InvalidSignatureRequest("sr25519 from_base64 error".to_string())
                })?
                .as_slice(),
        )
        .map_err(|_| Error::InvalidSignatureRequest("Invalid sr25519 signature".to_string()))?;
        Ok(Sr25519 { pub_key, signature })
    }

    fn pub_key_from_base64(pub_key_encoded: &[u8]) -> Result<Self::PublicKey, Error> {
        let pub_key = Sr25519PublicKey::from_bytes(
            BASE64_STANDARD
                .decode(pub_key_encoded)
                .map_err(|_| {
                    Error::InvalidSignatureRequest("sr25519 pub_key_from_base64 error".to_string())
                })?
                .as_slice(),
        )
        .map_err(|_| Error::InvalidSignatureRequest("Invalid sr25519 public key".to_string()))?;
        Ok(pub_key)
    }

    fn to_base64(&self) -> (String, String) {
        let pub_key_encoded = BASE64_STANDARD.encode(self.pub_key);
        let signature_encoded = BASE64_STANDARD.encode(self.signature.to_bytes());
        (pub_key_encoded, signature_encoded)
    }

    fn confirm_in_config(&self, config: &Config) -> Result<(), Error> {
        if !config.sr25519_public_keys.contains(&self.pub_key) {
            return Err(Error::InvalidSignatureRequest(
                "Sr25519 Public key not in config".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct DeviceKeyProxy;

impl Program for DeviceKeyProxy {
    fn evaluate(
        signature_request: SignatureRequest,
        raw_config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<Vec<u8>>>,
    ) -> Result<(), Error> {
        let config_json = serde_json::from_slice::<UserConfig>(
            raw_config
                .ok_or(Error::Evaluation("No config provided.".to_string()))?
                .as_slice(),
        )
        .map_err(|e| Error::Evaluation(format!("Failed to parse config: {}", e)))?;
        let aux_data_json = serde_json::from_slice::<AuxData>(
            signature_request
                .auxilary_data
                .ok_or(Error::InvalidSignatureRequest(
                    "No auxilary_data provided".to_string(),
                ))?
                .as_slice(),
        )
        .map_err(|e| {
            Error::InvalidSignatureRequest(format!("Failed to parse auxilary_data: {}", e))
        })?;

        let config = Config::try_from(config_json)?;

        // assert that the key in the aux data is in the config, and verify signature
        match aux_data_json.public_key_type.as_str() {
            "ecdsa" => {
                let verification_parameters = Ecdsa::from_base64(
                    aux_data_json.public_key.as_bytes(),
                    aux_data_json.signature.as_bytes(),
                )?;
                verification_parameters.confirm_in_config(&config)?;
                verification_parameters
                    .verify_signature(signature_request.message.as_slice(), b"")?;
            }
            "sr25519" => {
                let verification_parameters = Sr25519::from_base64(
                    aux_data_json.public_key.as_bytes(),
                    aux_data_json.signature.as_bytes(),
                )?;
                verification_parameters.confirm_in_config(&config)?;
                verification_parameters.verify_signature(
                    signature_request.message.as_slice(),
                    aux_data_json.context.as_bytes(),
                )?;
            }
            "ed25519" => {
                let verification_parameters = Ed25519::from_base64(
                    aux_data_json.public_key.as_bytes(),
                    aux_data_json.signature.as_bytes(),
                )?;
                verification_parameters.confirm_in_config(&config)?;
                verification_parameters
                    .verify_signature(signature_request.message.as_slice(), b"")?;
            }
            _ => {
                return Err(Error::InvalidSignatureRequest(
                    "Invalid public key type".to_string(),
                ))
            }
        }

        Ok(())
    }

    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

impl TryFrom<UserConfig> for Config {
    type Error = Error;

    fn try_from(config_json: UserConfig) -> Result<Config, Error> {
        let mut config = Config::default();

        if let Some(ecdsa_pub_keys) = config_json.ecdsa_public_keys {
            for encoded_key in ecdsa_pub_keys {
                config.ecdsa_public_keys.push(
                    Ecdsa::pub_key_from_base64(encoded_key.as_bytes()).map_err(|_| {
                        Error::InvalidSignatureRequest("config conversion ecdsa".to_string())
                    })?,
                );
            }
        }

        if let Some(sr25519_pub_keys) = config_json.sr25519_public_keys {
            for encoded_key in sr25519_pub_keys {
                let public_key =
                    Sr25519::pub_key_from_base64(encoded_key.as_bytes()).map_err(|_| {
                        Error::InvalidSignatureRequest("config conversion sr25519".to_string())
                    })?;
                config.sr25519_public_keys.push(public_key);
            }
        }

        if let Some(ed25519_pub_keys) = config_json.ed25519_public_keys {
            for encoded_key in ed25519_pub_keys {
                let public_key =
                    Ed25519::pub_key_from_base64(encoded_key.as_bytes()).map_err(|_| {
                        Error::InvalidSignatureRequest("config conversion ed25519".to_string())
                    })?;
                config.ed25519_public_keys.push(public_key);
            }
        }

        Ok(config)
    }
}

impl From<Config> for UserConfig {
    fn from(config: Config) -> UserConfig {
        let ecdsa_public_keys = config
            .ecdsa_public_keys
            .iter()
            .map(|key| {
                let encoded_key = BASE64_STANDARD.encode(key.to_encoded_point(true).as_bytes());
                encoded_key
            })
            .collect();
        let sr25519_public_keys = config
            .sr25519_public_keys
            .iter()
            .map(|key| {
                let encoded_key = BASE64_STANDARD.encode(key);
                encoded_key
            })
            .collect();
        let ed25519_public_keys = config
            .ed25519_public_keys
            .iter()
            .map(|key| {
                let encoded_key = BASE64_STANDARD.encode(key.as_bytes());
                encoded_key
            })
            .collect();

        UserConfig {
            ecdsa_public_keys: Some(ecdsa_public_keys),
            sr25519_public_keys: Some(sr25519_public_keys),
            ed25519_public_keys: Some(ed25519_public_keys),
        }
    }
}

export_program!(DeviceKeyProxy);
