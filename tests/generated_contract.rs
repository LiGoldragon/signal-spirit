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
            testimony: Testimony::new(vec![VerbatimQuote {
                quote_text: QuoteText::new("clarification means edit"),
                antecedent: None,
            }]),
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

#[test]
fn generated_signal_contract_exports_domain_tree() {
    let domain = Domain::Technology(Technology::Software(Software::Data(Some(
        DataLeaf::SchemaEvolution,
    ))));

    assert!(matches!(domain, Domain::Technology(_)));
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
