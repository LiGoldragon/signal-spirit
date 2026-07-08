//! Per-instance schema over the REAL spirit contract.
//!
//! Each test decodes a real signal-spirit value with the generated
//! decoder-driven `NotaDecodeTraced` (the emitter puts the canonical derive
//! on every type, including the whole `Domain` taxonomy whose value leaves now
//! carry required payloads), then renders
//! the captured trace through schema-language's encoder and asserts the endorsed
//! form. Both the decoded value and the captured `expected` types are checked,
//! and each rendered bare or dotted reference token round-trips through
//! `SourceReference::from_block`.
//!
//! Gated behind `nota-text` like the rest of signal-spirit's text surface.

#![cfg(feature = "nota-text")]

use nota::{InstanceSchema, InstanceSchemaBody, NotaDecodeTraced, NotaSource, TypeReference};
use schema_language::{InstanceSchemaText, SourceReference};
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

/// Every bare or dotted reference token the renderer emits must parse back
/// through schema-language's own reference reader. Expanded instance-schema
/// tuples such as `(Input DomainScopes)` are structural traces rather than a
/// single syntax reference.
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
    round_trips_as_reference("Magnitude");
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
        "(Domains Vector.Domain)"
    );
    round_trips_as_reference("Vector.Domain");
}

#[test]
fn domain_path_traces_expected_types_down_the_real_taxonomy() {
    let (value, schema) =
        schema_of::<Domain>("(Technology (Software (Programming CodeGeneration)))");
    assert!(matches!(value, Domain::Technology(_)));

    // Expected-type trace: Domain -> TechnologyDomain -> SoftwareDomain -> ProgrammingLeaf.
    assert_eq!(named(schema.expected()), "Domain");
    let technology = enum_payload(&schema);
    assert_eq!(named(technology.expected()), "TechnologyDomain");
    let software = enum_payload(technology);
    assert_eq!(named(software.expected()), "SoftwareDomain");
    // Software::Programming now carries a required ProgrammingLeaf; the payload
    // node is the leaf enum itself, with no intervening Optional.
    let programming = enum_payload(software);
    assert_eq!(named(programming.expected()), "ProgrammingLeaf");
    assert!(matches!(
        programming.body(),
        InstanceSchemaBody::EnumPayload(None)
    ));
}

#[test]
fn bare_leaf_variant_without_payload_is_rejected_and_all_traces_the_leaf() {
    // Strict positional NOTA: bare `Programming` no longer decodes — the payload
    // is required. The whole-category value is the explicit `All` leaf, which
    // traces the required ProgrammingLeaf payload with no empty Optional.
    let bare = NotaSource::new("(Technology (Software Programming))")
        .parse_root()
        .expect("parse a single root object");
    assert!(
        Domain::from_nota_block_traced(&bare).is_err(),
        "bare Programming must be rejected now that the payload is required"
    );

    let (value, schema) = schema_of::<Domain>("(Technology (Software (Programming All)))");
    assert!(matches!(value, Domain::Technology(_)));
    let software = enum_payload(enum_payload(&schema));
    assert_eq!(named(software.expected()), "SoftwareDomain");
    let programming = enum_payload(software);
    assert_eq!(named(programming.expected()), "ProgrammingLeaf");
    assert!(matches!(
        programming.body(),
        InstanceSchemaBody::EnumPayload(None)
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
    round_trips_as_reference("DomainScopes");
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
fn root_input_public_intent_renders_domain_scopes_payload() {
    let source = "(PublicIntent [All])";
    let (value, schema) = schema_of::<Input>(source);
    assert!(matches!(value, Input::PublicIntent(_)));

    assert_eq!(named(schema.expected()), "Input");
    assert_eq!(
        InstanceSchemaText::new(&schema).aligned(),
        "(Input DomainScopes)"
    );
    round_trips_as_reference("DomainScopes");
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
