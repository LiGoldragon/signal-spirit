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

## Direction

`signal-spirit` is the **ordinary peer-callable wire contract** for `spirit`. Its explicit goals: carry the ordinary contract for psyche-state observation, intent-record submission/observation, and subscription lifecycle; honour the single-channel-per-crate boundary; keep the wire surface on the current `signal-frame` stack; and stay binary-first with NOTA projection behind an explicit `nota-text` feature.

The record shape enforces **description-only discipline**: one agent-clarified `Description`, a `Kind`, required certainty, daemon-stamped time, and user-creatable topic strings. Daemon-stamped timestamps only — clients never supply capture time. Wire replies are terse; no verbatim echo of submitted content.

Privacy is a second directional `Magnitude` axis, not a named tier enum: `Zero` means open/public, higher magnitudes narrow the audience. The mandatory `Tap`/`Untap` observable surface is injected by `signal_channel!`; the domain-specific `Watch`/`Unwatch` pairs for psyche-state and intent-record streams coexist without collision.

Daemon startup carries `AuthorizationMode`: `Gating` keeps criome verdicts fail-closed for fan-out; `Observing` emits criome authorization requests and lets the local head proceed for monitoring.

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
- `PublicIntent` (agent-facing public intent lookup by schema-backed
  `DomainScope` selections; the daemon expands ancestors and dedupes records),
- `Watch` / `Unwatch` (domain-specific subscriptions — payload names
  which stream class to open).
- `Remove` (intent-store maintenance — payload is the `RecordIdentifier`
  to delete from the daemon-owned store).
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

**Daemon startup configuration.** The binary
`SpiritDaemonConfiguration` also carries daemon startup policy that must be set
before process launch. `AuthorizationMode` is explicit: `Gating` means criome
verdicts release or hold fan-out, while `Observing` means spirit emits the
criome authorization request but proceeds without waiting for the verdict.

**Text projection.** The default build is binary/rkyv-only and does not pull
`nota`, `nota-codec`, or `signal-core`. The `nota-text` feature enables
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
| `PublicIntent` | `DomainScopes` |
| `Watch` | `Subscription` stream selector |
| `Unwatch` | `SubscriptionToken` |
| `Remove` | `RecordIdentifier` |
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
| Agent-facing public intent lookup hides low-level query plumbing. | `PublicIntent(DomainScopes)` carries only schema-backed domain selections; the daemon expands each requested path to include the exact path plus top-level and ancestor `All` scopes, dedupes shared ancestors, and returns deterministic public records. |
| Intent entries can be removed explicitly by identifier. | `Remove(RecordIdentifier)` round-trips through RKYV and NOTA and returns `RecordRemoved`; production identifiers are opaque lowercase base36 codes minted by `spirit`, normally rendered at the shortest collision-free four-to-seven-character length while the wire type remains wide enough to decode older long codes. |
| Intent entries can be corrected in place without remove-and-recreate. | `ChangeRecord(RecordChange)` round-trips through RKYV and NOTA and returns `RecordMutationApplied`; the daemon replaces the user-authored `Entry` fields under the same `RecordIdentifier` while preserving daemon-owned provenance. |
| Removal-candidate collection is owner-side maintenance. | `CollectRemovalCandidates` is not an ordinary public working verb in this contract surface. |
| Historical storage migration shapes stay contract-owned and explicit. | `tests/migration.rs` projects a v0.3.0 `migration::v030::Entry` and `migration::v030::Operation::Record` into the current privacy-aware shape with `privacy = Zero`, proving the daemon can read the prior production row shape without guessing at bytes. |
| Agents can inspect the intent-topic catalog without reading every entry. | `Observation::Topics` returns `TopicsObserved` with one `TopicCount` per topic membership. |
| Every submitted entry is one top-level psyche statement without client-provided capture time. | `Entry` carries one or more domains, kind, description, importance, and privacy. Accepted records are accepted; uncertainty is not an active public entry field. |
| Spirit never accepts client-provided timestamps on `Record` requests. | `record_request_with_client_timestamp_shape_is_rejected` and `record_request_with_parenthesized_client_date_time_shape_is_rejected` fail old timestamp-bearing input shapes. |
| Capture time appears only in daemon-produced provenance. | `RecordProvenance` carries one bare `YYYY-MM-DD` date field and one bare `HH:MM:SS` time field. |
| Record identifiers are output-only. | `RecordIdentifier` appears in descriptions/provenance replies, not in `Entry`; `spirit` mints it from randomness, not from row position. |
| Database classification is daemon-side only; no Sema payloads appear on the wire. | `EffectEmitted` carries contract-owned `operation` and `outcome` fields, and `spirit_contract_has_no_sema_classification_dependency_or_roots` guards the dependency and head set. |
| Default consumers stay binary-only. | `default_dependency_tree_does_not_pull_text_or_legacy_signal_crates` proves the default normal dependency graph has no `nota`, `nota-codec`, or `signal-core`; `nota_text_feature_is_the_only_text_projection_opt_in` proves `nota` appears only when requested. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, database engine, or sema-engine code. |

## Code Map

```text
src/lib.rs              — request/reply/event records and explicit signal_channel! declaration
src/migration.rs        — historical contract shapes and projection bridges for store migrations
examples/canonical.nota — canonical NOTA examples
tests/round_trip.rs     — rkyv frame, NOTA, verb, and stream witnesses
tests/migration.rs      — prior-version projection witnesses
```
