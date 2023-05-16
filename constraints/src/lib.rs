//! Includes types and interfaces that are foundational to the core of constraints.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use constraints::*;
pub mod constraints;
