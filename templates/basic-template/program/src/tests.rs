use super::*;

#[test]
fn test_should_sign() {
    let signature_request = SignatureRequest {
        message: b"some_message".to_vec(),
        auxilary_data: None,
    };

    assert!({{project-name | upper_camel_case}}::evaluate(signature_request, None).is_ok());
}

#[test]
fn test_should_fail() {
    let signature_request = SignatureRequest {
        message: Vec::new(),
        auxilary_data: None,
    };

    assert!({{project-name | upper_camel_case}}::evaluate(signature_request, None).is_err());
}