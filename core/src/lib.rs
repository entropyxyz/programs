//! This supports core traits and types for supporting new architectures and constraints, and interfacing with them.

pub use architecture::*;
pub use constraints::*;

use thiserror::Error;

/// Errors related to parsing and evaulating constraints.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Transaction request could not be parsed
    #[error("Invalid transaction request: {0}")]
    InvalidTransactionRequest(String),
    /// Transaction request did not meet constraint requirements.
    #[error("Constraint Evaluation error: {0}")]
    Evaluation(&'static str),
}

/// Contains the traits and implementations of each supported constraint.
mod architecture {
    use super::Error;
    use serde::{Deserialize, Serialize};
    /// Trait for defining important types associated with an architecture.
    pub trait Architecture: Serialize + for<'de> Deserialize<'de> {
        /// Account type for that chain(SS58, H160, etc)
        type Address: Eq + Serialize + for<'de> Deserialize<'de>;
        /// Account type as it is stored in the database
        type AddressRaw: Eq + Serialize + for<'de> Deserialize<'de>;
        /// Transaction request type for unsigned transactions
        type TransactionRequest: GetSender<Self>
            + GetReceiver<Self>
            + Serialize
            + for<'de> Deserialize<'de>
            + Parse<Self>;
    }

    /// Trait for getting the the sender of a transaction.
    pub trait GetSender<A: Architecture> {
        fn sender(&self) -> Option<A::Address>;
    }

    /// Trait for getting the the receiver of a transaction.
    pub trait GetReceiver<A: Architecture> {
        fn receiver(&self) -> Option<A::Address>;
    }

    /// Trait for parsing a raw transaction request into its native transaction request struct.
    pub trait Parse<A: Architecture> {
        fn parse(raw_tx: String) -> Result<A::TransactionRequest, Error>;
    }
}

/// Constraint-specific traits
mod constraints {
    use super::architecture::Architecture;
    use super::Error;

    /// Any constraint must implement this for each architecture it wants to support.
    pub trait Evaluate<A: Architecture> {
        fn eval(self, tx: A::TransactionRequest) -> Result<(), Error>;
    }
}
