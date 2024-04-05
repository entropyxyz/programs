use super::*;

use ed25519_dalek::{Signature as Ed25519Signature, SigningKey as Ed25519Keypair};
use k256::ecdsa::{signature::Signer, Signature as EcdsaSignature, SigningKey as EcdsaKeypair};
use rand_core::OsRng;
use schnorrkel::{signing_context, Keypair as Sr25519Keypair, Signature as Sr25519Signature};

struct TestKeys {
    ecdsa_keys: Vec<EcdsaKeypair>,
    sr25519_keys: Vec<Sr25519Keypair>,
    ed25519_keys: Vec<Ed25519Keypair>,
}

#[test]
fn test_ok_for_only_device_key_signatures() {
    let device_keys = generate_test_keys();

    let config = Config {
        ecdsa_public_keys: device_keys
            .ecdsa_keys
            .iter()
            .map(|key| EcdsaPublicKey::from(key))
            .collect(),
        sr25519_public_keys: device_keys
            .sr25519_keys
            .iter()
            .map(|key| key.public)
            .collect(),
        ed25519_public_keys: device_keys
            .ed25519_keys
            .iter()
            .map(|key| key.verifying_key())
            .collect(),
    };
    let json_config = UserConfig::from(config.clone());

    let message: &str = "this is some message that we want to sign if its from a valid device key";

    // constrtuct signature request from device key (for positive test)
    let ecdsa_device_key_signature: EcdsaSignature = device_keys.ecdsa_keys[0]
        .try_sign(message.as_bytes())
        .unwrap();
    let device_key_aux_data_json_edcsa = AuxData {
        public_key_type: "ecdsa".to_string(),
        public_key: BASE64_STANDARD.encode(
            device_keys.ecdsa_keys[0]
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        ),
        signature: BASE64_STANDARD.encode(ecdsa_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    let mut request_from_device_key = SignatureRequest {
        message: message.to_string().into_bytes(),
        auxilary_data: Some(
            serde_json::to_string(&device_key_aux_data_json_edcsa)
                .unwrap()
                .into_bytes(),
        ),
    };

    let config_bytes = serde_json::to_vec(&json_config).unwrap();
    // positive for edcsa
    assert!(
        DeviceKeyProxy::evaluate(request_from_device_key.clone(), Some(config_bytes.clone()))
            .is_ok()
    );
    // positive for sr25519
    let context = signing_context(b"");

    let sr25519_device_key_signature: Sr25519Signature =
        device_keys.sr25519_keys[0].sign(context.bytes(message.as_bytes()));

    let device_key_aux_data_json_sr25519 = AuxData {
        public_key_type: "sr25519".to_string(),
        public_key: BASE64_STANDARD.encode(device_keys.sr25519_keys[0].public),
        signature: BASE64_STANDARD.encode(sr25519_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    request_from_device_key.auxilary_data = Some(
        serde_json::to_string(&device_key_aux_data_json_sr25519.clone())
            .unwrap()
            .into_bytes(),
    );
    assert!(
        DeviceKeyProxy::evaluate(request_from_device_key.clone(), Some(config_bytes.clone()))
            .is_ok()
    );
    // positive for ed25519
    let ed25519_device_key_signature: Ed25519Signature =
        device_keys.ed25519_keys[0].sign(message.as_bytes());
    let device_key_aux_data_json_ed25519 = AuxData {
        public_key_type: "ed25519".to_string(),
        public_key: BASE64_STANDARD.encode(device_keys.ed25519_keys[0].verifying_key()),
        signature: BASE64_STANDARD.encode(ed25519_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    request_from_device_key.auxilary_data = Some(
        serde_json::to_string(&device_key_aux_data_json_ed25519)
            .unwrap()
            .into_bytes(),
    );
    DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes.clone())).unwrap();
}

#[test]
fn test_fail_bad_signatures() {
    let device_keys = generate_test_keys();
    let non_device_keys = generate_test_keys();

    let config = Config {
        ecdsa_public_keys: device_keys
            .ecdsa_keys
            .iter()
            .map(|key| EcdsaPublicKey::from(key))
            .collect(),
        sr25519_public_keys: device_keys
            .sr25519_keys
            .iter()
            .map(|key| key.public)
            .collect(),
        ed25519_public_keys: device_keys
            .ed25519_keys
            .iter()
            .map(|key| key.verifying_key())
            .collect(),
    };
    let json_config = UserConfig::from(config.clone());

    let message: &str = "this is some message that we want to sign if its from a valid device key";
    let context = signing_context(b"");

    // constrtuct signature request from device key (for positive test)
    let ecdsa_non_device_key_signature: EcdsaSignature = non_device_keys.ecdsa_keys[0]
        .try_sign(message.as_bytes())
        .unwrap();

    let device_key_aux_data_json_edcsa = AuxData {
        public_key_type: "ecdsa".to_string(),
        public_key: BASE64_STANDARD.encode(
            device_keys.ecdsa_keys[0]
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        ),
        signature: BASE64_STANDARD.encode(ecdsa_non_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    let mut request_from_device_key = SignatureRequest {
        message: message.to_string().into_bytes(),
        auxilary_data: Some(
            serde_json::to_string(&device_key_aux_data_json_edcsa)
                .unwrap()
                .into_bytes(),
        ),
    };

    let config_bytes = serde_json::to_vec(&json_config).unwrap();
    // fail for edcsa
    assert_eq!(
        DeviceKeyProxy::evaluate(request_from_device_key.clone(), Some(config_bytes.clone()))
            .unwrap_err()
            .to_string(),
        "Error::InvalidSignatureRequest(\"Unable to verify ecdsa signature\")"
    );
    let sr25519_non_device_key_signature: Sr25519Signature =
        non_device_keys.sr25519_keys[0].sign(context.bytes(message.as_bytes()));
    // fail for sr25519
    let device_key_aux_data_json_sr25519 = AuxData {
        public_key_type: "sr25519".to_string(),
        public_key: BASE64_STANDARD.encode(device_keys.sr25519_keys[0].public),
        signature: BASE64_STANDARD.encode(sr25519_non_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    request_from_device_key.auxilary_data = Some(
        serde_json::to_string(&device_key_aux_data_json_sr25519.clone())
            .unwrap()
            .into_bytes(),
    );
    assert_eq!(
        DeviceKeyProxy::evaluate(request_from_device_key.clone(), Some(config_bytes.clone()))
            .unwrap_err()
            .to_string(),
        "Error::InvalidSignatureRequest(\"Unable to verify sr25519 signature\")"
    );
    // fail for ed25519
    let ed25519_non_device_key_signature: Ed25519Signature =
        non_device_keys.ed25519_keys[0].sign(message.as_bytes());
    let device_key_aux_data_json_ed25519 = AuxData {
        public_key_type: "ed25519".to_string(),
        public_key: BASE64_STANDARD.encode(device_keys.ed25519_keys[0].verifying_key()),
        signature: BASE64_STANDARD.encode(ed25519_non_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    request_from_device_key.auxilary_data = Some(
        serde_json::to_string(&device_key_aux_data_json_ed25519)
            .unwrap()
            .into_bytes(),
    );
    assert_eq!(
        DeviceKeyProxy::evaluate(request_from_device_key.clone(), Some(config_bytes.clone()))
            .unwrap_err()
            .to_string(),
        "Error::InvalidSignatureRequest(\"Unable to verify ed25519 signature\")"
    );
}

#[test]
fn test_fails_pub_key_not_found() {
    let device_keys = generate_test_keys();
    let non_device_keys = generate_test_keys();

    let config = Config {
        ecdsa_public_keys: device_keys
            .ecdsa_keys
            .iter()
            .map(|key| EcdsaPublicKey::from(key))
            .collect(),
        sr25519_public_keys: device_keys
            .sr25519_keys
            .iter()
            .map(|key| key.public)
            .collect(),
        ed25519_public_keys: device_keys
            .ed25519_keys
            .iter()
            .map(|key| key.verifying_key())
            .collect(),
    };
    let json_config = UserConfig::from(config.clone());
    let config_bytes = serde_json::to_vec(&json_config).unwrap();

    let message: &str = "this is some message that we want to sign if its from a valid device key";
    // construct signature request from non-device key (for negative test)
    let ecdsa_non_device_key_signature: EcdsaSignature = non_device_keys.ecdsa_keys[0]
        .try_sign(message.as_bytes())
        .unwrap();
    let non_device_key_aux_data_json = AuxData {
        public_key_type: "ecdsa".to_string(),
        public_key: BASE64_STANDARD.encode(
            non_device_keys.ecdsa_keys[0]
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        ),
        signature: BASE64_STANDARD.encode(ecdsa_non_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    let mut request_from_non_device_key = SignatureRequest {
        message: message.to_string().into_bytes(),
        auxilary_data: Some(
            serde_json::to_string(&non_device_key_aux_data_json)
                .unwrap()
                .into_bytes(),
        ),
    };
    assert_eq!(
        DeviceKeyProxy::evaluate(
            request_from_non_device_key.clone(),
            Some(config_bytes.clone())
        )
        .unwrap_err()
        .to_string(),
        "Error::InvalidSignatureRequest(\"ECDSA Public key not in config\")"
    );
    //sr25519 fail
    let context = signing_context(b"");

    let sr25519_device_key_signature: Sr25519Signature =
        non_device_keys.sr25519_keys[0].sign(context.bytes(message.as_bytes()));

    let non_device_key_aux_data_json_sr25519 = AuxData {
        public_key_type: "sr25519".to_string(),
        public_key: BASE64_STANDARD.encode(non_device_keys.sr25519_keys[0].public),
        signature: BASE64_STANDARD.encode(sr25519_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    request_from_non_device_key.auxilary_data = Some(
        serde_json::to_string(&non_device_key_aux_data_json_sr25519.clone())
            .unwrap()
            .into_bytes(),
    );
    assert_eq!(
        DeviceKeyProxy::evaluate(
            request_from_non_device_key.clone(),
            Some(config_bytes.clone())
        )
        .unwrap_err()
        .to_string(),
        "Error::InvalidSignatureRequest(\"Sr25519 Public key not in config\")"
    );

    //ed25519 fail
    let ed25519_device_key_signature: Ed25519Signature =
        non_device_keys.ed25519_keys[0].sign(message.as_bytes());
    let device_key_aux_data_json_ed25519 = AuxData {
        public_key_type: "ed25519".to_string(),
        public_key: BASE64_STANDARD.encode(non_device_keys.ed25519_keys[0].verifying_key()),
        signature: BASE64_STANDARD.encode(ed25519_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    request_from_non_device_key.auxilary_data = Some(
        serde_json::to_string(&device_key_aux_data_json_ed25519)
            .unwrap()
            .into_bytes(),
    );
    assert_eq!(
        DeviceKeyProxy::evaluate(request_from_non_device_key, Some(config_bytes))
            .unwrap_err()
            .to_string(),
        "Error::InvalidSignatureRequest(\"Ed25519 Public key not in config\")"
    );
}
#[test]
fn test_fails_with_no_aux_or_config() {
    let device_keys = generate_test_keys();

    let config = UserConfig {
        ecdsa_public_keys: Some(
            device_keys
                .ecdsa_keys
                .iter()
                .map(|key| {
                    let public_key = EcdsaPublicKey::from(key);
                    let encoded_key =
                        BASE64_STANDARD.encode(public_key.to_encoded_point(true).as_bytes());
                    encoded_key
                })
                .collect(),
        ),
        sr25519_public_keys: None,
        ed25519_public_keys: None,
    };
    let config_bytes = serde_json::to_vec(&config).unwrap();

    let message = "this is some message that we want to sign if its from a valid device key";
    let _device_key_signature: EcdsaSignature = device_keys.ecdsa_keys[0]
        .try_sign(message.as_bytes())
        .unwrap();

    let request_from_device_key_no_aux = SignatureRequest {
        message: message.to_string().into_bytes(),
        auxilary_data: None,
    };

    assert_eq!(
        DeviceKeyProxy::evaluate(
            request_from_device_key_no_aux.clone(),
            Some(config_bytes.clone())
        )
        .unwrap_err()
        .to_string(),
        "Error::InvalidSignatureRequest(\"No auxilary_data provided\")"
    );

    let ecdsa_device_key_signature: EcdsaSignature = device_keys.ecdsa_keys[0]
        .try_sign(message.as_bytes())
        .unwrap();

    let mut device_key_aux_data_json = AuxData {
        public_key_type: "ecdsa".to_string(),
        public_key: BASE64_STANDARD.encode(
            device_keys.ecdsa_keys[0]
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        ),
        signature: BASE64_STANDARD.encode(ecdsa_device_key_signature.to_bytes()),
        context: "".to_string(),
    };
    let mut request_from_device_key = SignatureRequest {
        message: message.to_string().into_bytes(),
        auxilary_data: Some(
            serde_json::to_string(&device_key_aux_data_json)
                .unwrap()
                .into_bytes(),
        ),
    };
    assert_eq!(
        DeviceKeyProxy::evaluate(request_from_device_key.clone(), None)
            .unwrap_err()
            .to_string(),
        "Error::Evaluation(\"No config provided.\")"
    );

    device_key_aux_data_json.public_key_type = "phish".to_string();
    request_from_device_key.auxilary_data = Some(
        serde_json::to_string(&device_key_aux_data_json)
            .unwrap()
            .into_bytes(),
    );
    assert_eq!(
        DeviceKeyProxy::evaluate(request_from_device_key, Some(config_bytes.clone()))
            .unwrap_err()
            .to_string(),
        "Error::InvalidSignatureRequest(\"Invalid public key type\")"
    );
}

/// Generates keys that can be used for testing
fn generate_test_keys() -> TestKeys {
    let ecdsa_keys: Vec<EcdsaKeypair> = (0..3).map(|_| EcdsaKeypair::random(&mut OsRng)).collect();
    let sr25519_keys: Vec<Sr25519Keypair> = (0..3)
        .map(|_| Sr25519Keypair::generate_with(&mut OsRng))
        .collect();
    let ed25519_keys: Vec<Ed25519Keypair> = (0..3)
        .map(|_| Ed25519Keypair::generate(&mut OsRng))
        .collect();

    TestKeys {
        ecdsa_keys,
        sr25519_keys,
        ed25519_keys,
    }
}
