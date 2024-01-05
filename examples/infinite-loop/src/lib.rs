#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct InfiniteLoop;

impl Program for InfiniteLoop {
    /// This is the only function required by the program runtime. `message` is the preimage of the curve element to be
    /// signed, eg. RLP-serialized Ethereum transaction request, raw x86_64 executable, etc.
    fn evaluate(_signature_request: SignatureRequest) -> Result<(), Error> {
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
