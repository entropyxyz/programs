//! This module contains the EVM architecture and its associated types. Since it implements Architecture, constraints written around the Architecture trait can be used with EVM.

extern crate alloc;

use alloc::string::String;

use ec_core::{Architecture, Error as CoreError, GetReceiver, GetSender, Parse, TryParse};
pub use ethers_core::types::transaction::request::TransactionRequest as EvmTransactionRequest;
pub use ethers_core::types::{NameOrAddress, H160};
use rlp::Rlp;
use serde::{Deserialize, Serialize};

/// EVM Architecture
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evm;

impl Architecture for Evm {
    type Address = H160;
    type AddressRaw = [u8; 20];
    type TransactionRequest = EvmTransactionRequest;
}

impl GetSender<Evm> for <Evm as Architecture>::TransactionRequest {
    fn sender(&self) -> Option<<Evm as Architecture>::Address> {
        self.from
    }
}

impl GetReceiver<Evm> for <Evm as Architecture>::TransactionRequest {
    fn receiver(&self) -> Option<<Evm as Architecture>::Address> {
        match &self.to {
            Some(to) => match to {
                NameOrAddress::Address(addr) => Some(addr.to_owned()),
                // This should never get returned because we Error on ENS names in the `parse`
                // function
                NameOrAddress::Name(_) => None,
            },
            None => None,
        }
    }
}

impl Parse<Evm> for <Evm as Architecture>::TransactionRequest {
    fn parse(
        hex_rlp_raw_tx: String,
    ) -> Result<<Evm as Architecture>::TransactionRequest, CoreError> {
        let bytes = hex::decode(hex_rlp_raw_tx.replace("0x", "")).map_err(|e| {
            CoreError::InvalidSignatureRequest(format!("Unable to parse to RLP: {}", e))
        })?;
        let rlp = Rlp::new(&bytes);
        match Self::decode_unsigned_rlp(&rlp) {
            Ok(tx) => match tx.to {
                // Clients shouldn't even be able to serialize tx reqs with ENS names, but it it
                // does somehow, err
                Some(NameOrAddress::Name(_)) => Err(CoreError::InvalidSignatureRequest(
                    "ENS recipients not supported. Resolve to an address first.".to_string(),
                )),
                _ => Ok(tx),
            },
            Err(e) => Err(CoreError::InvalidSignatureRequest(format!(
                "Unable to decode string: {}",
                e
            ))),
        }
    }
}

impl TryParse<Evm> for <Evm as Architecture>::TransactionRequest {
    /// TODO expect the hex-encoded RLP of the transaction request, so user doesn't have to hex::decode
    fn try_parse(bytes: &[u8]) -> Result<Self, CoreError> {
        let request_as_string = String::from_utf8(bytes.to_owned()).map_err(|e| {
            CoreError::InvalidSignatureRequest(format!("Unable to parse to String: {}", e))
        })?;
        let into_bytes = hex::decode(request_as_string.replace("0x", "")).map_err(|e| {
            CoreError::InvalidSignatureRequest(format!("Unable to parse to RLP: {}", e))
        })?;
        let rlp = Rlp::new(&into_bytes);

        match Self::decode_unsigned_rlp(&rlp) {
            Ok(tx) => match tx.to {
                // Clients shouldn't even be able to serialize tx reqs with ENS names, but it it
                // does somehow, err
                Some(NameOrAddress::Name(_)) => Err(CoreError::InvalidSignatureRequest(
                    "ENS recipients not supported. Resolve to an address first.".to_string(),
                )),
                _ => Ok(tx),
            },
            Err(e) => Err(CoreError::InvalidSignatureRequest(format!(
                "Unable to decode string: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers_core::types::{Address as EvmAddress, H256};

    use super::*;

    #[test]
    fn can_parse_evm_rlp_transactions() {
        // This is `serializedUnsignedTx` from entropy-js threshold-server tests
        let raw_unsigned_tx = "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string();
        let unsigned_tx = EvmTransactionRequest::parse(raw_unsigned_tx).unwrap();
        assert_eq!(unsigned_tx.sender(), None);
        assert_eq!(
            unsigned_tx.receiver(),
            Some(EvmAddress::from_str("772b9a9e8aa1c9db861c6611a82d251db4fac990").unwrap())
        ); // manually removed the 0x
    }

    /// Tests that the parsed transaction's sighash matches the client's sighash
    #[test]
    fn evm_parsed_sighash_matches_clients_sighash() {
        // These are from from entropy-js threshold-server tests
        let raw_unsigned_tx = "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string();
        let known_expected_sighash: H256 = H256::from_slice(
            hex::decode(
                "0xe62e139a15f27f3d5ba043756aaca2b6fe9597a95973befa36dbe6095ee16da2"
                    .replace("0x", ""),
            )
            .unwrap()
            .as_slice(),
        );

        let unsigned_tx = EvmTransactionRequest::parse(raw_unsigned_tx).unwrap();
        assert_eq!(unsigned_tx.sighash(), known_expected_sighash);
    }

    #[test]
    fn throws_error_parsing_malformed_evm_rlp() {
        let random_bytes =
            "0x1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080"
                .to_string();

        let unsigned_tx = EvmTransactionRequest::parse(random_bytes);
        assert!(unsigned_tx.is_err());
    }
}
