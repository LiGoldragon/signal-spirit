use signal_spirit::{
    ClarificationRecordIdentifier, ClarificationResolution, ClarificationResolutionReceipt,
    DataLeaf, Description, Domain, DomainMatch, DomainScope, DomainScopes, Domains, Input,
    InputRoute, Justification, OperationKind, Output, OutputRoute, QuoteText, Reasoning,
    RecordIdentifier, RecordIdentifiers, ScopeSet, Software, TargetClarification,
    TargetClarifications, Technology, Testimony, VerbatimQuote, VersionReport, VersionText,
};
#[cfg(feature = "nota-text")]
use std::collections::BTreeSet;

#[cfg(feature = "nota-text")]
use nota::{NotaEncode, NotaSource};

#[cfg(feature = "nota-text")]
const DOMAIN_HELP_ROW: &str = "[All (Health) (Food) (Home) (Finance) (Work) (Craft) (Knowledge) (Education) (Language) (Art) (Kinship) (Selfhood) (Spirituality) (Governance) (Law) (Community) (Nature) (Travel) (Commerce) (Leisure) (Appearance) (Safety) (Information) (Technology)]";

#[test]
fn generated_input_frame_round_trips() {
    let input = Input::Version;
    let bytes = input.encode_signal_frame().expect("encode input frame");
    let (route, decoded) = Input::decode_signal_frame(&bytes).expect("decode input frame");

    assert_eq!(route, InputRoute::Version);
    assert_eq!(decoded, input);
}

#[test]
fn generated_public_intent_frame_round_trips_without_moving_existing_routes() {
    let input = Input::public_intent(DomainScopes::new(vec![DomainScope::All]));
    let bytes = input.encode_signal_frame().expect("encode input frame");
    let (route, decoded) = Input::decode_signal_frame(&bytes).expect("decode input frame");

    assert_eq!(route, InputRoute::PublicIntent);
    assert_eq!(decoded, input);
    assert_eq!(
        OperationKind::from_input(&input),
        OperationKind::PublicIntent
    );
    assert_eq!(
        input.short_header(),
        0x0018_0000_0000_0000,
        "new PublicIntent route is appended after the existing route range"
    );
    assert_eq!(
        signal_spirit::schema::signal::short_header::INPUT_PUBLIC_TEXT_SEARCH,
        0x0008_0000_0000_0000,
        "existing PublicTextSearch route must keep its short header"
    );
    assert_eq!(
        signal_spirit::schema::signal::short_header::INPUT_MARKER,
        0x0017_0000_0000_0000,
        "existing Marker route must keep its short header when PublicIntent is added"
    );
}

#[test]
fn generated_output_frame_round_trips() {
    let output = Output::version_reported(VersionReport::new(VersionText::new("0.12.1")));
    let bytes = output.encode_signal_frame().expect("encode output frame");
    let (route, decoded) = Output::decode_signal_frame(&bytes).expect("decode output frame");

    assert_eq!(route, OutputRoute::VersionReported);
    assert_eq!(decoded, output);
}

#[test]
fn generated_resolve_clarification_frame_round_trips() {
    let input = Input::resolve_clarification(ClarificationResolution {
        clarification_record_identifier: ClarificationRecordIdentifier::new(RecordIdentifier::new(
            "clar1",
        )),
        target_clarifications: TargetClarifications::new(vec![TargetClarification {
            record_identifier: RecordIdentifier::new("targ1"),
            description: Description::new("clarified target"),
        }]),
        justification: Justification {
            testimony: Testimony::new(vec![VerbatimQuote::new(
                QuoteText::new("clarification means edit"),
                None,
            )]),
            reasoning: Reasoning::new("fold standalone clarification into target"),
        },
    });
    let bytes = input.encode_signal_frame().expect("encode input frame");
    let (route, decoded) = Input::decode_signal_frame(&bytes).expect("decode input frame");

    assert_eq!(route, InputRoute::ResolveClarification);
    assert_eq!(decoded, input);
}

#[test]
fn generated_clarification_resolved_frame_round_trips() {
    let output = Output::clarification_resolved(ClarificationResolutionReceipt {
        clarification_record_identifier: ClarificationRecordIdentifier::new(RecordIdentifier::new(
            "clar1",
        )),
        record_identifiers: RecordIdentifiers::new(vec![RecordIdentifier::new("targ1")]),
    });
    let bytes = output.encode_signal_frame().expect("encode output frame");
    let (route, decoded) = Output::decode_signal_frame(&bytes).expect("decode output frame");

    assert_eq!(route, OutputRoute::ClarificationResolved);
    assert_eq!(decoded, output);
}

#[cfg(feature = "nota-text")]
#[test]
fn negative_guideline_rejection_reason_round_trips_through_nota() {
    let rendered = signal_spirit::GuardianRejectionReason::NegativeGuideline.to_nota();

    assert_eq!(rendered, "NegativeGuideline");
    assert_eq!(
        NotaSource::new(&rendered)
            .parse::<signal_spirit::GuardianRejectionReason>()
            .expect("parse NegativeGuideline reason"),
        signal_spirit::GuardianRejectionReason::NegativeGuideline
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn matter_rejection_reason_round_trips_through_nota() {
    let rendered = signal_spirit::GuardianRejectionReason::Matter.to_nota();

    assert_eq!(rendered, "Matter");
    assert_eq!(
        NotaSource::new(&rendered)
            .parse::<signal_spirit::GuardianRejectionReason>()
            .expect("parse Matter reason"),
        signal_spirit::GuardianRejectionReason::Matter
    );
}

#[test]
fn generated_signal_contract_exports_domain_tree() {
    let domain = Domain::Technology(Technology::Software(Software::Data(
        DataLeaf::SchemaEvolution,
    )));

    assert!(matches!(domain, Domain::Technology(_)));
}

#[test]
fn public_domain_paths_are_signal_domain_types() {
    let shared_domain: signal_domain::Domain = signal_spirit::Domain::All;
    let top_level: signal_spirit::Domain = shared_domain.clone();
    let schema_path: signal_spirit::schema::domain::Domain = shared_domain.clone();
    let signal_schema_path: signal_spirit::schema::signal::Domain = shared_domain;

    assert_eq!(top_level, signal_domain::Domain::All);
    assert_eq!(schema_path, signal_domain::Domain::All);
    assert_eq!(signal_schema_path, signal_domain::Domain::All);

    let technology = signal_domain::Technology::Software(signal_domain::Software::Data(
        signal_domain::DataLeaf::SchemaEvolution,
    ));
    let top_level: Domain = Domain::Technology(technology);
    let schema_path: signal_spirit::schema::domain::Domain = top_level.clone();
    let signal_schema_path: signal_spirit::schema::signal::Domain = schema_path;

    assert!(matches!(
        signal_schema_path,
        signal_domain::Domain::Technology(_)
    ));
}

#[test]
fn public_domain_path_round_trips_through_rkyv() {
    let domain = signal_spirit::Domain::Technology(signal_spirit::Technology::Software(
        signal_spirit::Software::Data(signal_spirit::DataLeaf::SchemaEvolution),
    ));

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&domain).expect("encode domain");
    let decoded = rkyv::from_bytes::<signal_spirit::Domain, rkyv::rancor::Error>(&bytes)
        .expect("decode domain");

    assert_eq!(decoded, domain);
}

#[cfg(feature = "nota-text")]
#[test]
fn public_domain_path_round_trips_through_nota() {
    let domain = signal_spirit::Domain::Technology(signal_spirit::Technology::Software(
        signal_spirit::Software::Data(signal_spirit::DataLeaf::SchemaEvolution),
    ));
    let rendered = domain.to_nota();

    assert_eq!(
        NotaSource::new(&rendered)
            .parse::<signal_spirit::Domain>()
            .expect("parse domain"),
        domain
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn public_all_domain_and_scope_paths_round_trip_through_nota() {
    let rendered_domain = signal_spirit::Domain::All.to_nota();
    let rendered_scope = signal_spirit::DomainScope::All.to_nota();

    assert_eq!(
        NotaSource::new(&rendered_domain)
            .parse::<signal_domain::Domain>()
            .expect("parse Domain::All through shared type"),
        signal_domain::Domain::All
    );
    assert_eq!(
        NotaSource::new(&rendered_scope)
            .parse::<signal_domain::DomainScope>()
            .expect("parse DomainScope::All through shared type"),
        signal_domain::DomainScope::All
    );
}

#[test]
fn public_all_domain_and_scope_paths_round_trip_through_rkyv() {
    let domain = signal_spirit::Domain::All;
    let domain_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&domain).expect("archive Domain::All");
    let decoded_domain =
        rkyv::from_bytes::<signal_domain::Domain, rkyv::rancor::Error>(&domain_bytes)
            .expect("decode Domain::All archive through shared type");

    let scope = signal_spirit::DomainScope::All;
    let scope_bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(&scope).expect("archive DomainScope::All");
    let decoded_scope =
        rkyv::from_bytes::<signal_domain::DomainScope, rkyv::rancor::Error>(&scope_bytes)
            .expect("decode DomainScope::All archive through shared type");

    assert_eq!(decoded_domain, signal_domain::Domain::All);
    assert_eq!(decoded_scope, signal_domain::DomainScope::All);
}

#[cfg(feature = "nota-text")]
#[test]
fn generated_help_request_recognizes_top_level_and_named_forms() {
    assert_eq!(
        signal_spirit::HelpRequest::from_text("(Help)")
            .expect("parse top-level help")
            .expect("recognize help")
            .target(),
        None
    );
    assert_eq!(
        signal_spirit::HelpRequest::from_text("(Help Entry)")
            .expect("parse named help")
            .expect("recognize help")
            .target()
            .expect("named target")
            .as_str(),
        "Entry"
    );
    assert!(
        signal_spirit::HelpRequest::from_text("Version")
            .expect("parse non-help")
            .is_none(),
        "non-help NOTA should be left for generated Input parsing"
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn generated_help_model_renders_spirit_one_level_shapes() {
    let model = signal_spirit::HelpModel::from_signal_schema_source().expect("build help model");

    assert!(
        model
            .render(&signal_spirit::HelpRequest::new(None))
            .expect("render top-level help")
            .entries()
            .iter()
            .any(|entry| {
                entry.name().as_str() == "Record"
                    && entry.to_string()
                        == "{ Entry Justification }\n{ Domains Kind Description Certainty Importance Privacy Referents }\n{ Testimony Reasoning }"
            }),
        "top-level help should include Record's positional payload rows"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Record"))
            .expect("render Record help")
            .to_string(),
        "{ Entry Justification }\n{ Domains Kind Description Certainty Importance Privacy Referents }\n{ Testimony Reasoning }"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Entry"))
            .expect("render Entry help")
            .to_string(),
        "{ Domains Kind Description Certainty Importance Privacy Referents }\n(Vector Domain)\n[Decision Principle Correction Clarification Constraint]\nString\nMagnitude\nMagnitude\nMagnitude\n(Vector Referent)"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Domains"))
            .expect("render Domains help")
            .to_string(),
        "(Vector Domain)"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Description"))
            .expect("render Description help")
            .to_string(),
        "String"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("VerbatimQuote"))
            .expect("render VerbatimQuote help")
            .to_string(),
        "{ QuoteText OptionalAntecedent }\nString\n(Optional Antecedent)"
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn generated_help_model_renders_every_decoded_schema_target() {
    let model = signal_spirit::HelpModel::from_signal_schema_source().expect("build help model");
    let decoded = DecodedHelpTargets::from_deployed_schema_sources();

    let rendered_roots = model
        .render(&signal_spirit::HelpRequest::new(None))
        .expect("render top-level help")
        .entries()
        .iter()
        .map(|entry| entry.name().as_str().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rendered_roots, decoded.root_names,
        "top-level Help must be generated from every decoded Input and Output root"
    );

    for name in decoded.all_names() {
        let rendered = model
            .render(&signal_spirit::HelpRequest::for_name(name.clone()))
            .unwrap_or_else(|error| panic!("render decoded Help target {name}: {error}"))
            .to_string();
        assert!(
            !rendered.trim().is_empty(),
            "decoded Help target {name} rendered no positional rows"
        );
        assert!(
            !rendered.starts_with(&format!("({name} ")),
            "decoded Help target {name} leaked a wrapper head: {rendered}"
        );
    }

    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Domain"))
            .expect("render imported Domain help")
            .to_string(),
        // Help renders the positional body only. Same-named payload variants
        // are encoded in schema-language's self-tagged form, so each Domain
        // arm is one navigable step away from its nested enum body. Top-level
        // All is the explicit payload-free universal domain.
        DOMAIN_HELP_ROW
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("IntentEventStream"))
            .expect("render stream help")
            .to_string(),
        "(Stream { token.SubscriptionToken opened.SubscriptionStarted event.IntentEvent close.SubscriptionToken })"
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn generated_help_model_round_trips_through_rkyv() {
    let model = signal_spirit::HelpModel::from_signal_schema_source().expect("build help model");
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&model).expect("archive help model");
    let decoded = rkyv::from_bytes::<signal_spirit::HelpModel, rkyv::rancor::Error>(&bytes)
        .expect("decode help model");

    assert_eq!(decoded, model);

    let response = model
        .render(&signal_spirit::HelpRequest::for_name("Entry"))
        .expect("render typed help response");
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&response).expect("archive help response");
    let decoded = rkyv::from_bytes::<signal_spirit::HelpResponse, rkyv::rancor::Error>(&bytes)
        .expect("decode help response");

    assert_eq!(decoded, response);
    assert_eq!(
        decoded.entries()[0].name().as_str(),
        "Entry",
        "help response should preserve the typed entry name before rendering"
    );
}

/// The help text codec is schema-language's declaration-body codec, not a hand-rolled
/// or nota-derive one. For each representative target this asserts both the
/// canonical rendered positional rows and a true row round trip through that
/// same schema-language codec (`HelpResponse::to_schema_text` ->
/// `HelpResponse::from_schema_text` -> `to_schema_text`). Covered shapes: a
/// root payload struct (`Record`), a struct of newtype roles (`Entry`), a vector
/// reference rendered through TrueSchema as the canonical `(Vector Domain)`
/// (`Domains`), a newtype (`RecordAccepted`), an enum (`DomainMatch`), and a
/// stream (`IntentEventStream`).
#[cfg(feature = "nota-text")]
#[test]
fn generated_help_round_trips_through_the_schema_codec() {
    let model = signal_spirit::HelpModel::from_signal_schema_source().expect("build help model");

    for (target, expected) in [
        (
            "Record",
            "{ Entry Justification }\n{ Domains Kind Description Certainty Importance Privacy Referents }\n{ Testimony Reasoning }",
        ),
        (
            "Entry",
            "{ Domains Kind Description Certainty Importance Privacy Referents }\n(Vector Domain)\n[Decision Principle Correction Clarification Constraint]\nString\nMagnitude\nMagnitude\nMagnitude\n(Vector Referent)",
        ),
        ("Domains", "(Vector Domain)"),
        ("RecordAccepted", "RecordIdentifier"),
        // Same-named payload variants project to schema-language's self-tagged
        // form and round trip without leaking a duplicate payload name.
        ("Domain", DOMAIN_HELP_ROW),
        ("DomainMatch", "[Any (Partial) (Full)]"),
        (
            "IntentEventStream",
            "(Stream { token.SubscriptionToken opened.SubscriptionStarted event.IntentEvent close.SubscriptionToken })",
        ),
    ] {
        let response = model
            .render(&signal_spirit::HelpRequest::for_name(target))
            .unwrap_or_else(|error| panic!("render {target} help: {error}"));

        let encoded = response.to_schema_text();
        assert_eq!(
            encoded, expected,
            "{target} should render its canonical positional help rows"
        );
        assert_eq!(
            response.to_string(),
            expected,
            "{target} Display must delegate to the schema-language codec"
        );

        let decoded = signal_spirit::HelpResponse::from_schema_text(&encoded)
            .unwrap_or_else(|error| panic!("decode {target} help schema text: {error}"));
        assert_eq!(
            decoded.to_schema_text(),
            encoded,
            "{target} must round trip positional rows through the schema-language codec"
        );
    }

    // The multi-entry top-level response round trips as one positional row
    // document through the same codec.
    let top_level = model
        .render(&signal_spirit::HelpRequest::new(None))
        .expect("render top-level help");
    let encoded = top_level.to_schema_text();
    let decoded =
        signal_spirit::HelpResponse::from_schema_text(&encoded).expect("decode top-level help");
    assert_eq!(decoded.to_schema_text(), encoded);
}

#[cfg(feature = "nota-text")]
struct DecodedHelpTargets {
    root_names: BTreeSet<String>,
    declaration_names: BTreeSet<String>,
}

#[cfg(feature = "nota-text")]
impl DecodedHelpTargets {
    fn from_deployed_schema_sources() -> Self {
        let signal_source =
            schema_language::SchemaSource::from_schema_text(signal_spirit::SIGNAL_SCHEMA_SOURCE)
                .expect("signal schema source decodes");
        let domain_source =
            schema_language::SchemaSource::from_schema_text(signal_spirit::DOMAIN_SCHEMA_SOURCE)
                .expect("domain schema source decodes");
        let mut targets = Self {
            root_names: BTreeSet::new(),
            declaration_names: BTreeSet::new(),
        };
        targets.insert_root_names(signal_source.input());
        targets.insert_root_names(signal_source.output());
        targets.insert_namespace_names(signal_source.namespace());
        targets.insert_namespace_names(domain_source.namespace());
        targets
    }

    fn all_names(&self) -> BTreeSet<String> {
        self.root_names
            .iter()
            .chain(self.declaration_names.iter())
            .cloned()
            .collect()
    }

    fn insert_root_names(&mut self, root: &schema_language::SourceRootEnum) {
        if let Some(body) = root.body().as_enum() {
            for variant in body.variants() {
                self.root_names.insert(variant.name().as_str().to_owned());
            }
        }
    }

    fn insert_namespace_names(&mut self, namespace: &schema_language::SourceNamespace) {
        for entry in namespace.entries() {
            if let Some(value) = entry.value() {
                self.declaration_names
                    .insert(entry.name().as_str().to_owned());
                self.insert_inline_declaration_names(value);
            }
            if let Some(child_namespace) = entry.namespace() {
                self.insert_namespace_names(child_namespace);
            }
        }
    }

    fn insert_inline_declaration_names(&mut self, value: &schema_language::SourceDeclarationValue) {
        match value {
            schema_language::SourceDeclarationValue::Struct(body) => {
                for field in body.fields() {
                    self.insert_field_declaration_name(field);
                }
            }
            schema_language::SourceDeclarationValue::Enum(body) => {
                for variant in body.variants() {
                    if let Some(schema_language::SourceVariantPayload::Declaration(value)) =
                        variant.payload_source()
                    {
                        self.declaration_names
                            .insert(variant.name().as_str().to_owned());
                        self.insert_inline_declaration_names(value);
                    }
                }
            }
            schema_language::SourceDeclarationValue::Reference(_)
            | schema_language::SourceDeclarationValue::Text(_)
            | schema_language::SourceDeclarationValue::Stream(_)
            | schema_language::SourceDeclarationValue::Family(_) => {}
        }
    }

    fn insert_field_declaration_name(&mut self, field: &schema_language::SourceField) {
        if !Self::is_type_name(field.name().as_str()) {
            return;
        }
        self.declaration_names
            .insert(field.name().as_str().to_owned());
        if let schema_language::SourceFieldValue::Declaration(value) = field.value() {
            self.insert_inline_declaration_names(value);
        }
    }

    fn is_type_name(name: &str) -> bool {
        name.chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
            && !matches!(name, "String" | "Integer" | "Boolean" | "Path" | "Bytes")
    }
}

#[cfg(feature = "nota-text")]
#[test]
fn top_level_all_domain_round_trips_through_nota() {
    let domain = "All".parse_domain().expect("top-level all domain parses");

    assert_eq!(domain, Domain::All);
    assert_eq!(domain.to_nota(), "All");
}

#[cfg(feature = "nota-text")]
#[test]
fn terminal_value_domain_tags_round_trip_through_nota() {
    // Strict positional NOTA: bare `Data` no longer decodes — a variant payload
    // must always appear, and the whole-category value is the explicit `All`.
    assert!(
        "(Technology (Software Data))".parse_domain().is_err(),
        "bare Data must be rejected now that the payload is required"
    );

    let domain = "(Technology (Software (Data All)))"
        .parse_domain()
        .expect("explicit all-data domain parses");

    assert_eq!(
        domain,
        Domain::Technology(Technology::Software(Software::Data(DataLeaf::All)))
    );
    assert_eq!(domain.to_nota(), "(Technology (Software (Data All)))");
}

#[cfg(feature = "nota-text")]
#[test]
fn curated_leaf_domain_tags_round_trip_through_nota() {
    let domain = "(Technology (Software (Data SchemaEvolution)))"
        .parse_domain()
        .expect("schema evolution domain parses");

    assert_eq!(
        domain,
        Domain::Technology(Technology::Software(Software::Data(
            DataLeaf::SchemaEvolution
        )))
    );
    assert_eq!(
        domain.to_nota(),
        "(Technology (Software (Data SchemaEvolution)))"
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn pure_terminal_theory_domain_round_trips_through_nota() {
    let domain = "(Technology (Software Theory))"
        .parse_domain()
        .expect("theory terminal domain parses");

    assert_eq!(
        domain,
        Domain::Technology(Technology::Software(Software::Theory))
    );
    assert_eq!(domain.to_nota(), "(Technology (Software Theory))");
}

#[cfg(feature = "nota-text")]
#[test]
fn deleted_software_leaves_do_not_parse() {
    for deleted in [
        "(Technology (Software (Distributed ServiceMesh)))",
        "(Technology (Software (Quality UnitTesting)))",
        "(Technology (Software (Engineering SoftwareArchitecture)))",
    ] {
        assert!(
            deleted.parse_domain().is_err(),
            "deleted domain leaf must not parse: {deleted}"
        );
    }
}

#[test]
fn terminal_value_domains_convert_to_scope_all() {
    let scope = DomainScope::from(Domain::Technology(Technology::Software(Software::Data(
        DataLeaf::All,
    ))));

    assert!(
        scope.contains_domain(&Domain::Technology(Technology::Software(Software::Data(
            DataLeaf::SchemaEvolution,
        ))))
    );
}

#[test]
fn top_level_all_scope_matches_every_entry_domain() {
    let all_scope = DomainScope::from(Domain::All);
    let data_domain = Domain::Technology(Technology::Software(Software::Data(
        DataLeaf::SchemaEvolution,
    )));
    let domains = Domains::new(vec![data_domain.clone()]);

    assert_eq!(all_scope, DomainScope::All);
    assert!(all_scope.matches_domain(&data_domain));
    assert!(data_domain.matches_scope(&all_scope));
    assert!(ScopeSet::new(vec![all_scope.clone()]).matches_domain(&data_domain));
    assert!(DomainScopes::new(vec![all_scope.clone()]).matches_any_domain(domains.payload()));
    assert!(DomainMatch::partial(DomainScopes::new(vec![all_scope.clone()])).matches(&domains));
    assert!(DomainMatch::full(DomainScopes::new(vec![all_scope])).matches(&domains));
}

#[cfg(feature = "nota-text")]
trait DomainNotaTest {
    fn parse_domain(&self) -> Result<Domain, nota::NotaDecodeError>;
}

#[cfg(feature = "nota-text")]
impl DomainNotaTest for str {
    fn parse_domain(&self) -> Result<Domain, nota::NotaDecodeError> {
        NotaSource::new(self).parse::<Domain>()
    }
}
