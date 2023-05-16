//! This supports building new architectures and interfacing with them.

use serde::{Deserialize, Serialize};

use crate::Error;

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
