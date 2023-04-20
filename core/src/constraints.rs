pub use acl::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::{fmt::Debug, vec::Vec};

/// Supported architectures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Serialize, Deserialize, TypeInfo)]
pub enum Arch {
    Evm,
    /// Not yet supported on the client, supported in Substrate storage/ACL
    Btc,
}

/// Represents a user's constraints
#[derive(Default, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct Constraints {
    pub evm_acl: Option<Acl<[u8; 20]>>,
    pub btc_acl: Option<Acl<[u8; 32]>>,
}

/// This includes common types and functions related to using ACL functionality.
mod acl {
    use super::*;

    /// An access control list (Allow/Deny lists).
    #[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo, MaxEncodedLen)]
    pub struct Acl<Address> {
        pub addresses: Vec<Address>,
        pub kind: AclKind,
        pub allow_null_recipient: bool,
    }

    /// Represents either an allow or deny list.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum AclKind {
        Allow,
        Deny,
    }

    /// Creates an empty ACL that always evaluates to false.
    impl<A: Default> Default for Acl<A> {
        fn default() -> Self {
            let addresses = Vec::<A>::default();
            Self { addresses, kind: AclKind::Allow, allow_null_recipient: false }
        }
    }
}
