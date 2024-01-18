extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;

use codec::MaxEncodedLen;
use codec::{Decode, Encode};
pub use entropy_programs_core::{Architecture, Error as CoreError, SatisfiableForArchitecture};

#[cfg(feature = "evm")]
pub use entropy_programs_evm::{Evm, NameOrAddress, H160};

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

// TODO This needs to be generic over any architecture (use GetRecipient and GetSender traits)
// TODO Move to `entropy-programs-evm` crate?
#[allow(clippy::needless_collect)]
#[cfg(feature = "evm")]
impl SatisfiableForArchitecture<Evm> for Acl<<Evm as Architecture>::AddressRaw> {
    fn is_satisfied_by(
        self,
        tx: &<Evm as Architecture>::TransactionRequest,
    ) -> Result<(), CoreError> {
        if tx.to.is_none() {
            return match self.allow_null_recipient {
                true => Ok(()),
                false => Err(CoreError::Evaluation(
                    "Null recipients are not allowed.".to_string(),
                )),
            };
        }

        let converted_addresses: Vec<NameOrAddress> = self
            .addresses
            .into_iter()
            .map(|a| NameOrAddress::Address(H160::from(a)))
            .collect();

        match (
            converted_addresses.contains(&tx.to.clone().unwrap()),
            self.kind,
        ) {
            (true, AclKind::Allow) => Ok(()),
            (false, AclKind::Deny) => Ok(()),
            _ => Err(CoreError::Evaluation(
                "Transaction not allowed.".to_string(),
            )),
        }
    }
}
