#![no_std]

extern crate alloc;

use alloc::{borrow::ToOwned, format, string::{String, ToString}, vec::Vec};

use entropy_programs_core::{bindgen::*, export_program, prelude::*};

use serde_json;
use serde::{Serialize, Deserialize};
use k256::ecdsa::{VerifyingKey, Signature};
use base64::{prelude::BASE64_STANDARD, Engine};
use k256::ecdsa::signature::Verifier;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct DeviceKeyProxyConfig {
    // base64-encoded device keys
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

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sign() {
        let signature_request = SignatureRequest {
            message: "some_data_longer_than_10_bytes".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(DeviceKeyProxy::evaluate(signature_request, None).is_ok());
    }

    #[test]
    fn test_should_error() {
        // data being checked is under 10 bytes in length
        let signature_request = SignatureRequest {
            message: "under10".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(DeviceKeyProxy::evaluate(signature_request, None).is_err());
    }
}
