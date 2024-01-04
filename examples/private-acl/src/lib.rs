// Copyright (C) 2023 Entropy Cryptography Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! This is an allow list where the allowed addresses are hashed to improve privacy
//! It is still possible for anyone to check whether a given address is in the list, using the
//! on-chain bytecode. But you cannot just read the allowed addresses from it.
#![no_std]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use blake2::{Blake2s256, Digest};
use ec_constraints::{
    arch::evm::NameOrAddress,
    constraints::acl::*,
    core::{bindgen::*, export_program, prelude::*, TryParse},
};

pub struct PrivateTransactionAcl;

include!(concat!(env!("OUT_DIR"), "/addresses.rs"));

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

impl Program for PrivateTransactionAcl {
    /// Allow any address given in the pre-defined list (addresses.txt)
    // #[no_mangle]
    fn evaluate(signature_request: SignatureRequest) -> Result<(), CoreError> {
        // parse the raw tx into some type
        let parsed_tx =
            <Evm as Architecture>::TransactionRequest::try_parse(signature_request.message.as_slice())?;

        let name_or_address: NameOrAddress = parsed_tx.to.ok_or(Error::Evaluation(
            "No recipient given in transaction".to_string(),
        ))?;

        match name_or_address {
            NameOrAddress::Name(_) => Err(Error::Evaluation("ENS names not supported".to_string())),
            NameOrAddress::Address(address) => {
                let hashed_address = {
                    let mut hasher = Blake2s256::new();
                    hasher.update(&address.0);
                    hasher.finalize().into()
                };
                if ADDRESSES.contains(&hashed_address) {
                    Ok(())
                } else {
                    Err(Error::Evaluation("Address not in allow list".to_string()))
                }
            }
        }
    }

    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(PrivateTransactionAcl);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_evaluate() {
        let signature_request = SignatureRequest {
            // `data` is an RLP serialized ETH transaction with the recipient set to `0x772b9a9e8aa1c9db861c6611a82d251db4fac990`
            message: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(PrivateTransactionAcl::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_start_fail() {
        let signature_request = SignatureRequest {
            // `data` is the same as previous test, but recipient address ends in `1` instead of `0`, so it should fail
            message: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac991019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
            auxilary_data: None,
        };

        assert!(PrivateTransactionAcl::evaluate(signature_request).is_err());
    }
