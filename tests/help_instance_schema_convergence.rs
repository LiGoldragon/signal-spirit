//! Convergence proof: Help and per-instance schema project the SAME resolved
//! reference IR and render identical schema text.
//!
//! The vision is one "what a type is" object — schema-next's resolved
//! `SourceReference` — with Help, instance-schema, and Rust lowering as
//! projections of it. Before the collapse, Help carried its own duplicate AST
//! (`HelpTypeExpression`) built from the raw source, so `(Vec Domain)` survived
//! as an opaque application in Help while instance-schema (reading the resolved
//! type) emitted the canonical `(Vector Domain)`. With the duplicate gone, both
//! read the one `SourceReference::Vector(Plain(Domain))` and render it through
//! the one schema encoder.
//!
//! Gated behind `nota-text` like the rest of the text surface.

#![cfg(feature = "nota-text")]

use nota_next::{InstanceSchema, InstanceSchemaBody, NotaDecodeTraced, NotaSource};
use schema_next::{InstanceSchemaText, SourceDeclarationValue, SourceReference};
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

/// The resolved reference Help projects for a named target. Help's entry body is
/// the resolved-IR `SourceDeclarationValue`; for a vector-typed root like
/// `Domains` that body is a `Reference(SourceReference::Vector(..))`.
fn help_reference(target: &str) -> SourceReference {
    let model = HelpModel::from_signal_schema_source().expect("build help model");
    let response = model
        .render(&HelpRequest::for_name(target))
        .unwrap_or_else(|error| panic!("render {target} help: {error}"));
    let entry = response
        .entries()
        .entries()
        .first()
        .expect("one help entry");
    match entry.body() {
        Some(SourceDeclarationValue::Reference(reference)) => reference.clone(),
        other => panic!("expected {target} help body to be a reference, found {other:?}"),
    }
}

/// The resolved reference the per-instance schema captured for a value's
/// vector position. `Domains` is a newtype over `(Vector Domain)`, so the
/// vector reference lives one level inside the newtype trace.
fn instance_vector_reference(schema: &InstanceSchema) -> SourceReference {
    let inner = match schema.body() {
        InstanceSchemaBody::Newtype(inner) => inner.as_ref(),
        // A bare vector value (no newtype wrapper) traces the vector directly.
        InstanceSchemaBody::Vector(_) => schema,
        other => panic!("expected a vector-bearing instance schema, found {other:?}"),
    };
    SourceReference::from_instance_reference(inner.expected())
}

#[test]
fn help_domains_renders_the_canonical_vector_reference() {
    let model = HelpModel::from_signal_schema_source().expect("build help model");
    let rendered = model
        .render(&HelpRequest::for_name("Domains"))
        .expect("render Domains help")
        .to_string();
    // The dropped `(Vec Domain)` alias is gone; Help projects the resolved IR's
    // canonical `(Vector Domain)` through the one schema encoder.
    assert_eq!(rendered, "(Domains (Vector Domain))");
}

#[test]
fn help_and_instance_schema_render_the_same_domains_reference() {
    // Help side: the resolved reference for the `Domains` type.
    let help_inner = help_reference("Domains");

    // Instance side: the resolved reference the decoder captured for a real
    // (empty) `Domains` value.
    let schema = instance_schema_of::<Domains>("[]");
    let instance_inner = instance_vector_reference(&schema);

    // Same IR object (a vector of the plain `Domain` type).
    assert_eq!(
        help_inner, instance_inner,
        "Help and instance-schema must project the same resolved SourceReference for Domains"
    );

    // Same rendered text, through the one schema encoder — neither path
    // hand-prints a spelling.
    let help_text = help_inner.rendered_schema_text();
    let instance_text = instance_inner.rendered_schema_text();
    assert_eq!(help_text, "(Vector Domain)");
    assert_eq!(
        help_text, instance_text,
        "Help and instance-schema must render the same reference to identical schema text"
    );
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
