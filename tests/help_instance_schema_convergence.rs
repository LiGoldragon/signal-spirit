//! Convergence proof: Help and per-instance schema project through the same
//! canonical schema reference spine and render identical schema text.
//!
//! Help stores `SpecifiedSchema`. A rendered Help response projects that value
//! into schema declarations; per-instance schema records the decoder's expected
//! reference. Both paths render `(Vector Domain)` for the `Domains` newtype
//! through the one schema encoder.
//!
//! Gated behind `nota-text` like the rest of the text surface.

#![cfg(feature = "nota-text")]

use nota_next::{InstanceSchema, NotaDecodeTraced, NotaSource};
use schema_next::InstanceSchemaText;
use signal_spirit::{Domains, HelpModel, HelpRequest};

/// Decode a real value and capture its per-instance schema trace.
fn instance_schema_of<Value>(source: &str) -> InstanceSchema
where
    Value: NotaDecodeTraced,
{
    let block = NotaSource::new(source)
        .parse_root()
        .expect("parse a single root object");
    let (_value, schema) = Value::from_nota_block_traced(&block)
        .expect("decode value and capture its instance schema")
        .into_parts();
    schema
}

/// The schema body text Help projects for a named target. Help stores
/// `SpecifiedSchema`; the rendered entry body is exposed through
/// signal-spirit's own `HelpBody` API, not schema-next source nouns.
fn help_body_schema_text(target: &str) -> String {
    let model = HelpModel::from_signal_schema_source().expect("build help model");
    let response = model
        .render(&HelpRequest::for_name(target))
        .unwrap_or_else(|error| panic!("render {target} help: {error}"));
    let entry = response.entries().first().expect("one help entry");
    entry
        .body()
        .expect("help entry has a body")
        .to_schema_text()
}

#[test]
fn help_domains_renders_the_canonical_vector_reference() {
    let model = HelpModel::from_signal_schema_source().expect("build help model");
    let rendered = model
        .render(&HelpRequest::for_name("Domains"))
        .expect("render Domains help")
        .to_string();
    // The dropped `(Vec Domain)` alias is gone; Help projects the canonical
    // `(Vector Domain)` through the one schema encoder.
    assert_eq!(rendered, "(Domains (Vector Domain))");
}

#[test]
fn help_body_exposes_schema_text_without_schema_next_source_nouns() {
    let help_text = help_body_schema_text("Domains");
    assert_eq!(help_text, "(Vector Domain)");
}

#[test]
fn help_domains_reference_matches_instance_schema_expansion() {
    // The whole-declaration views also agree: Help's `(Domains (Vector Domain))`
    // and the empty value's expanded per-instance schema are the same string,
    // because both re-head the same resolved vector reference.
    let model = HelpModel::from_signal_schema_source().expect("build help model");
    let help_rendered = model
        .render(&HelpRequest::for_name("Domains"))
        .expect("render Domains help")
        .to_string();

    let schema = instance_schema_of::<Domains>("[]");
    let instance_expanded = InstanceSchemaText::new(&schema).expanded();

    assert_eq!(help_rendered, instance_expanded);
    assert_eq!(help_rendered, "(Domains (Vector Domain))");
}
