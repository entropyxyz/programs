/// Points to the `template-barebones` program binary.
const BAREBONES_COMPONENT_WASM: &[u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/template_barebones.wasm");
const CUSTOM_HASH_COMPONENT_WASM: &[u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/example_custom_hash.wasm");
/// Points to the `infinite-loop` program binary.
const INFINITE_LOOP_WASM: &[u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/infinite_loop.wasm");

use blake3;
use entropy_programs_runtime::{Runtime, SignatureRequest};

#[test]
fn test_barebones_component() {
    let mut runtime = Runtime::default();

    // The barebones example simply validates that the length of the data to be signed is greater than 10.
    let longer_than_10 = "asdfasdfasdfasdf".to_string();
    let signature_request = SignatureRequest {
        message: longer_than_10.into_bytes(),
        auxilary_data: None,
    };

    let res = runtime.evaluate(BAREBONES_COMPONENT_WASM, &signature_request, None, None);
    assert!(res.is_ok());
}

#[test]
fn test_barebones_component_fails_with_data_length_less_than_10() {
    let mut runtime = Runtime::default();

    // Since the barebones example verifies that the length of the data to be signed is greater than 10, this should fail.
    let shorter_than_10 = "asdf".to_string();
    let signature_request = SignatureRequest {
        message: shorter_than_10.into_bytes(),
        auxilary_data: None,
    };

    let res = runtime.evaluate(BAREBONES_COMPONENT_WASM, &signature_request, None, None);
    assert!(res.is_err());
}

#[test]
fn test_empty_bytecode_fails() {
    let mut runtime = Runtime::default();

    let signature_request = SignatureRequest {
        message: vec![],
        auxilary_data: None,
    };

    let res = runtime.evaluate(&[], &signature_request, None, None);
    assert_eq!(res.unwrap_err().to_string(), "Bytecode length is zero");
}

#[test]
fn test_infinite_loop() {
    let mut runtime = Runtime::default();

    let signature_request = SignatureRequest {
        message: vec![],
        auxilary_data: None,
    };

    let res = runtime.evaluate(INFINITE_LOOP_WASM, &signature_request, None, None);
    assert_eq!(res.unwrap_err().to_string(), "Out of fuel");
}

#[test]
fn test_custom_hash() {
    let mut runtime = Runtime::default();

    let message = "some_data_to_be_hashed".to_string().into_bytes();

    let mut expected_hash = [0u8; 32];
    let expected_hash_as_vec = blake3::hash(&message).as_bytes().to_vec();
    expected_hash.copy_from_slice(&expected_hash_as_vec);

    let actual_hash = runtime
        .custom_hash(CUSTOM_HASH_COMPONENT_WASM, message.as_slice())
        .unwrap();

    assert_eq!(actual_hash, expected_hash);
}

#[test]
fn test_custom_hash_errors_when_returning_none() {
    let mut runtime = Runtime::default();

    let message = "some_data_to_be_hashed".to_string().into_bytes();

    let res = runtime.custom_hash(
        // Remember, barebones component doesn't define a custom hash function
        BAREBONES_COMPONENT_WASM,
        message.as_slice(),
    );
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Runtime error: Error::InvalidSignatureRequest(\"`custom-hash` returns `None`. Implement the hash function in your program, or select a predefined `hash` in your signature request.\")"
    );
}

// TODO add test for custom hash returning a vec of length != 32
