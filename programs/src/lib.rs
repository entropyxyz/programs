//! Includes types and interfaces that are foundational to the core of programs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use ec_core as core;
/// All architecture-agnostic programs should be re-exported from this module
pub mod programs {
    pub use ec_acl as acl;
}
/// All architectures that implement the `ec_core::Architecture` trait should be re-exported from here.
pub mod arch {
    pub use ec_evm as evm;
}

/// Dynamic parsing allows for easily hooking transactions into
pub mod parsing {}
