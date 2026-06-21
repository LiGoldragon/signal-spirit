use signal_spirit::{
    ClarificationRecordIdentifier, ClarificationResolution, ClarificationResolutionReceipt,
    DataLeaf, Description, Domain, DomainScope, Input, InputRoute, Justification, Output,
    OutputRoute, QuoteText, Reasoning, RecordIdentifier, RecordIdentifiers, Software,
    TargetClarification, TargetClarifications, Technology, Testimony, VerbatimQuote, VersionReport,
    VersionText,
};

#[cfg(feature = "nota-text")]
use nota_next::{NotaEncode, NotaSource};

#[test]
fn generated_input_frame_round_trips() {
    let input = Input::Version;
    let bytes = input.encode_signal_frame().expect("encode input frame");
    let (route, decoded) = Input::decode_signal_frame(&bytes).expect("decode input frame");

    assert_eq!(route, InputRoute::Version);
    assert_eq!(decoded, input);
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

#[test]
fn generated_signal_contract_exports_domain_tree() {
    let domain = Domain::Technology(Technology::Software(Software::Data(Some(
        DataLeaf::SchemaEvolution,
    ))));

    assert!(matches!(domain, Domain::Technology(_)));
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
            .entries()
            .iter()
            .any(|entry| entry.to_string() == "(Record { Entry Justification })"),
        "top-level help should include Record's one-level payload shape"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Record"))
            .expect("render Record help")
            .to_string(),
        "(Record { Entry Justification })"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Entry"))
            .expect("render Entry help")
            .to_string(),
        "(Entry { Domains Kind Description Certainty Importance Privacy Referents })"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Domains"))
            .expect("render Domains help")
            .to_string(),
        "(Domains (Vec Domain))"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("Description"))
            .expect("render Description help")
            .to_string(),
        "(Description String)"
    );
    assert_eq!(
        model
            .render(&signal_spirit::HelpRequest::for_name("VerbatimQuote"))
            .expect("render VerbatimQuote help")
            .to_string(),
        "(VerbatimQuote { QuoteText OptionalAntecedent })"
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
        decoded.entries().entries()[0].name().as_str(),
        "Entry",
        "help response should preserve the typed entry name before rendering"
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn terminal_value_domain_tags_round_trip_through_nota() {
    let domain = "(Technology (Software Data))"
        .parse_domain()
        .expect("terminal data domain parses");

    assert_eq!(
        domain,
        Domain::Technology(Technology::Software(Software::Data(None)))
    );
    assert_eq!(domain.to_nota(), "(Technology (Software Data))");
}

#[cfg(feature = "nota-text")]
#[test]
fn curated_leaf_domain_tags_round_trip_through_nota() {
    let domain = "(Technology (Software (Data SchemaEvolution)))"
        .parse_domain()
        .expect("schema evolution domain parses");

    assert_eq!(
        domain,
        Domain::Technology(Technology::Software(Software::Data(Some(
            DataLeaf::SchemaEvolution,
        ))))
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
        None,
    ))));

    assert!(
        scope.contains_domain(&Domain::Technology(Technology::Software(Software::Data(
            Some(DataLeaf::SchemaEvolution),
        ))))
    );
}

#[cfg(feature = "nota-text")]
trait DomainNotaTest {
    fn parse_domain(&self) -> Result<Domain, nota_next::NotaDecodeError>;
}

#[cfg(feature = "nota-text")]
impl DomainNotaTest for str {
    fn parse_domain(&self) -> Result<Domain, nota_next::NotaDecodeError> {
        NotaSource::new(self).parse::<Domain>()
    }
}
