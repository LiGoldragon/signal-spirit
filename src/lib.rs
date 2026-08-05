//! Generated ordinary Spirit wire contract.
//!
//! The sealed `spirit-ethos` package is the only source. `build.rs` lowers its
//! Interface root through the current Core/Nomos/Rust Logos chain and this
//! crate exposes that artifact without a legacy schema-rust projection.

#[allow(dead_code, non_camel_case_types)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/signal-spirit.rs"));
}

pub use generated::*;
