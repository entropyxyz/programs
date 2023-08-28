/// Points to the `template-barebones` program binary.
const BAREBONES_COMPONENT_WASM: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/template_barebones.wasm");

use ec_runtime::{ Runtime, InitialState };

#[test]
fn test_barebones_component() {
    let mut runtime = Runtime::new();

    // The barebones example simply validates that the length of the data to be signed is greater than 10.
    let longer_than_10 = "asdfasdfasdfasdf".to_string();
    let initial_state = InitialState {
        data: longer_than_10.into_bytes(),
    };

    let res = runtime.evaluate(BAREBONES_COMPONENT_WASM, &initial_state);
    assert!(res.is_ok());
}

#[test]
fn test_barebones_component_fails_with_data_length_less_than_10() {
    let mut runtime = Runtime::new();

    // Since the barebones example verifies that the length of the data to be signed is greater than 10, this should fail.
    let shorter_than_10 = "asdf".to_string();
    let initial_state = InitialState {
        data: shorter_than_10.into_bytes(),
    };

    let res = runtime.evaluate(BAREBONES_COMPONENT_WASM, &initial_state);
    assert!(res.is_err());
}