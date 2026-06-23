//! Per-instance schema over the REAL spirit contract.
//!
//! Each test decodes a real signal-spirit value with the generated
//! decoder-driven `NotaDecodeTraced` (the emitter put it on every type,
//! including the hand-emitted optional-leaf `Domain` taxonomy), then renders
//! the captured trace through schema's encoder and asserts the endorsed
//! form. Both the decoded value and the captured `expected` types are checked,
//! and the rendered reference tokens round-trip through
//! `SourceReference::from_block`.
//!
//! Gated behind `nota-text` like the rest of signal-spirit's text surface.

#![cfg(feature = "nota-text")]

use nota::{InstanceSchema, InstanceSchemaBody, NotaDecodeTraced, NotaSource, TypeReference};
use schema::{InstanceSchemaText, SourceReference};
use signal_spirit::{Certainty, Domain, DomainMatch, Entry, Input, Kind, Magnitude};

fn schema_of<Value>(source: &str) -> (Value, InstanceSchema)
where
    Value: NotaDecodeTraced,
{
    let block = NotaSource::new(source)
        .parse_root()
        .expect("parse a single root object");
    Value::from_nota_block_traced(&block)
        .expect("decode value and capture its instance schema")
        .into_parts()
}

fn named(reference: &TypeReference) -> &str {
    match reference {
        TypeReference::Named(name) => name,
        other => panic!("expected a named reference, found {other:?}"),
    }
}

fn enum_payload(schema: &InstanceSchema) -> &InstanceSchema {
    match schema.body() {
        InstanceSchemaBody::EnumPayload(Some(inner)) => inner,
        other => panic!("expected an enum payload, found {other:?}"),
    }
}

/// Every parenthesised reference token the renderer emits must parse back
/// through schema's own reference reader.
fn round_trips_as_reference(text: &str) {
    let block = NotaSource::new(text)
        .parse_root()
        .expect("rendered reference parses as a NOTA root");
    SourceReference::from_block(&block)
        .expect("rendered reference round-trips through SourceReference::from_block");
}

#[test]
fn kind_value_renders_the_enum_name() {
    let (value, schema) = schema_of::<Kind>("Decision");
    assert_eq!(value, Kind::Decision);
    assert_eq!(named(schema.expected()), "Kind");
    assert_eq!(InstanceSchemaText::new(&schema).aligned(), "Kind");
}

#[test]
fn certainty_newtype_preserves_wrapper_then_magnitude() {
    let (value, schema) = schema_of::<Certainty>("High");
    assert_eq!(value, Certainty::new(Magnitude::High));
    assert_eq!(named(schema.expected()), "Certainty");
    let InstanceSchemaBody::Newtype(inner) = schema.body() else {
        panic!("certainty must carry a Newtype body");
    };
    assert_eq!(named(inner.expected()), "Magnitude");
    assert_eq!(
        InstanceSchemaText::new(&schema).expanded(),
        "(Certainty Magnitude)"
    );
    round_trips_as_reference("(Certainty Magnitude)");
}

#[test]
fn entry_renders_its_field_type_names_in_declared_order() {
    let source = "([(Technology (Software (Programming CodeGeneration)))] Decision [a description] High Low Zero [spirit])";
    let (value, schema) = schema_of::<Entry>(source);
    assert_eq!(value.kind, Kind::Decision);
    assert_eq!(value.certainty, Certainty::new(Magnitude::High));

    assert_eq!(named(schema.expected()), "Entry");
    assert_eq!(
        InstanceSchemaText::new(&schema).aligned(),
        "{ Domains Kind Description Certainty Importance Privacy Referents }"
    );
}

#[test]
fn empty_domains_still_names_its_element_type() {
    use signal_spirit::Domains;
    let (value, schema) = schema_of::<Domains>("[]");
    assert_eq!(value, Domains::new(vec![]));
    assert_eq!(named(schema.expected()), "Domains");
    assert_eq!(InstanceSchemaText::new(&schema).aligned(), "Domains");
    assert_eq!(
        InstanceSchemaText::new(&schema).expanded(),
        "(Domains (Vector Domain))"
    );
    round_trips_as_reference("(Vector Domain)");
}

#[test]
fn domain_path_traces_expected_types_down_the_real_taxonomy() {
    let (value, schema) =
        schema_of::<Domain>("(Technology (Software (Programming CodeGeneration)))");
    assert!(matches!(value, Domain::Technology(_)));

    // Expected-type trace: Domain -> Technology -> Software -> (Optional ProgrammingLeaf).
    assert_eq!(named(schema.expected()), "Domain");
    let technology = enum_payload(&schema);
    assert_eq!(named(technology.expected()), "Technology");
    let software = enum_payload(technology);
    assert_eq!(named(software.expected()), "Software");
    // Software::Programming carries Option<ProgrammingLeaf>; the payload node is
    // the optional, with the leaf type one level in.
    let optional = enum_payload(software);
    match optional.expected() {
        TypeReference::Optional(inner) => assert_eq!(named(inner), "ProgrammingLeaf"),
        other => panic!("expected (Optional ProgrammingLeaf), found {other:?}"),
    }
    let InstanceSchemaBody::Optional(Some(leaf)) = optional.body() else {
        panic!("the realized programming leaf must be present");
    };
    assert_eq!(named(leaf.expected()), "ProgrammingLeaf");
}

#[test]
fn bare_optional_leaf_variant_traces_an_empty_optional() {
    // (Technology (Software Programming)) -> Software::Programming(None).
    let (value, schema) = schema_of::<Domain>("(Technology (Software Programming))");
    assert!(matches!(value, Domain::Technology(_)));
    let software = enum_payload(enum_payload(&schema));
    assert_eq!(named(software.expected()), "Software");
    let optional = enum_payload(software);
    match optional.expected() {
        TypeReference::Optional(inner) => assert_eq!(named(inner), "ProgrammingLeaf"),
        other => panic!("expected (Optional ProgrammingLeaf), found {other:?}"),
    }
    assert!(matches!(
        optional.body(),
        InstanceSchemaBody::Optional(None)
    ));
}

#[test]
fn domain_match_partial_renders_enum_name_with_payload_reference() {
    let (value, schema) = schema_of::<DomainMatch>(
        "(Partial [(Technology (Software (Programming CodeGeneration)))])",
    );
    assert!(matches!(value, DomainMatch::Partial(_)));
    assert_eq!(named(schema.expected()), "DomainMatch");
    // The transparent Partial wrapper collapses to its DomainScopes payload.
    assert_eq!(
        InstanceSchemaText::new(&schema).aligned(),
        "(DomainMatch DomainScopes)"
    );
    round_trips_as_reference("(DomainMatch DomainScopes)");
}

#[test]
fn root_input_record_renders_the_endorsed_root_form() {
    let source = "(Record (([(Technology (Software (Programming CodeGeneration)))] Decision [a description] Medium Medium Zero [spirit]) ([([a quote] None)] [the reasoning])))";
    let (value, schema) = schema_of::<Input>(source);
    assert!(matches!(value, Input::Record(_)));

    // Root schema node is the enum name Input, not the variant Record.
    assert_eq!(named(schema.expected()), "Input");
    assert_eq!(
        InstanceSchemaText::new(&schema).aligned(),
        "(Input ({ Domains Kind Description Certainty Importance Privacy Referents } { Testimony Reasoning }))"
    );
}

#[test]
fn unit_root_variant_is_a_scalar_terminal() {
    let (value, schema) = schema_of::<Input>("Version");
    assert_eq!(value, Input::Version);
    assert_eq!(named(schema.expected()), "Input");
    assert!(matches!(
        schema.body(),
        InstanceSchemaBody::EnumPayload(None)
    ));
    assert_eq!(InstanceSchemaText::new(&schema).aligned(), "Input");
}

#[test]
fn traced_value_agrees_with_ordinary_decode() {
    let source = "(Technology (Software Theory))";
    let (traced, _) = schema_of::<Domain>(source);
    let ordinary = NotaSource::new(source)
        .parse::<Domain>()
        .expect("ordinary decode");
    assert_eq!(traced, ordinary);
}
