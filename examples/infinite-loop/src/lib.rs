#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

use serde::{Deserialize, Serialize};

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {}

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct InfiniteLoop;

impl Program for InfiniteLoop {
    /// This is the only function required by the program runtime. `message` is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(
        _signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<Vec<u8>>>,
    ) -> Result<(), Error> {
        loop {}
        #[allow(unreachable_code)]
        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(InfiniteLoop);
