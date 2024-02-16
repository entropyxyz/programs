//! This supports core traits and types for supporting new architectures and programs, and interfacing with them.

/// See the [`wit-bindgen` Rust guest example](https://github.com/bytecodealliance/wit-bindgen#guest-rust) for information on how to use this.
pub mod bindgen {
    wit_bindgen::generate!({
        world: "program",
        path: "../wit/application.wit",
        macro_export
    });
}

pub use bindgen::Error;

pub mod programs;

pub use architecture::*;
pub use programs::*;

/// Each transaction-like architecture should implement these.
pub mod architecture {
    use super::bindgen::Error;
    use serde::{Deserialize, Serialize};

    /// Trait for defining important types associated with an architecture.
    pub trait Architecture: Serialize + for<'de> Deserialize<'de> {
        /// Account type for that chain(SS58, H160, etc)
        type Address: Eq + Serialize + for<'de> Deserialize<'de> + From<Self::AddressRaw>;
        /// Account type as it is stored in the database
        type AddressRaw: Eq + Serialize + for<'de> Deserialize<'de> + From<Self::Address>;
        /// Transaction request type for unsigned transactions
        type TransactionRequest: GetSender<Self>
            + GetReceiver<Self>
            + Serialize
            + for<'de> Deserialize<'de>
            + Parse<Self>
            + TryParse<Self>;
    }

    /// Trait for getting the the sender of a transaction.
    pub trait GetSender<A: Architecture> {
        fn sender(&self) -> Option<A::Address>;
    }

    /// Trait for getting the the receiver of a transaction.
    pub trait GetReceiver<A: Architecture> {
        fn receiver(&self) -> Option<A::Address>;
    }

    /// DEPRECATED: Use `TryParse`
    ///
    /// Trait for parsing a raw transaction request into its native transaction request struct.
    pub trait Parse<A: Architecture> {
        fn parse(raw_tx: String) -> Result<A::TransactionRequest, Error>;
    }

    /// Tries to parse a raw transaction request into its native transaction request struct.
    pub trait TryParse<A: Architecture> {
        fn try_parse(raw_tx: &[u8]) -> Result<A::TransactionRequest, Error>;
    }
}

/// Includes items that should be imported into most scopes
pub mod prelude {
    // reexport getrandom custom handler (move to macro)
    pub use getrandom::register_custom_getrandom;
    // reexport all core traits
    pub use super::architecture::*;

    use core::num::NonZeroU32;
    use getrandom::Error;

    /// Custom `getrandom()` handler that always returns an error.
    ///
    /// `getrandom` is a commonly used package for sourcing randomness.This should return an error for now,
    /// but in the future it might make sense for the validators to determinstically source randomness from
    /// BABE (eg. at a certain block)
    ///
    /// From https://docs.rs/getrandom/latest/getrandom/macro.register_custom_getrandom.html
    // TODO This should get throw into the macros
    pub fn always_fail(_buf: &mut [u8]) -> Result<(), Error> {
        let code = NonZeroU32::new(Error::CUSTOM_START.saturating_add(1)).unwrap();
        Err(Error::from(code))
    }
}
