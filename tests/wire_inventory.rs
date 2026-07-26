use std::marker::PhantomData;

#[cfg(feature = "nota-text")]
use signal_spirit::OperationKind;
use signal_spirit::schema::signal::short_header;
#[cfg(feature = "nota-text")]
use signal_spirit::{Input, Output};
use signal_spirit::{InputRoute, OutputRoute};

const INPUT_ROUTES: [(&str, InputRoute); 25] = [
    ("State", InputRoute::State),
    ("Record", InputRoute::Record),
    ("Propose", InputRoute::Propose),
    ("Clarify", InputRoute::Clarify),
    ("Supersede", InputRoute::Supersede),
    ("Retire", InputRoute::Retire),
    ("ResolveClarification", InputRoute::ResolveClarification),
    ("Observe", InputRoute::Observe),
    ("PublicTextSearch", InputRoute::PublicTextSearch),
    ("PublicRecords", InputRoute::PublicRecords),
    ("PrivateRecords", InputRoute::PrivateRecords),
    ("Lookup", InputRoute::Lookup),
    ("Count", InputRoute::Count),
    ("ChangeCertainty", InputRoute::ChangeCertainty),
    ("BumpImportance", InputRoute::BumpImportance),
    ("ChangeRecord", InputRoute::ChangeRecord),
    ("RegisterReferent", InputRoute::RegisterReferent),
    ("LookupStash", InputRoute::LookupStash),
    ("Tap", InputRoute::Tap),
    ("Untap", InputRoute::Untap),
    ("ApplyAuthorizedRecord", InputRoute::ApplyAuthorizedRecord),
    ("SubscribeIntent", InputRoute::SubscribeIntent),
    ("Version", InputRoute::Version),
    ("Marker", InputRoute::Marker),
    ("PublicIntent", InputRoute::PublicIntent),
];

const OUTPUT_ROUTES: [(&str, OutputRoute); 27] = [
    ("RecordAccepted", OutputRoute::RecordAccepted),
    ("Proposed", OutputRoute::Proposed),
    ("Clarified", OutputRoute::Clarified),
    ("Superseded", OutputRoute::Superseded),
    ("Retired", OutputRoute::Retired),
    ("ClarificationResolved", OutputRoute::ClarificationResolved),
    ("GuardianRejected", OutputRoute::GuardianRejected),
    (
        "ReferentGuardianRejected",
        OutputRoute::ReferentGuardianRejected,
    ),
    ("RecordsObserved", OutputRoute::RecordsObserved),
    ("RecordsStashed", OutputRoute::RecordsStashed),
    ("RecordFound", OutputRoute::RecordFound),
    ("RecordsCounted", OutputRoute::RecordsCounted),
    ("CertaintyChanged", OutputRoute::CertaintyChanged),
    ("ImportanceBumped", OutputRoute::ImportanceBumped),
    ("RecordChanged", OutputRoute::RecordChanged),
    ("ReferentRegistered", OutputRoute::ReferentRegistered),
    ("ObservationTapped", OutputRoute::ObservationTapped),
    ("ObservationUntapped", OutputRoute::ObservationUntapped),
    ("SubscriptionStarted", OutputRoute::SubscriptionStarted),
    ("VersionReported", OutputRoute::VersionReported),
    ("MarkerReported", OutputRoute::MarkerReported),
    ("RecordApplied", OutputRoute::RecordApplied),
    ("ApplyRefused", OutputRoute::ApplyRefused),
    ("Event", OutputRoute::Event),
    ("Error", OutputRoute::Error),
    ("Rejected", OutputRoute::Rejected),
    ("AdvanceRefused", OutputRoute::AdvanceRefused),
];

const INPUT_SHORT_HEADERS: [u64; 25] = [
    short_header::INPUT_STATE,
    short_header::INPUT_RECORD,
    short_header::INPUT_PROPOSE,
    short_header::INPUT_CLARIFY,
    short_header::INPUT_SUPERSEDE,
    short_header::INPUT_RETIRE,
    short_header::INPUT_RESOLVE_CLARIFICATION,
    short_header::INPUT_OBSERVE,
    short_header::INPUT_PUBLIC_TEXT_SEARCH,
    short_header::INPUT_PUBLIC_RECORDS,
    short_header::INPUT_PRIVATE_RECORDS,
    short_header::INPUT_LOOKUP,
    short_header::INPUT_COUNT,
    short_header::INPUT_CHANGE_CERTAINTY,
    short_header::INPUT_BUMP_IMPORTANCE,
    short_header::INPUT_CHANGE_RECORD,
    short_header::INPUT_REGISTER_REFERENT,
    short_header::INPUT_LOOKUP_STASH,
    short_header::INPUT_TAP,
    short_header::INPUT_UNTAP,
    short_header::INPUT_APPLY_AUTHORIZED_RECORD,
    short_header::INPUT_SUBSCRIBE_INTENT,
    short_header::INPUT_VERSION,
    short_header::INPUT_MARKER,
    short_header::INPUT_PUBLIC_INTENT,
];

const OUTPUT_SHORT_HEADERS: [u64; 27] = [
    short_header::OUTPUT_RECORD_ACCEPTED,
    short_header::OUTPUT_PROPOSED,
    short_header::OUTPUT_CLARIFIED,
    short_header::OUTPUT_SUPERSEDED,
    short_header::OUTPUT_RETIRED,
    short_header::OUTPUT_CLARIFICATION_RESOLVED,
    short_header::OUTPUT_GUARDIAN_REJECTED,
    short_header::OUTPUT_REFERENT_GUARDIAN_REJECTED,
    short_header::OUTPUT_RECORDS_OBSERVED,
    short_header::OUTPUT_RECORDS_STASHED,
    short_header::OUTPUT_RECORD_FOUND,
    short_header::OUTPUT_RECORDS_COUNTED,
    short_header::OUTPUT_CERTAINTY_CHANGED,
    short_header::OUTPUT_IMPORTANCE_BUMPED,
    short_header::OUTPUT_RECORD_CHANGED,
    short_header::OUTPUT_REFERENT_REGISTERED,
    short_header::OUTPUT_OBSERVATION_TAPPED,
    short_header::OUTPUT_OBSERVATION_UNTAPPED,
    short_header::OUTPUT_SUBSCRIPTION_STARTED,
    short_header::OUTPUT_VERSION_REPORTED,
    short_header::OUTPUT_MARKER_REPORTED,
    short_header::OUTPUT_RECORD_APPLIED,
    short_header::OUTPUT_APPLY_REFUSED,
    short_header::OUTPUT_EVENT,
    short_header::OUTPUT_ERROR,
    short_header::OUTPUT_REJECTED,
    short_header::OUTPUT_ADVANCE_REFUSED,
];

#[test]
fn complete_route_header_and_tag_inventory_is_stable() {
    for (index, ((_, route), header)) in INPUT_ROUTES.iter().zip(INPUT_SHORT_HEADERS).enumerate() {
        let expected_header = ((index as u64) << 48) | 0x0000_0001_0000_0001;
        assert_eq!(
            header, expected_header,
            "Input route {route:?} moved or lost its SignalSpirit binding"
        );
        let archived =
            rkyv::to_bytes::<rkyv::rancor::Error>(route).expect("archive Input route tag");
        assert_eq!(
            archived.as_ref(),
            &[index as u8],
            "Input route tag at index {index} moved"
        );
    }

    for (index, ((_, route), header)) in OUTPUT_ROUTES.iter().zip(OUTPUT_SHORT_HEADERS).enumerate()
    {
        let expected_header = ((0x0100_u64 + index as u64) << 48) | 0x0000_0001_0000_0001;
        assert_eq!(
            header, expected_header,
            "Output route {route:?} moved or lost its SignalSpirit binding"
        );
        let archived =
            rkyv::to_bytes::<rkyv::rancor::Error>(route).expect("archive Output route tag");
        assert_eq!(
            archived.as_ref(),
            &[index as u8],
            "Output route tag at index {index} moved"
        );
    }
}

fn same_type<Type>(_: PhantomData<Type>, _: PhantomData<Type>) {}

macro_rules! assert_alias {
    ($public:ty, $generated:ty) => {
        same_type(PhantomData::<$public>, PhantomData::<$generated>);
    };
}

#[test]
fn established_public_payload_names_remain_exact_type_aliases() {
    assert_alias!(signal_spirit::SpiritFrame, signal_spirit::Frame);
    assert_alias!(signal_spirit::SpiritFrameBody, signal_spirit::FrameBody);
    assert_alias!(signal_spirit::State, signal_spirit::StateInput);
    assert_alias!(signal_spirit::Record, signal_spirit::RecordInput);
    assert_alias!(signal_spirit::Propose, signal_spirit::ProposeInput);
    assert_alias!(signal_spirit::Clarify, signal_spirit::ClarifyInput);
    assert_alias!(signal_spirit::Supersede, signal_spirit::SupersedeInput);
    assert_alias!(signal_spirit::Retire, signal_spirit::RetireInput);
    assert_alias!(
        signal_spirit::ResolveClarification,
        signal_spirit::ResolveClarificationInput
    );
    assert_alias!(signal_spirit::Observe, signal_spirit::ObserveInput);
    assert_alias!(
        signal_spirit::PublicTextSearch,
        signal_spirit::PublicTextSearchInput
    );
    assert_alias!(
        signal_spirit::PublicRecords,
        signal_spirit::PublicRecordsInput
    );
    assert_alias!(
        signal_spirit::PrivateRecords,
        signal_spirit::PrivateRecordsInput
    );
    assert_alias!(signal_spirit::Lookup, signal_spirit::LookupInput);
    assert_alias!(signal_spirit::Count, signal_spirit::CountInput);
    assert_alias!(
        signal_spirit::ChangeCertainty,
        signal_spirit::ChangeCertaintyInput
    );
    assert_alias!(
        signal_spirit::BumpImportance,
        signal_spirit::BumpImportanceInput
    );
    assert_alias!(
        signal_spirit::ChangeRecord,
        signal_spirit::ChangeRecordInput
    );
    assert_alias!(
        signal_spirit::RegisterReferent,
        signal_spirit::RegisterReferentInput
    );
    assert_alias!(signal_spirit::LookupStash, signal_spirit::LookupStashInput);
    assert_alias!(signal_spirit::Tap, signal_spirit::TapInput);
    assert_alias!(signal_spirit::Untap, signal_spirit::UntapInput);
    assert_alias!(
        signal_spirit::ApplyAuthorizedRecord,
        signal_spirit::ApplyAuthorizedRecordInput
    );
    assert_alias!(
        signal_spirit::SubscribeIntent,
        signal_spirit::SubscribeIntentInput
    );
    assert_alias!(
        signal_spirit::PublicIntent,
        signal_spirit::PublicIntentInput
    );

    assert_alias!(
        signal_spirit::RecordAccepted,
        signal_spirit::RecordAcceptedOutput
    );
    assert_alias!(signal_spirit::Proposed, signal_spirit::ProposedOutput);
    assert_alias!(signal_spirit::Clarified, signal_spirit::ClarifiedOutput);
    assert_alias!(signal_spirit::Superseded, signal_spirit::SupersededOutput);
    assert_alias!(signal_spirit::Retired, signal_spirit::RetiredOutput);
    assert_alias!(
        signal_spirit::ClarificationResolved,
        signal_spirit::ClarificationResolvedOutput
    );
    assert_alias!(
        signal_spirit::GuardianRejected,
        signal_spirit::GuardianRejectedOutput
    );
    assert_alias!(
        signal_spirit::ReferentGuardianRejected,
        signal_spirit::ReferentGuardianRejectedOutput
    );
    assert_alias!(
        signal_spirit::RecordsObserved,
        signal_spirit::RecordsObservedOutput
    );
    assert_alias!(
        signal_spirit::RecordsStashed,
        signal_spirit::RecordsStashedOutput
    );
    assert_alias!(signal_spirit::RecordFound, signal_spirit::RecordFoundOutput);
    assert_alias!(
        signal_spirit::RecordsCounted,
        signal_spirit::RecordsCountedOutput
    );
    assert_alias!(
        signal_spirit::CertaintyChanged,
        signal_spirit::CertaintyChangedOutput
    );
    assert_alias!(
        signal_spirit::ImportanceBumped,
        signal_spirit::ImportanceBumpedOutput
    );
    assert_alias!(
        signal_spirit::RecordChanged,
        signal_spirit::RecordChangedOutput
    );
    assert_alias!(
        signal_spirit::ReferentRegistered,
        signal_spirit::ReferentRegisteredOutput
    );
    assert_alias!(
        signal_spirit::ObservationTapped,
        signal_spirit::ObservationTappedOutput
    );
    assert_alias!(
        signal_spirit::ObservationUntapped,
        signal_spirit::ObservationUntappedOutput
    );
    assert_alias!(
        signal_spirit::SubscriptionStarted,
        signal_spirit::SubscriptionStartedOutput
    );
    assert_alias!(
        signal_spirit::VersionReported,
        signal_spirit::VersionReportedOutput
    );
    assert_alias!(
        signal_spirit::MarkerReported,
        signal_spirit::MarkerReportedOutput
    );
    assert_alias!(
        signal_spirit::RecordApplied,
        signal_spirit::RecordAppliedOutput
    );
    assert_alias!(
        signal_spirit::ApplyRefused,
        signal_spirit::ApplyRefusedOutput
    );
    assert_alias!(signal_spirit::Error, signal_spirit::ErrorOutput);
    assert_alias!(signal_spirit::Rejected, signal_spirit::RejectedOutput);
    assert_alias!(
        signal_spirit::AdvanceRefused,
        signal_spirit::AdvanceRefusedOutput
    );

    assert_alias!(
        signal_spirit::IntentRecorded,
        signal_spirit::IntentRecordedEvent
    );
    assert_alias!(
        signal_spirit::IntentClarified,
        signal_spirit::IntentClarifiedEvent
    );
    assert_alias!(
        signal_spirit::IntentSuperseded,
        signal_spirit::IntentSupersededEvent
    );
    assert_alias!(
        signal_spirit::IntentRetired,
        signal_spirit::IntentRetiredEvent
    );
    assert_alias!(signal_spirit::Partial, signal_spirit::PartialMatch);
    assert_alias!(signal_spirit::Full, signal_spirit::FullMatch);
    assert_alias!(
        signal_spirit::AnyReferent,
        signal_spirit::AnyReferentSelection
    );
    assert_alias!(
        signal_spirit::AllReferents,
        signal_spirit::AllReferentsSelection
    );
    assert_alias!(signal_spirit::AnyKeyword, signal_spirit::AnyKeywordMatch);
    assert_alias!(signal_spirit::AllKeywords, signal_spirit::AllKeywordsMatch);
    assert_alias!(
        signal_spirit::ContainsText,
        signal_spirit::ContainsTextMatch
    );
    assert_alias!(signal_spirit::Exact, signal_spirit::ExactPrivacy);
    assert_alias!(signal_spirit::AtMost, signal_spirit::AtMostPrivacy);
    assert_alias!(signal_spirit::AtLeast, signal_spirit::AtLeastPrivacy);
    assert_alias!(
        signal_spirit::ExactCertainty,
        signal_spirit::ExactCertaintySelection
    );
    assert_alias!(
        signal_spirit::AtMostCertainty,
        signal_spirit::AtMostCertaintySelection
    );
    assert_alias!(
        signal_spirit::AtLeastCertainty,
        signal_spirit::AtLeastCertaintySelection
    );
    assert_alias!(
        signal_spirit::ExactImportance,
        signal_spirit::ExactImportanceSelection
    );
    assert_alias!(
        signal_spirit::AtMostImportance,
        signal_spirit::AtMostImportanceSelection
    );
    assert_alias!(
        signal_spirit::AtLeastImportance,
        signal_spirit::AtLeastImportanceSelection
    );
}

#[cfg(feature = "nota-text")]
#[test]
fn authored_root_order_and_operation_kind_close_the_wire_inventory() {
    let source =
        schema_language::SchemaSource::from_schema_text(signal_spirit::SIGNAL_SCHEMA_SOURCE)
            .expect("decode authored signal schema");
    let input = source
        .input()
        .body()
        .as_enum()
        .expect("Input root remains an enum");
    let output = source
        .output()
        .body()
        .as_enum()
        .expect("Output root remains an enum");

    let input_names = input
        .variants()
        .iter()
        .map(|variant| variant.name().as_str())
        .collect::<Vec<_>>();
    let output_names = output
        .variants()
        .iter()
        .map(|variant| variant.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        input_names,
        INPUT_ROUTES
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        output_names,
        OUTPUT_ROUTES
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
    );

    let operation_kind = source
        .types()
        .entries()
        .iter()
        .find(|entry| entry.name().as_str() == "OperationKind")
        .expect("OperationKind declaration remains present");
    let schema_language::SourceDeclarationValue::Enum(operation_kind) = operation_kind.value()
    else {
        panic!("OperationKind remains an enum");
    };
    let operation_names = operation_kind
        .variants()
        .iter()
        .map(|variant| variant.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(operation_names, input_names);

    for closed_root in ["Watch", "Unwatch", "Remove", "CollectRemovalCandidates"] {
        assert!(!input_names.contains(&closed_root));
    }
    for closed_reply in ["RecordRemoved", "RemovalCandidatesCollected"] {
        assert!(!output_names.contains(&closed_reply));
    }
}

#[cfg(feature = "nota-text")]
#[test]
fn canonical_examples_are_generated_from_current_typed_wire_values() {
    use nota::{NotaEncode, NotaSource};
    use signal_spirit::{
        AdvanceRefusal, AdvanceRefusalReason, DomainScope, DomainScopes, RecordIdentifier,
        SearchText, Statement, StatementText, SubscriptionToken, VersionReport, VersionText,
    };

    let inputs = [
        Input::state(Statement::new(StatementText::new("capture this intent"))),
        Input::public_text_search(SearchText::new("exact public text")),
        Input::tap(signal_spirit::ObserverFilter::All),
        Input::untap(SubscriptionToken::new(3)),
        Input::Version,
        Input::Marker,
        Input::public_intent(DomainScopes::new(vec![DomainScope::All])),
    ];
    let outputs = [
        Output::record_accepted(RecordIdentifier::new("0001")),
        Output::version_reported(VersionReport::new(VersionText::new("0.12.1"))),
        Output::advance_refused(AdvanceRefusal::new(AdvanceRefusalReason::Denied)),
    ];

    for input in &inputs {
        let text = input.to_nota();
        assert_eq!(
            NotaSource::new(&text)
                .parse::<Input>()
                .expect("decode generated canonical Input example"),
            *input
        );
    }
    for output in &outputs {
        let text = output.to_nota();
        assert_eq!(
            NotaSource::new(&text)
                .parse::<Output>()
                .expect("decode generated canonical Output example"),
            *output
        );
    }

    let rendered = inputs
        .iter()
        .map(NotaEncode::to_nota)
        .chain(outputs.iter().map(NotaEncode::to_nota))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(rendered, include_str!("../examples/canonical.nota"));
}

#[cfg(feature = "nota-text")]
#[test]
fn public_text_search_crosses_text_archive_and_process_frame_boundaries() {
    use nota::{NotaEncode, NotaSource};
    use signal_frame::{ExchangeIdentifier, ExchangeLane, LaneSequence, SessionEpoch};
    use signal_spirit::SearchText;

    let input = Input::public_text_search(SearchText::new("exact public text"));
    assert_eq!(input.route(), InputRoute::PublicTextSearch);
    assert_eq!(
        OperationKind::from_input(&input),
        OperationKind::PublicTextSearch
    );
    assert_eq!(input.short_header(), 0x0008_0001_0000_0001);

    let text = input.to_nota();
    assert_eq!(text, "PublicTextSearch.(exact public text)");
    assert_eq!(
        NotaSource::new(&text)
            .parse::<Input>()
            .expect("decode PublicTextSearch NOTA"),
        input
    );

    let exchange = ExchangeIdentifier::new(
        SessionEpoch::new(7),
        ExchangeLane::Connector,
        LaneSequence::first(),
    );
    let process_frame = input.clone().into_frame(exchange);
    assert_eq!(process_frame.short_header().value(), 0x0008_0001_0000_0001);
    let process_wire = input
        .clone()
        .encode_request_frame(exchange)
        .expect("encode bound process-frame PublicTextSearch request");
    let decoded_frame = signal_spirit::ContractMarker::decode_frame(&process_wire)
        .expect("decode bound process-frame PublicTextSearch request");
    assert_eq!(decoded_frame.short_header(), process_frame.short_header());
    assert_eq!(decoded_frame.into_body(), process_frame.into_body());
    assert_eq!(
        signal_spirit::ContractMarker::decode_single_request(&process_wire)
            .expect("decode one bound PublicTextSearch request"),
        (exchange, input)
    );

    assert!(signal_spirit::SIGNAL_SCHEMA_SOURCE.contains("PublicTextSearch.PublicTextSearchInput"));
    assert!(
        signal_spirit::SIGNAL_RUST_SOURCE
            .contains("pub fn public_text_search(payload: SearchText) -> Self")
    );
}
