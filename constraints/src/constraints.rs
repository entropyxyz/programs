//! Contains the traits and implementations of each supported constraint.

use ec_core::{Acl, AclKind};
use ethers_core::types::{NameOrAddress, H160};

use crate::{Architecture, Error, Evm};

/// Constraints must implement an evaluation trait that parses.
pub trait Evaluate<A: Architecture> {
    fn eval(self, tx: A::TransactionRequest) -> Result<(), Error>;
}

// TODO This can likely be made generic over any architecture with GetRecipient and GetSender traits
#[allow(clippy::needless_collect)]
impl Evaluate<Evm> for Acl<[u8; 20]> {
    fn eval(self, tx: <Evm as Architecture>::TransactionRequest) -> Result<(), Error> {
        if tx.to.is_none() {
            return match self.allow_null_recipient {
                true => Ok(()),
                false => Err(Error::Evaluation("Null recipients are not allowed.")),
            };
        }

        let converted_addresses: Vec<NameOrAddress> =
            self.addresses.into_iter().map(|a| NameOrAddress::Address(H160::from(a))).collect();

        match (converted_addresses.contains(&tx.to.unwrap()), self.kind) {
            (true, AclKind::Allow) => Ok(()),
            (false, AclKind::Deny) => Ok(()),
            _ => Err(Error::Evaluation("Transaction not allowed.")),
        }
    }
}
