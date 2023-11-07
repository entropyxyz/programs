/// Points to the `template-barebones` program binary.
const BAREBONES_COMPONENT_WASM: &[u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/template_barebones.wasm");

use ec_runtime::{Runtime, SignatureRequest};

#[test]
fn test_barebones_component() {
    let mut runtime = Runtime::new();

    // The barebones example simply validates that the length of the data to be signed is greater than 10.
    let longer_than_10 = "asdfasdfasdfasdf".to_string();
    let signature_request = SignatureRequest {
        message: longer_than_10.into_bytes(),
        auxilary_data: None,
    };

    let res = runtime.evaluate(BAREBONES_COMPONENT_WASM, &signature_request);
    assert!(res.is_ok());
}

#[test]
fn test_barebones_component_fails_with_data_length_less_than_10() {
    let mut runtime = Runtime::new();

    // Since the barebones example verifies that the length of the data to be signed is greater than 10, this should fail.
    let shorter_than_10 = "asdf".to_string();
    let signature_request = SignatureRequest {
        message: shorter_than_10.into_bytes(),
        auxilary_data: None,
    };

    let res = runtime.evaluate(BAREBONES_COMPONENT_WASM, &signature_request);
    assert!(res.is_err());
}
