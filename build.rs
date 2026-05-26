//! Build script: lower `schema/signal-spirit.schema` via schema-next and
//! emit Rust source via schema-rust-next into `$OUT_DIR/signal_spirit_generated.rs`.
//!
//! The library `src/lib.rs` then `include!`s the generated source under a
//! `generated` module. This realises the eventual `emit_schema!()` macro
//! shape (record 844) by mechanical build-script means while the proc-
//! macro front-end is not yet shipped on schema-rust-next.
//!
//! Per the designer running-concept brief (/368): the schema is THE
//! source of truth for wire types; the generated `.rs` is a derived
//! artifact rebuilt on every change.

use std::env;
use std::fs;
use std::path::PathBuf;

use schema_next::{SchemaEngine, SchemaIdentity};
use schema_rust_next::RustEmitter;

fn main() {
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schema/signal-spirit.schema");
    println!("cargo:rerun-if-changed={}", schema_path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let source = fs::read_to_string(&schema_path).expect("read signal-spirit.schema");
    let asschema = SchemaEngine::default()
        .lower_source(&source, SchemaIdentity::new("signal-spirit", "0.1.0"))
        .expect("lower signal-spirit.schema via schema-next");
    let generated = RustEmitter.emit_file(&asschema);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    let out_path = out_dir.join("signal_spirit_generated.rs");
    fs::write(&out_path, generated.code.as_str()).expect("write generated Rust source");
}
