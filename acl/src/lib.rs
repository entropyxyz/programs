extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;

use codec::MaxEncodedLen;
use codec::{Decode, Encode};
pub use ec_core::{Architecture, Error as CoreError, Evaluate};

#[cfg(feature = "evm")]
use ec_evm::{Evm, NameOrAddress, H160};

use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// An access control list (Allow/Deny lists).
#[derive(
    Clone,
    Debug,
    Encode,
    Decode,
    PartialEq,
    Eq,
    scale_info::TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub struct Acl<Address> {
    pub addresses: Vec<Address>,
    pub kind: AclKind,
    pub allow_null_recipient: bool,
}

/// Represents either an allow or deny list.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
    Serialize,
    Deserialize,
)]
pub enum AclKind {
    Allow,
    Deny,
}

/// Creates an empty ACL that always evaluates to false.
impl<A: Default> Default for Acl<A> {
    fn default() -> Self {
        let addresses = Vec::<A>::default();
        Self {
            addresses,
            kind: AclKind::Allow,
            allow_null_recipient: false,
        }
    }
}

// TODO This can likely be made generic over any architecture with GetRecipient and GetSender traits

#[allow(clippy::needless_collect)]
#[cfg(feature = "evm")]
impl Evaluate<Evm> for Acl<[u8; 20]> {
    fn eval(self, tx: <Evm as Architecture>::TransactionRequest) -> Result<(), CoreError> {
        if tx.to.is_none() {
            return match self.allow_null_recipient {
                true => Ok(()),
                false => Err(CoreError::Evaluation("Null recipients are not allowed.")),
            };
        }

        let converted_addresses: Vec<NameOrAddress> = self
            .addresses
            .into_iter()
            .map(|a| NameOrAddress::Address(H160::from(a)))
            .collect();

        match (converted_addresses.contains(&tx.to.unwrap()), self.kind) {
            (true, AclKind::Allow) => Ok(()),
            (false, AclKind::Deny) => Ok(()),
            _ => Err(CoreError::Evaluation("Transaction not allowed.")),
        }
    }
}
