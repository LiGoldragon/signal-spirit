//! Strict Rust projection of the authored spirit Interface.
//!
//! Generation gaps (licensed breakage per hqu.14 replacement-kills):
//! - rust-logos does not yet emit `#[derive(...)]` attributes (no Clone,
//!   rkyv, etc. — Debug is manually added where the Refusal bound needs it)
//! - rust-logos does not yet emit `Display` / `Error` implementations
//!   for Refusal types (manually supplied below)
//! - The old nomos-engine pipeline generated full derives and accessor
//!   methods; this bare projection does not yet replicate that surface
//!
// psyche-grasp: unseen

#![allow(dead_code, non_camel_case_types)]

/// Rust mapping of the Ethos `Unit` builtin — a type with exactly one
/// inhabitant, isomorphic to `()`.
pub type Unit = ();

/// Rust mapping of the Ethos `Integer` builtin.
pub type Integer = i64;

include!("generated.rs");

// Generation gap: the Refusal role trait requires `std::error::Error`,
// which requires `Debug + Display`. rust-logos does not yet emit these.
impl std::fmt::Display for AdmissionRejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "admission rejected: {:?}", self.0)
    }
}

impl std::error::Error for AdmissionRejected {}

impl std::fmt::Display for QueryRejected {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "query rejected: {:?}", self.0)
    }
}

impl std::error::Error for QueryRejected {}
