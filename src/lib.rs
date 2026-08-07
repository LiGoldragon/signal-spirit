//! Signal contract for the ordinary spirit surface.
//!
//! `schema/spirit.schema` is the authored Interface source, rescued from the
//! condemned `spirit-ethos` repository and re-spelled to the blessed 5-section
//! form. `build.rs` authorizes it through `SemaBootstrapAuthority` and
//! generates the Rust projection through `CommitBootstrap`.
//!
//! The Observer stream's initiation/termination mapping is held for the
//! psyche — specifically whether `Observe.Query` double-duties as stream
//! initiation and that a termination entry does not yet exist.
//!
// psyche-grasp: unseen

pub mod schema;

pub use crate::schema::spirit::*;

/// Canonical textual projection of the spirit Interface.
pub const SPIRIT_INTERFACE_SOURCE: &str = include_str!("../schema/spirit.schema");
