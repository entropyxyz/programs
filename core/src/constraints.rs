}
//! Contains traits that constraints should implement, including Architecture-agnostic constraints and generic constraints.
//!
//! For runtime and binary size optimizations, constraint construction should be done at compile time by using `const` types, if possible. This can be done by using `const` generic parameters,
//! or by using a `const` builder. Both methods are described nicely here: https://wapl.es/rust/2022/07/03/const-builder-pattern.html

use crate::architecture::Architecture;
use crate::bindgen::Error;

/// Constraints on binary (or other unserialized) data must implement this. This is the most barebones trait for constraints.
pub trait Satisfiable {
    /// Indicates that the data satisfies the constraint.
    fn is_satisfied_by(self, data: &[u8]) -> Result<(), Error>;
}

/// Any constraint on transaction-like/architecture-agnostic signature requests must implement this.
///
/// For example, a constraint that checks that the recipient is not a blacklisted address would implement this trait for EVM, and would be similar to this:
/// ```
/// use ec_acl::*;
/// use ec_evm::*;
///
/// let non_blacklisted_addr: [u8; 20] = [1u8; 20];
/// let blacklisted_addr_1: [u8; 20] = [2u8; 20];
/// let blacklisted_addr_2: [u8; 20] = [3u8; 20];
///
/// let no_malicious_addresses = Acl {
///    addresses: vec![blacklisted_addr_1, blacklisted_addr_2],
///    kind: AclKind::Deny,
///    allow_null_recipient: false,
/// };
///
/// let non_blacklisted_recipient_tx = EvmTransactionRequest {
///    to: Some(NameOrAddress::Address(H160::from(non_blacklisted_addr))),
///   ..Default::default()
/// };
///
/// let blacklisted_recipient_tx = EvmTransactionRequest {
///    to: Some(NameOrAddress::Address(H160::from(blacklisted_addr_1))),
///   ..Default::default()
/// };
///
/// // This will be allowed, since the recipient is not in the blacklisted ACL.
/// no_malicious_addresses.clone().is_satisfied_by(&non_blacklisted_recipient_tx)?;
/// // This will return an error, because the recipient is not in the ACL.
/// assert!(no_malicious_addresses.is_satisfied_by(&blacklisted_recipient_tx).is_err());
/// Ok::<(), CoreError>(())
/// ```
///
pub trait SatisfiableForArchitecture<A: Architecture> {
    /// Indicates that the transaction request satisfies the constraint.
    fn is_satisfied_by(self, tx: &<A as Architecture>::TransactionRequest) -> Result<(), Error>;
