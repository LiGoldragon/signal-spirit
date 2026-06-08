# signal-spirit — architecture

*Ordinary Signal contract for the psyche-facing Spirit surface.*

## Role

`signal-spirit` is the peer-callable contract for
`spirit`. It carries the vocabulary for submitting psyche statements,
observing psyche state, observing intent records, and subscribing to those
streams.

This repo carries the active ordinary Spirit contract. The
`signal-persona-spirit` name is retired for this surface.

Meta-policy lifecycle/configuration orders live in the sibling meta
contract. Runtime actors, sockets, storage, classifier logic, and mind
forwarding live in `spirit`.

## Contract/Daemon Boundary

This contract owns only the ordinary public wire vocabulary. The
`spirit` daemon lowers those operations into its own Nexus commands,
SEMA reads or writes, effects, rejections, replies, and observer events.

```text
contract Operation  ->  daemon Nexus/SEMA/effect work
wire vocabulary         daemon executable boundary
```

**Contract operations on the wire (this crate).**
The ordinary contract uses contract-local verbs:
- `State` (the psyche stating intent, payload `Statement`),
- `Record` (an agent submitting a typed intent entry without capture time,
  payload `Entry`),
- `Observe` (the read side — payload is a closed `Observation` enum
  naming `State`, `Records`, `Topics`, `QuestionsPending`, etc.),
- `Watch` / `Unwatch` (domain-specific subscriptions — payload names
  which stream class to open).
- `Remove` (intent-store maintenance — payload is the `RecordIdentifier`
  to delete from the daemon-owned store).
- `ChangeCertainty` (intent-store maintenance — payload is a
  `CertaintyChange` naming the record and replacement certainty).
- `ChangeRecord` (intent-store maintenance — payload is a
  `RecordChange` naming the record and replacement entry).
- `CollectRemovalCandidates` (intent-store maintenance — payload is a
  `RemovalCandidateCollection` that selects exact-`Zero` candidates and
  names an output target before daemon-side retraction).

Apply the verb-form rule per `intent/naming.nota` 19:45Z:
`State` not `Statement`, `Record` not `Entry`-as-a-verb, `Observe` not
`Observation`.

**Mandatory `Tap`/`Untap` observability.** Spirit's observable surface is
standardized.
The macro-injected `Tap(ObserverFilter)` /
`Untap(ObserverSubscriptionToken)` verbs are mandatory on the
ordinary socket. The domain-specific `Watch`/`Unwatch` for psyche-
state and intent-record streams is a separate surface and coexists
without collision (spirit's domain doesn't use `Tap` as a verb).

**Component commands (spirit daemon).** The spirit
daemon owns its typed Command enum plus a `CommandExecutor` that knows
the spirit tables. Executable payloads do not live in this contract.

The public observer event stays contract-owned:
`EffectEmitted { operation, outcome }`. It does not carry
`SemaObservation` or depend on `signal-sema`.

**Frame layer.** Frame mechanics come from `signal-frame`.

**Text projection.** The default build is binary/rkyv-only and does not pull
`nota-next`, `nota-codec`, or `signal-core`. The `nota-text` feature enables
NOTA derives, manual NOTA codecs, and text round-trip tests for CLI/debug/audit
edges. Daemon consumers use the default graph.

References:
- `primary/skills/contract-repo.md` §"Public contracts use contract-local operation verbs"

The generic observable event record is `EffectEmitted`, matching the
current architecture where generic observers see the effect publication
moment through the contract-owned operation/outcome pair rather than
an executable daemon effect record or database-classification payload.

## Contract Surface

| Operation | Payload |
|---|---|
| `State` | `Statement` |
| `Record` | `Entry` without date/time |
| `Observe` | closed `Observation` enum |
| `Watch` | `Subscription` stream selector |
| `Unwatch` | `SubscriptionToken` |
| `Remove` | `RecordIdentifier` |
| `ChangeCertainty` | `CertaintyChange` |
| `ChangeRecord` | `RecordChange` |
| `CollectRemovalCandidates` | `RemovalCandidateCollection` |
| `Tap` | `ObserverFilter` |
| `Untap` | `ObserverSubscriptionToken` |

The wire form carries the contract-local verb only. Database classes and store
effects are daemon-owned lowering, not public operation roots, event payloads,
or dependencies of this crate.

## Constraints

| Constraint | Witness |
|---|---|
| Every request variant is a contract-local verb in verb form. | `round_trip.rs` asserts each variant's NOTA head. |
| Subscribe-shaped variants declare stream relations. | `signal_channel!` stream blocks bind subscribe/open/event/close. |
| Retract-shaped close variants have typed close acknowledgements. | `SubscriptionRetracted` carries the typed `SubscriptionToken` sum and round-trips through RKYV and NOTA. |
| Intent queries return compact summaries unless provenance is requested. | `ObservationMode::SummaryOnly` is the explicit query mode used in canonical examples. |
| Intent record queries support the agent-useful filters needed for intent work. | `PublicRecordQuery` carries `TopicSelection` (`Any`, `Partial`, `Full`), optional `kind`, `CertaintySelection` (`Any`, `Exact`, `AtMost`, `AtLeast`), `RecordedTimeSelection` (`Any`, `Between`, `Since`, `Until`, `Recent`, `Shallow`, `Deep`, `VeryDeep`), and description/provenance mode; it has no privacy field and means exact-`Zero` privacy. `RecordIdentifierQuery` selects one opaque identifier exactly; identifier ranges are intentionally absent because random identifiers do not carry recency or ordinal meaning. `PrivacyScopedRecordQuery` and `PrivacyScopedRecordIdentifierQuery` are explicit elevated read shapes carrying `PrivacySelection` (`Any`, `Exact`, `AtMost`, `AtLeast`). `RecordQuery` remains the full maintenance/internal query shape for candidate collection and daemon projection. |
| Intent entries can be removed explicitly by identifier. | `Remove(RecordIdentifier)` round-trips through RKYV and NOTA and returns `RecordRemoved`; production identifiers are opaque lowercase base36 codes minted by `spirit`, normally rendered at the shortest collision-free four-to-seven-character length while the wire type remains wide enough to decode older long codes. |
| Intent entries can be nominated for removal without deletion. | `ChangeCertainty(CertaintyChange)` round-trips through RKYV and NOTA and returns `CertaintyChanged`; setting certainty to `Zero` makes the record visible to removal-candidate review. |
| Intent entries can be corrected in place without remove-and-recreate. | `ChangeRecord(RecordChange)` round-trips through RKYV and NOTA and returns `RecordMutationApplied`; the daemon replaces the user-authored `Entry` fields under the same `RecordIdentifier` while preserving daemon-owned provenance. |
| Removal-candidate collection is explicit capture-before-retract maintenance. | `CollectRemovalCandidates(RemovalCandidateCollection)` round-trips through RKYV and NOTA, requires exact-`Zero` certainty and exact-`Zero` privacy by contract method, carries `OutputTarget::ArchiveDatabase(Default)` for the daemon-derived archive database, `OutputTarget::ArchiveDatabase(Path(ArchivePath))` for an explicit archive database path, or `OutputTarget::Print(OutputStream)` for client-rendered compact material, and returns `RemovalCandidatesCollected` with compact `RecordSummary` archive material plus removed identifiers and skipped candidates. |
| Historical storage migration shapes stay contract-owned and explicit. | `tests/migration.rs` projects a v0.3.0 `migration::v030::Entry` and `migration::v030::Operation::Record` into the current privacy-aware shape with `privacy = Zero`, proving the daemon can read the prior production row shape without guessing at bytes. |
| Agents can inspect the intent-topic catalog without reading every entry. | `Observation::Topics` returns `TopicsObserved` with one `TopicCount` per topic membership. |
| Every submitted entry is one top-level psyche statement without client-provided capture time. | `Entry` carries one or more topics, kind, description, required `Magnitude` certainty, and required `Magnitude` privacy; repeated entries are the restatement signal. |
| Spirit never accepts client-provided timestamps on `Record` requests. | `record_request_with_client_timestamp_shape_is_rejected` and `record_request_with_parenthesized_client_date_time_shape_is_rejected` fail old timestamp-bearing input shapes. |
| Capture time appears only in daemon-produced provenance. | `RecordProvenance` carries one bare `YYYY-MM-DD` date field and one bare `HH:MM:SS` time field. |
| Record identifiers are output-only. | `RecordIdentifier` appears in descriptions/provenance replies, not in `Entry`; `spirit` mints it from randomness, not from row position. |
| Database classification is daemon-side only; no Sema payloads appear on the wire. | `EffectEmitted` carries contract-owned `operation` and `outcome` fields, and `spirit_contract_has_no_sema_classification_dependency_or_roots` guards the dependency and head set. |
| Default consumers stay binary-only. | `default_dependency_tree_does_not_pull_text_or_legacy_signal_crates` proves the default normal dependency graph has no `nota-next`, `nota-codec`, or `signal-core`; `nota_text_feature_is_the_only_text_projection_opt_in` proves `nota-next` appears only when requested. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, database engine, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and explicit signal_channel! declaration
src/migration.rs        — historical contract shapes and projection bridges for store migrations
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
tests/migration.rs      — prior-version projection witnesses
```
