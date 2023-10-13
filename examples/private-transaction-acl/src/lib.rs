//! This is an allow list where the allowed addresses are hashed to improve privacy
//! It is still possible for anyone to check whether a given address is in the list, using the
//! on-chain bytecode. But you cannot just read the allowed addresses from it.
#![no_std]

extern crate alloc;

use alloc::string::ToString;
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
    fn evaluate(state: InitialState) -> Result<(), CoreError> {
        // parse the raw tx into some type
        let parsed_tx =
            <Evm as Architecture>::TransactionRequest::try_parse(state.data.as_slice())?;

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
}

export_program!(PrivateTransactionAcl);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_evaluate() {
        let signature_request = InitialState {
            // `data` is an RLP serialized ETH transaction with the recipient set to `0x772b9a9e8aa1c9db861c6611a82d251db4fac990`
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac990019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };

        assert!(PrivateTransactionAcl::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_start_fail() {
        let signature_request = InitialState {
            // `data` is the same as previous test, but recipient address ends in `1` instead of `0`, so it should fail
            data: "0xef01808094772b9a9e8aa1c9db861c6611a82d251db4fac991019243726561746564204f6e20456e74726f7079018080".to_string().into_bytes(),
        };

        assert!(PrivateTransactionAcl::evaluate(signature_request).is_err());
    }
}
