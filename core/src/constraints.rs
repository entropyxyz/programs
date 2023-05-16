//! Contains the traits and implementations of each supported constraint.

use crate::{Architecture, Error};

/// Constraints must implement an evaluation trait that parses.
pub trait Evaluate<A: Architecture> {
    fn eval(self, tx: A::TransactionRequest) -> Result<(), Error>;
}
