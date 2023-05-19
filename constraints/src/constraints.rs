use core::fmt::Debug;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use ec_acl::*;

/// Supported architectures.
/// DEPRECATED: This will be removed with V1 constraints removal in favor of a more generic listing of architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Serialize, Deserialize, TypeInfo)]
pub enum Arch {
    Evm,
    /// Not yet supported on the client, supported in Substrate storage/ACL
    Btc,
}

/// Represents a user's constraints
/// DEPRECATED: This will be removed with V1 constraints removal 
#[derive(Default, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct Constraints {
    pub evm_acl: Option<Acl<[u8; 20]>>,
    pub btc_acl: Option<Acl<[u8; 32]>>,
}
