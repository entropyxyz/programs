}
//! Includes types and interfaces that are foundational to the core of constraints.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use ec_core as core;
/// All architecture-agnostic constraints should be re-exported from this module
pub mod constraints {
    pub use ec_acl as acl;
}
/// All architectures that implement the `ec_core::Architecture` trait should be re-exported from here.
pub mod arch {
    pub use ec_evm as evm;
}

/// Dynamic parsing allows for easily hooking transactions into
