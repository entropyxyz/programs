//! Contains the traits and implementations of each supported constraint.

use crate::{Architecture, Error};

/// Any constraint must implement this for each architecture.
pub trait Evaluate<A: Architecture> {
    fn eval(self, tx: A::TransactionRequest) -> Result<(), Error>;
}
