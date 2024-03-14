#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{format, string::{String, ToString}, vec::Vec};

use entropy_programs_core::{bindgen::*, export_program, prelude::*};

use serde::{Serialize, Deserialize};
use k256::ecdsa::{VerifyingKey, Signature};
use base64::{prelude::BASE64_STANDARD, Engine};
use k256::ecdsa::signature::Verifier;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct DeviceKeyProxyConfig {
    /// base64-encoded compressed point (33-byte) ECDSA public keys, (eg. "A572dqoue5OywY/48dtytQimL9WO0dpSObaFbAxoEWW9")
    pub device_keys: Vec<String>,
}

pub struct DeviceKeyProxy;

impl Program for DeviceKeyProxy {
    fn evaluate(signature_request: SignatureRequest, raw_config: Option<Vec<u8>>) -> Result<(), Error> {
        let config = serde_json::from_slice::<DeviceKeyProxyConfig>(
                raw_config.ok_or(Error::Evaluation("No config provided.".to_string()))?.as_slice()
            ).map_err(|e| Error::Evaluation(format!("Failed to parse config: {}", e)))?;
        
        let public_keys = config.device_keys.iter().map(|encoded_key| {
            let key = BASE64_STANDARD.decode(encoded_key.as_bytes()).map_err(|_| {
                Error::InvalidSignatureRequest("Could not parse base64 public key".to_string())
            }).unwrap();
            VerifyingKey::from_sec1_bytes(key.as_slice()).unwrap()
        }).collect::<Vec<VerifyingKey>>();
        
        let signature: Signature = match signature_request.auxilary_data {
            Some(base64_sig) => {
                let decoded_signature = BASE64_STANDARD.decode(base64_sig).map_err(|_| {
                    Error::InvalidSignatureRequest("Could not parse base64 signature".to_string())
                })?;
                Signature::try_from(decoded_signature.as_slice()).map_err(|_| {
                    Error::InvalidSignatureRequest("Could not parse base64 input to ecdsa signature".to_string())
                })?
            }
            None => {
                return Err(Error::InvalidSignatureRequest(
                    "No signature provided in auxilary_data".to_string(),
                ))
            }
        };

        public_keys.iter().find(|key| key.verify(signature_request.message.as_slice(), &signature).is_ok())
            .ok_or(Error::InvalidSignatureRequest("Signature is not valid for any key".to_string()))?;

        Ok(())
    }

    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(DeviceKeyProxy);

#[cfg(test)]
mod tests {
    use super::*;

    use k256::ecdsa::{SigningKey, Signature, signature::Signer};
    use rand_core::OsRng;

    #[test]
    fn test_ok_for_only_device_key_signatures() {
        let (device_keys, non_device_keys) = key_generation();

        let config = DeviceKeyProxyConfig {
            device_keys: device_keys.iter().map(|key| {
                let public_key = VerifyingKey::from(key);
                let encoded_key = BASE64_STANDARD.encode(public_key.to_encoded_point(true).as_bytes());
                println!("{}", encoded_key);
                encoded_key
            }).collect(),
        };
        let config_bytes = serde_json::to_vec(&config).unwrap();

        let message = "this is some message that we want to sign if its from a valid device key";
        let device_key_signature: Signature = device_keys[0].try_sign(message.as_bytes()).unwrap();
        let non_device_key_signature: Signature = non_device_keys[0].try_sign(message.as_bytes()).unwrap();

        let request_from_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: Some(BASE64_STANDARD.encode(device_key_signature.to_bytes()).into_bytes()),
        };
        let request_from_non_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: Some(BASE64_STANDARD.encode(non_device_key_signature.to_bytes()).into_bytes()),
        };

        assert!(DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes.clone())).is_ok());
        assert!(DeviceKeyProxy::evaluate(request_from_non_device_key, Some(config_bytes)).is_err());
    }

    #[test]
    fn test_fails_with_empty_aux_data() {
        let (device_keys, _)= key_generation();

        let config = DeviceKeyProxyConfig {
            device_keys: device_keys.iter().map(|key| {
                let public_key = VerifyingKey::from(key);
                let encoded_key = BASE64_STANDARD.encode(public_key.to_encoded_point(true).as_bytes());
                encoded_key
            }).collect(),
        };
        let config_bytes = serde_json::to_vec(&config).unwrap();

        let message = "this is some message that we want to sign if its from a valid device key";
        let _device_key_signature: Signature = device_keys[0].try_sign(message.as_bytes()).unwrap();

        let request_from_device_key = SignatureRequest {
            message: message.to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes)).is_err());
    }

    fn key_generation() -> (Vec<SigningKey>, Vec<SigningKey>) {
        let keys: Vec<SigningKey> = (0..5)
            .map(|_| SigningKey::random(&mut OsRng))
            .collect();
        let (device_keys, non_device_keys) = keys.split_at(3);
        (device_keys.to_vec(), non_device_keys.to_vec())
    }
}
