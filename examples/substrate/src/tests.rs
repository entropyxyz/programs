use super::*;
use alloc::{
    string::{ToString},
    vec,
};
use entropy_programs_substrate::{get_offline_api, handle_encoding};
use subxt::config::PolkadotExtrinsicParamsBuilder as Params;
use subxt::dynamic::tx;

#[test]
fn test_should_sign() {
    let aux_data = create_aux_data();

    let api = get_offline_api(
        aux_data.genesis_hash.clone(),
        aux_data.spec_version,
        aux_data.transaction_version,
    )
    .unwrap();

    let deserialized: Vec<(&str, &str)> = serde_json::from_str(&aux_data.values).unwrap();
    let encoding = handle_encoding(deserialized).unwrap();

    let balance_transfer_tx = tx(aux_data.pallet.clone(), aux_data.function.clone(), encoding);

    let tx_params = Params::new().build();

    let partial = api
        .tx()
        .create_partial_signed_offline(&balance_transfer_tx, tx_params)
        .unwrap()
        .signer_payload();

    let signature_request = SignatureRequest {
        message: partial.to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };

    assert!(SubstrateProgram::evaluate(signature_request, None, None).is_ok());
}

#[test]
fn test_should_fail() {
    let aux_data = create_aux_data();

    let api = get_offline_api(
        aux_data.genesis_hash.clone(),
        aux_data.spec_version,
        aux_data.transaction_version,
    )
    .unwrap();

    let amount = 1000u128;
    let binding = amount.to_string();
    let string_account_id = "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu";

    let values: Vec<(&str, &str)> = vec![("account", string_account_id), ("amount", &binding)];

    let encoding = handle_encoding(values.clone()).unwrap();
    let balance_transfer_tx = tx(
        aux_data.pallet.clone(),
        aux_data.function.clone(),
        encoding.clone(),
    );

    let tx_params = Params::new().build();

    let partial = api
        .tx()
        .create_partial_signed_offline(&balance_transfer_tx, tx_params)
        .unwrap()
        .signer_payload();

    let signature_request = SignatureRequest {
        message: partial.to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };

    assert_eq!(
        SubstrateProgram::evaluate(signature_request, None, None)
            .unwrap_err()
            .to_string(),
            "Error::InvalidSignatureRequest(\"Error comparing tx request and message: Error::Evaluation(\\\"Signatures don't match, message: \\\\\\\"07000088dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0eea10f0000000a0000000a00000044670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a\\\\\\\", calldata: \\\\\\\"07000088dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee9101\\\\\\\", genesis_hash: \\\\\\\"34343637306136383137373832316136313636623235663864383662343565306631633362323830666635373665656136343035376534623064643966663461\\\\\\\"\\\")\")"
    );
}

pub fn create_aux_data() -> AuxData {
    let genesis_hash =
        "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a".to_string();
    let spec_version = 10;
    let transaction_version = 10;
    let string_account_id = "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu";
    let amount = 100u128;
    let binding = amount.to_string();
    let values: Vec<(&str, &str)> = vec![("account", string_account_id), ("amount", &binding)];

    let aux_data = AuxData {
        genesis_hash,
        spec_version,
        transaction_version,
        pallet: "Balances".to_string(),
        function: "transfer_allow_death".to_string(),
        values: serde_json::to_string(&values).unwrap(),
    };
    aux_data
}
