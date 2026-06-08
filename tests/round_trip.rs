#![cfg(feature = "nota-text")]

use nota_next::{NotaDecode, NotaEncode, NotaSource};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply as FrameReply, RequestPayload,
    SessionEpoch, SignalOperationHeads, StreamEventIdentifier, StreamingFrameBody, SubReply,
    SubscriptionTokenInner,
};
use signal_spirit::{
    CertaintyChange, CertaintyChanged, CertaintySelection, Date, Description, EffectEmitted,
    EffectOutcome, Entry, Event, FocusArea, Frame, FrameBody, Kind, Magnitude, Observation,
    ObservationMode, ObserverFilter, ObserverFilterMatch, ObserverSubscriptionToken, Operation,
    OperationKind, OperationReceived, Presence, PresenceView, PrivacyScopedRecordIdentifierQuery,
    PrivacyScopedRecordQuery, PrivacySelection, PublicRecordQuery, QuestionIdentifier,
    QuestionSummary, QuestionText, QuestionsObserved, RecordAccepted, RecordCaptured, RecordChange,
    RecordIdentifier, RecordIdentifierQuery, RecordIdentifierSelection, RecordMutationApplied,
    RecordProvenance, RecordProvenancesObserved, RecordQuery, RecordRemoved, RecordSubscription,
    RecordSubscriptionToken, RecordedTime, RecordedTimeRange, RecordedTimeSelection,
    RecordsObserved, RemovalCandidateCollection, RemovalCandidatesCollected, Reply,
    RequestUnimplemented, StateChanged, StateObserved, StateSubscriptionToken, Statement,
    StatementText, Subscription, SubscriptionOpened, SubscriptionRetracted, SubscriptionSnapshot,
    SubscriptionToken, Time, Topic, TopicCount, TopicSelection, Topics, TopicsObserved,
    UnimplementedReason,
};

const CANONICAL: &str = include_str!("../examples/canonical.nota");

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn description() -> signal_spirit::RecordSummary {
    signal_spirit::RecordSummary {
        identifier: RecordIdentifier::new(1),
        topics: Topics::single(Topic::new("workspace")),
        kind: Kind::Decision,
        description: Description::new("description only"),
        certainty: Magnitude::Maximum,
        privacy: Magnitude::Zero,
    }
}

fn candidate_description() -> signal_spirit::RecordSummary {
    signal_spirit::RecordSummary {
        identifier: RecordIdentifier::new(1),
        topics: Topics::single(Topic::new("workspace")),
        kind: Kind::Correction,
        description: Description::new("candidate description"),
        certainty: Magnitude::Zero,
        privacy: Magnitude::Zero,
    }
}

fn provenance() -> RecordProvenance {
    RecordProvenance {
        summary: description(),
        date: Date::new(2026, 5, 20),
        time: Time::new(14, 30, 0),
    }
}

fn entry() -> Entry {
    Entry {
        topics: Topics::single(Topic::new("workspace")),
        kind: Kind::Decision,
        description: Description::new("description only"),
        certainty: Magnitude::Maximum,
        privacy: Magnitude::Zero,
    }
}

fn state() -> PresenceView {
    PresenceView {
        presence: Presence::Active,
        focus: Some(FocusArea::new("implementation")),
    }
}

fn round_trip_request(request: Operation) -> Operation {
    let frame = Frame::new(FrameBody::Request {
        exchange: exchange(),
        request: request.clone().into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: Reply) -> Reply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: FrameReply::committed(NonEmpty::single(SubReply::Ok(reply))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            FrameReply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted reply, got {other:?}"),
        },
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let encoded = value.to_nota();
    assert_eq!(encoded, expected);

    let recovered = NotaSource::new(&encoded)
        .parse::<T>()
        .expect("decode nota text");
    assert_eq!(recovered, value);
    assert!(
        CANONICAL.contains(expected),
        "examples/canonical.nota missing line: {expected}"
    );
}

fn decode_only_nota<T>(text: &str, expected: T)
where
    T: NotaDecode + PartialEq + std::fmt::Debug,
{
    let recovered = NotaSource::new(text)
        .parse::<T>()
        .expect("decode nota text");
    assert_eq!(recovered, expected);
}

#[test]
fn spirit_requests_round_trip() {
    let requests = [
        Operation::State(Statement {
            text: StatementText::new("capture this intent"),
        }),
        Operation::Record(entry()),
        Operation::Observe(Observation::State),
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::any(),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
        Operation::Observe(Observation::PrivateRecords(PrivacyScopedRecordQuery::new(
            PrivacySelection::AtMost(Magnitude::High),
            PublicRecordQuery::any(ObservationMode::SummaryOnly),
        ))),
        Operation::Observe(Observation::RecordIdentifiers(RecordIdentifierQuery::new(
            RecordIdentifierSelection::Exact(RecordIdentifier::new(1)),
            ObservationMode::SummaryOnly,
        ))),
        Operation::Observe(Observation::PrivateRecordIdentifiers(
            PrivacyScopedRecordIdentifierQuery::new(
                PrivacySelection::AtMost(Magnitude::High),
                RecordIdentifierQuery::new(
                    RecordIdentifierSelection::Exact(RecordIdentifier::new(1)),
                    ObservationMode::SummaryOnly,
                ),
            ),
        )),
        Operation::Observe(Observation::Topics),
        Operation::Observe(Observation::Questions),
        Operation::Watch(Subscription::State),
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        Operation::Unwatch(SubscriptionToken::State(StateSubscriptionToken {
            identifier: 1,
        })),
        Operation::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        Operation::Remove(RecordIdentifier::new(1)),
        Operation::ChangeCertainty(CertaintyChange {
            identifier: RecordIdentifier::new(1),
            certainty: Magnitude::Zero,
        }),
        Operation::ChangeRecord(RecordChange {
            record_identifier: RecordIdentifier::new(1),
            entry: entry(),
        }),
        Operation::CollectRemovalCandidates(RemovalCandidateCollection::default_archive_database()),
        Operation::Tap(ObserverFilter::OperationsOnly),
        Operation::Untap(ObserverSubscriptionToken::new(SubscriptionTokenInner::new(
            3,
        ))),
    ];

    for request in requests {
        assert_eq!(round_trip_request(request.clone()), request);
    }
}

#[test]
fn spirit_replies_round_trip() {
    let replies = [
        Reply::RecordAccepted(RecordAccepted::new(RecordIdentifier::new(1))),
        Reply::RecordRemoved(RecordRemoved::new(RecordIdentifier::new(1))),
        Reply::CertaintyChanged(CertaintyChanged {
            identifier: RecordIdentifier::new(1),
            certainty: Magnitude::Zero,
        }),
        Reply::RecordMutationApplied(RecordMutationApplied::new(RecordIdentifier::new(1))),
        Reply::RemovalCandidatesCollected(RemovalCandidatesCollected::new(
            vec![candidate_description()],
            vec![RecordIdentifier::new(1)],
            Vec::new(),
        )),
        Reply::StateObserved(StateObserved::new(state())),
        Reply::RecordsObserved(RecordsObserved::new(vec![description()])),
        Reply::RecordProvenancesObserved(RecordProvenancesObserved::new(vec![provenance()])),
        Reply::TopicsObserved(TopicsObserved::new(vec![TopicCount {
            topic: Topic::new("workspace"),
            entries: 2,
        }])),
        Reply::QuestionsObserved(QuestionsObserved::new(vec![QuestionSummary {
            identifier: QuestionIdentifier::new("question-one"),
            question: QuestionText::new("which intent wins?"),
        }])),
        Reply::SubscriptionOpened(SubscriptionOpened {
            token: SubscriptionToken::State(StateSubscriptionToken { identifier: 1 }),
            snapshot: SubscriptionSnapshot::State(state()),
        }),
        Reply::SubscriptionOpened(SubscriptionOpened {
            token: SubscriptionToken::Records(RecordSubscriptionToken { identifier: 2 }),
            snapshot: SubscriptionSnapshot::Records(vec![description()]),
        }),
        Reply::SubscriptionRetracted(SubscriptionRetracted {
            token: SubscriptionToken::State(StateSubscriptionToken { identifier: 1 }),
        }),
        Reply::SubscriptionRetracted(SubscriptionRetracted {
            token: SubscriptionToken::Records(RecordSubscriptionToken { identifier: 2 }),
        }),
        Reply::RequestUnimplemented(RequestUnimplemented {
            reason: UnimplementedReason::NotBuiltYet,
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn legacy_description_only_input_decodes_as_summary_only() {
    let operation =
        NotaSource::new("(Observe (RecordIdentifiers ((Exact [00t9]) DescriptionOnly)))")
            .parse::<Operation>()
            .expect("legacy mode decodes");

    assert_eq!(
        operation,
        Operation::Observe(Observation::RecordIdentifiers(RecordIdentifierQuery::new(
            RecordIdentifierSelection::Exact(RecordIdentifier::new(1053)),
            ObservationMode::SummaryOnly,
        )))
    );
}

#[test]
fn spirit_reply_payloads_convert_through_macro_generated_from_impls() {
    let reply: Reply = RecordAccepted::new(RecordIdentifier::new(1)).into();

    assert_eq!(
        reply,
        Reply::RecordAccepted(RecordAccepted::new(RecordIdentifier::new(1)))
    );
}

#[test]
fn spirit_events_round_trip() {
    let events = [
        Event::StateChanged(StateChanged { state: state() }),
        Event::RecordCaptured(RecordCaptured {
            record: description(),
        }),
        Event::OperationReceived(OperationReceived {
            operation: OperationKind::Record,
        }),
        Event::EffectEmitted(EffectEmitted {
            operation: OperationKind::Record,
            outcome: EffectOutcome::RecordCaptured,
        }),
    ];

    for event in events {
        let frame = Frame::new(StreamingFrameBody::SubscriptionEvent {
            event_identifier: StreamEventIdentifier::new(
                SessionEpoch::new(1),
                ExchangeLane::Acceptor,
                LaneSequence::first(),
            ),
            token: SubscriptionTokenInner::new(1),
            event: event.clone(),
        });
        let bytes = frame.encode_length_prefixed().expect("encode");
        let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
        match decoded.into_body() {
            FrameBody::SubscriptionEvent { event: decoded, .. } => assert_eq!(decoded, event),
            other => panic!("expected event frame, got {other:?}"),
        }
    }
}

#[test]
fn spirit_request_exposes_contract_owned_kind() {
    assert_eq!(
        Operation::State(Statement {
            text: StatementText::new("capture this intent"),
        })
        .kind(),
        OperationKind::State
    );
    assert_eq!(Operation::Record(entry()).kind(), OperationKind::Record);
    assert_eq!(
        Operation::Remove(RecordIdentifier::new(1)).kind(),
        OperationKind::Remove
    );
    assert_eq!(
        Operation::ChangeCertainty(CertaintyChange {
            identifier: RecordIdentifier::new(1),
            certainty: Magnitude::Zero,
        })
        .kind(),
        OperationKind::ChangeCertainty
    );
    assert_eq!(
        Operation::CollectRemovalCandidates(RemovalCandidateCollection::default_archive_database())
            .kind(),
        OperationKind::CollectRemovalCandidates
    );
    assert_eq!(
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        }))
        .kind(),
        OperationKind::Watch
    );
}

#[test]
fn spirit_contract_has_no_sema_classification_dependency_or_roots() {
    let manifest = include_str!("../Cargo.toml");
    assert!(
        !manifest.contains("signal-sema"),
        "ordinary signal contracts must not depend on signal-sema for public wire vocabulary"
    );

    let source = include_str!("../src/lib.rs");
    assert!(
        !source.contains("SemaObservation"),
        "EffectEmitted must stay a contract-owned operation/outcome event, not a SemaObservation payload"
    );

    let heads = <Operation as SignalOperationHeads>::HEADS;
    for forbidden in [
        "Assert",
        "Mutate",
        "Retract",
        "Match",
        "Subscribe",
        "Validate",
    ] {
        assert!(
            !heads.contains(&forbidden),
            "Sema classification root {forbidden} must not appear on the public spirit wire"
        );
    }
}

#[test]
fn spirit_stream_witnesses_are_emitted() {
    assert_eq!(
        Operation::Watch(Subscription::State).opened_stream(),
        Some(signal_spirit::StreamKind::DomainStream)
    );
    assert_eq!(
        Event::RecordCaptured(RecordCaptured {
            record: description()
        })
        .stream_kind(),
        signal_spirit::StreamKind::DomainStream
    );
    assert_eq!(
        Operation::Unwatch(SubscriptionToken::State(StateSubscriptionToken {
            identifier: 1
        }))
        .closed_stream(),
        Some(signal_spirit::StreamKind::DomainStream)
    );
    assert_eq!(
        Operation::Tap(ObserverFilter::All).opened_stream(),
        Some(signal_spirit::StreamKind::ObserverStream)
    );
}

#[test]
fn spirit_observer_filter_routes_operation_and_effect_events() {
    let operation = OperationReceived {
        operation: OperationKind::Record,
    };
    let effect = EffectEmitted {
        operation: OperationKind::Record,
        outcome: EffectOutcome::RecordCaptured,
    };

    assert!(ObserverFilter::All.matches_operation_received(&operation));
    assert!(ObserverFilter::All.matches_effect_emitted(&effect));
    assert!(ObserverFilter::OperationsOnly.matches_operation_received(&operation));
    assert!(!ObserverFilter::OperationsOnly.matches_effect_emitted(&effect));
    assert!(!ObserverFilter::EffectsOnly.matches_operation_received(&operation));
    assert!(ObserverFilter::EffectsOnly.matches_effect_emitted(&effect));
}

#[test]
fn spirit_canonical_examples_round_trip() {
    round_trip_nota(
        Operation::State(Statement {
            text: StatementText::new("capture this intent"),
        }),
        "(State ([capture this intent]))",
    );
    round_trip_nota(
        Operation::Record(entry()),
        "(Record ([workspace] Decision [description only] Maximum Zero))",
    );
    let mut high_entry = entry();
    high_entry.description = Description::new("high description");
    high_entry.certainty = Magnitude::High;
    round_trip_nota(
        Operation::Record(high_entry),
        "(Record ([workspace] Decision [high description] High Zero))",
    );
    let mut candidate_entry = entry();
    candidate_entry.kind = Kind::Correction;
    candidate_entry.description = Description::new("candidate description");
    candidate_entry.certainty = Magnitude::Zero;
    round_trip_nota(
        Operation::Record(candidate_entry.clone()),
        "(Record ([workspace] Correction [candidate description] Zero Zero))",
    );
    let mut multi_topic_entry = entry();
    multi_topic_entry.topics = Topics::new(vec![Topic::new("spirit"), Topic::new("nota")]);
    multi_topic_entry.description = Description::new("multi topic");
    round_trip_nota(
        Operation::Record(multi_topic_entry),
        "(Record ([spirit nota] Decision [multi topic] Maximum Zero))",
    );
    round_trip_nota(Operation::Observe(Observation::State), "(Observe State)");
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::any(),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Any []) None Any Any SummaryOnly)))",
    );
    decode_only_nota(
        "(Observe (Records ((Any []) None Any Any (Exact Zero) SummaryOnly)))",
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::any(),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
    );
    decode_only_nota(
        "(Observe (Records ((Any []) None SummaryOnly)))",
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::any(),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
    );
    round_trip_nota(
        Operation::Observe(Observation::PrivateRecords(PrivacyScopedRecordQuery::new(
            PrivacySelection::AtMost(Magnitude::High),
            PublicRecordQuery {
                topic_selection: TopicSelection::any(),
                kind: None,
                certainty_selection: CertaintySelection::Any,
                recorded_time_selection: RecordedTimeSelection::Recent,
                mode: ObservationMode::SummaryOnly,
            },
        ))),
        "(Observe (PrivateRecords ((AtMost High) ((Any []) None Any Recent SummaryOnly))))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![Topic::new("workspace")]),
            kind: Some(Kind::Decision),
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Partial [workspace]) (Some Decision) Any Any SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![
                Topic::new("spirit"),
                Topic::new("nota"),
            ]),
            kind: None,
            certainty_selection: CertaintySelection::AtMost(Magnitude::Low),
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Partial [spirit nota]) None (AtMost Low) Any SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::full(vec![Topic::new("spirit"), Topic::new("nota")]),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Any,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Full [spirit nota]) None Any Any SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![Topic::new("spirit")]),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Between(RecordedTimeRange::new(
                RecordedTime::new(Date::new(2026, 5, 29), Time::new(0, 0, 0)),
                RecordedTime::new(Date::new(2026, 5, 30), Time::new(23, 59, 59)),
            )),
            mode: ObservationMode::WithProvenance,
        })),
        "(Observe (Records ((Partial [spirit]) None Any (Between ((2026-05-29 00:00:00) (2026-05-30 23:59:59))) WithProvenance)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![Topic::new("spirit")]),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Recent,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Partial [spirit]) None Any Recent SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![Topic::new("spirit")]),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Shallow,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Partial [spirit]) None Any Shallow SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![Topic::new("spirit")]),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::Deep,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Partial [spirit]) None Any Deep SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery {
            topic_selection: TopicSelection::partial(vec![Topic::new("spirit")]),
            kind: None,
            certainty_selection: CertaintySelection::Any,
            recorded_time_selection: RecordedTimeSelection::VeryDeep,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Observe (Records ((Partial [spirit]) None Any VeryDeep SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::Records(PublicRecordQuery::removal_candidates(
            ObservationMode::WithProvenance,
        ))),
        "(Observe (Records ((Any []) None (Exact Zero) Any WithProvenance)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::RecordIdentifiers(RecordIdentifierQuery::new(
            RecordIdentifierSelection::Exact(RecordIdentifier::new(1053)),
            ObservationMode::SummaryOnly,
        ))),
        "(Observe (RecordIdentifiers ((Exact [00t9]) SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Observe(Observation::PrivateRecordIdentifiers(
            PrivacyScopedRecordIdentifierQuery::new(
                PrivacySelection::AtMost(Magnitude::High),
                RecordIdentifierQuery::new(
                    RecordIdentifierSelection::Exact(RecordIdentifier::new(1053)),
                    ObservationMode::SummaryOnly,
                ),
            ),
        )),
        "(Observe (PrivateRecordIdentifiers ((AtMost High) ((Exact [00t9]) SummaryOnly))))",
    );
    round_trip_nota(Operation::Observe(Observation::Topics), "(Observe Topics)");
    round_trip_nota(
        Operation::Observe(Observation::Questions),
        "(Observe Questions)",
    );
    round_trip_nota(Operation::Watch(Subscription::State), "(Watch State)");
    round_trip_nota(
        Operation::Watch(Subscription::Records(RecordSubscription {
            topic: None,
            mode: ObservationMode::SummaryOnly,
        })),
        "(Watch (Records (None SummaryOnly)))",
    );
    round_trip_nota(
        Operation::Unwatch(SubscriptionToken::Records(RecordSubscriptionToken {
            identifier: 2,
        })),
        "(Unwatch (Records (2)))",
    );
    round_trip_nota(
        Operation::Remove(RecordIdentifier::new(1)),
        "(Remove [0001])",
    );
    round_trip_nota(
        Operation::ChangeCertainty(CertaintyChange {
            identifier: RecordIdentifier::new(1),
            certainty: Magnitude::Zero,
        }),
        "(ChangeCertainty ([0001] Zero))",
    );
    round_trip_nota(
        Operation::ChangeRecord(RecordChange {
            record_identifier: RecordIdentifier::new(1),
            entry: entry(),
        }),
        "(ChangeRecord ([0001] ([workspace] Decision [description only] Maximum Zero)))",
    );
    round_trip_nota(
        Operation::CollectRemovalCandidates(RemovalCandidateCollection::default_archive_database()),
        "(CollectRemovalCandidates (((Any []) None (Exact Zero) Any (Exact Zero) SummaryOnly) (ArchiveDatabase Default)))",
    );
    round_trip_nota(
        Operation::CollectRemovalCandidates(RemovalCandidateCollection::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            signal_spirit::OutputTarget::archive_database("/tmp/spirit-removal-candidates.sema"),
        )),
        "(CollectRemovalCandidates (((Any []) None (Exact Zero) Any (Exact Zero) SummaryOnly) (ArchiveDatabase (Path [/tmp/spirit-removal-candidates.sema]))))",
    );
    round_trip_nota(
        Operation::CollectRemovalCandidates(RemovalCandidateCollection::print_standard_output()),
        "(CollectRemovalCandidates (((Any []) None (Exact Zero) Any (Exact Zero) SummaryOnly) (Print StandardOutput)))",
    );
    round_trip_nota(
        Operation::CollectRemovalCandidates(RemovalCandidateCollection::print_standard_error()),
        "(CollectRemovalCandidates (((Any []) None (Exact Zero) Any (Exact Zero) SummaryOnly) (Print StandardError)))",
    );
    round_trip_nota(
        Reply::RecordAccepted(RecordAccepted::new(RecordIdentifier::new(1))),
        "(RecordAccepted [0001])",
    );
    round_trip_nota(
        Reply::RecordRemoved(RecordRemoved::new(RecordIdentifier::new(1))),
        "(RecordRemoved [0001])",
    );
    round_trip_nota(
        Reply::CertaintyChanged(CertaintyChanged {
            identifier: RecordIdentifier::new(1),
            certainty: Magnitude::Zero,
        }),
        "(CertaintyChanged ([0001] Zero))",
    );
    round_trip_nota(
        Reply::RecordMutationApplied(RecordMutationApplied::new(RecordIdentifier::new(1))),
        "(RecordMutationApplied [0001])",
    );
    round_trip_nota(
        Reply::RemovalCandidatesCollected(RemovalCandidatesCollected::new(
            vec![candidate_description()],
            vec![RecordIdentifier::new(1)],
            Vec::new(),
        )),
        "(RemovalCandidatesCollected ([([0001] [workspace] Correction [candidate description] Zero Zero)] [[0001]] []))",
    );
    round_trip_nota(
        Reply::StateObserved(StateObserved::new(state())),
        "(StateObserved (Active (Some [implementation])))",
    );
    round_trip_nota(
        Reply::RecordsObserved(RecordsObserved::new(vec![description()])),
        "(RecordsObserved [([0001] [workspace] Decision [description only] Maximum Zero)])",
    );
    round_trip_nota(
        Reply::RecordProvenancesObserved(RecordProvenancesObserved::new(vec![provenance()])),
        "(RecordProvenancesObserved [(([0001] [workspace] Decision [description only] Maximum Zero) 2026-05-20 14:30:00)])",
    );
    round_trip_nota(
        Reply::TopicsObserved(TopicsObserved::new(vec![TopicCount {
            topic: Topic::new("workspace"),
            entries: 2,
        }])),
        "(TopicsObserved [([workspace] 2)])",
    );
    round_trip_nota(
        Reply::QuestionsObserved(QuestionsObserved::new(vec![QuestionSummary {
            identifier: QuestionIdentifier::new("question-one"),
            question: QuestionText::new("which intent wins?"),
        }])),
        "(QuestionsObserved [([question-one] [which intent wins?])])",
    );
    round_trip_nota(
        Event::RecordCaptured(RecordCaptured {
            record: description(),
        }),
        "(RecordCaptured (([0001] [workspace] Decision [description only] Maximum Zero)))",
    );
    round_trip_nota(
        Event::EffectEmitted(EffectEmitted {
            operation: OperationKind::Record,
            outcome: EffectOutcome::RecordCaptured,
        }),
        "(EffectEmitted (Record RecordCaptured))",
    );
}

#[test]
fn record_request_with_client_timestamp_shape_is_rejected() {
    NotaSource::new("(Record ([workspace] Decision [description only] Maximum 1779000000))")
        .parse::<Operation>()
        .expect_err("client timestamp must not decode");
}

#[test]
fn record_request_with_duplicate_topics_is_rejected() {
    let error = NotaSource::new("(Record ([spirit spirit] Decision [duplicate] Maximum))")
        .parse::<Operation>()
        .expect_err("duplicate topics must not decode");

    assert!(
        error.to_string().contains("record repeats topic spirit"),
        "unexpected error: {error}"
    );
}

#[test]
fn record_request_with_empty_topics_is_rejected() {
    let error = NotaSource::new("(Record ([] Decision [missing topic] Maximum))")
        .parse::<Operation>()
        .expect_err("empty topics must not decode");

    assert!(
        error
            .to_string()
            .contains("record must carry at least one topic"),
        "unexpected error: {error}"
    );
}

#[test]
fn record_request_with_parenthesized_client_date_time_shape_is_rejected() {
    NotaSource::new(
        "(Record ([workspace] Decision [description only] Maximum (2026 5 20) (14 30 0)))",
    )
    .parse::<Operation>()
    .expect_err("parenthesized client date/time must not decode");
}
