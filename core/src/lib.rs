use thiserror::Error;

pub mod architectures;
pub mod constraints;

pub use architectures::*;
pub use constraints::*;

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
