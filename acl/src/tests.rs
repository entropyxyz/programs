#![cfg(test)]

use ec_core::{Acl, AclKind};
use ethers_core::types::{NameOrAddress, TransactionRequest, H160};

use crate::Evaluate;

#[test]
fn test_acl_functions_properly() {
    let evm_address_1: [u8; 20] = [1u8; 20];
    let evm_address_2: [u8; 20] = [2u8; 20];
    let evm_address_3: [u8; 20] = [3u8; 20];

    let to_address_1_tx = TransactionRequest {
        to: Some(NameOrAddress::Address(H160::from(evm_address_1))),
        ..Default::default()
    };
    let to_address_2_tx = TransactionRequest {
        to: Some(NameOrAddress::Address(H160::from(evm_address_2))),
        ..Default::default()
    };
    let to_address_3_tx = TransactionRequest {
        to: Some(NameOrAddress::Address(H160::from(evm_address_3))),
        ..Default::default()
    };
    let to_null_recipient_tx = TransactionRequest {
        to: None,
        ..Default::default()
    };

    let allowlisted_acl = Acl::<[u8; 20]> {
        addresses: vec![evm_address_1],
        ..Default::default()
    };

    // should only let allowlisted_tx through
    assert!(allowlisted_acl
        .clone()
        .is_satisfied_by(to_address_1_tx.clone())
        .is_ok());

    assert!(allowlisted_acl
        .clone()
        .is_satisfied_by(to_address_2_tx.clone())
        .is_err());
    assert!(allowlisted_acl
        .clone()
        .is_satisfied_by(to_address_3_tx.clone())
        .is_err());
    assert!(allowlisted_acl
        .clone()
        .is_satisfied_by(to_null_recipient_tx.clone())
        .is_err());

    let denylisted_acl = Acl::<[u8; 20]> {
        addresses: vec![evm_address_1],
        kind: AclKind::Deny,
        ..Default::default()
    };

    // should only block whitelisted and null recipient txs
    assert!(denylisted_acl.clone().is_satisfied_by(to_address_2_tx.clone()).is_ok());
    assert!(denylisted_acl.clone().is_satisfied_by(to_address_3_tx.clone()).is_ok());

    assert!(denylisted_acl.is_satisfied_by(to_address_1_tx.clone()).is_err());
    assert!(allowlisted_acl.is_satisfied_by(to_null_recipient_tx.clone()).is_err());

    let allowlisted_acl_with_null_recipient = Acl::<[u8; 20]> {
        addresses: vec![evm_address_1],
        allow_null_recipient: true,
        ..Default::default()
    };

    // should only let allowlisted_tx and null recipient txs through
    assert!(allowlisted_acl_with_null_recipient
        .clone()
        .is_satisfied_by(to_address_1_tx.clone())
        .is_ok());
    assert!(allowlisted_acl_with_null_recipient
        .clone()
        .is_satisfied_by(to_null_recipient_tx.clone())
        .is_ok());

    assert!(allowlisted_acl_with_null_recipient
        .clone()
        .is_satisfied_by(to_address_2_tx.clone())
        .is_err());
    assert!(allowlisted_acl_with_null_recipient
        .is_satisfied_by(to_address_3_tx.clone())
        .is_err());

    let denylisted_acl_with_null_recipient = Acl::<[u8; 20]> {
        addresses: vec![evm_address_1],
        kind: AclKind::Deny,
        allow_null_recipient: true,
    };

    // should only block whitelisted
    assert!(denylisted_acl_with_null_recipient
        .clone()
        .is_satisfied_by(to_address_2_tx.clone())
        .is_ok());
    assert!(denylisted_acl_with_null_recipient
        .clone()
        .is_satisfied_by(to_address_3_tx.clone())
        .is_ok());
    assert!(denylisted_acl_with_null_recipient
        .clone()
        .is_satisfied_by(to_null_recipient_tx.clone())
        .is_ok());

    assert!(denylisted_acl_with_null_recipient
        .is_satisfied_by(to_address_1_tx.clone())
        .is_err());

    let empty_acl = Acl::<[u8; 20]>::default();

    // should fail all txs
    assert!(empty_acl.clone().is_satisfied_by(to_address_1_tx).is_err());
    assert!(empty_acl.clone().is_satisfied_by(to_address_2_tx).is_err());
    assert!(empty_acl.clone().is_satisfied_by(to_address_3_tx).is_err());
    assert!(empty_acl.is_satisfied_by(to_null_recipient_tx).is_err());
}
