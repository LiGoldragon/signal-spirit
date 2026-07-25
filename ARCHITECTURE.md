# signal-spirit — architecture

*Ordinary Signal contract for the psyche-facing Spirit surface.*

## Role

`signal-spirit` is the peer-callable contract for
`spirit`. It carries the vocabulary for submitting psyche statements,
observing psyche state, observing intent records, and subscribing to those
intent events.

This repo carries the active ordinary Spirit contract. The
`signal-persona-spirit` name is retired for this surface.

Meta-policy lifecycle/configuration orders live in the sibling meta
contract. Runtime actors, sockets, storage, classifier logic, and mind
forwarding live in `spirit`.

## Direction

`signal-spirit` is the **ordinary peer-callable wire contract** for `spirit`. Its explicit goals: carry the ordinary contract for psyche-state observation, intent-record submission/observation, and subscription lifecycle; honour the single-channel-per-crate boundary; keep the wire surface on the current `signal-frame` stack; and stay binary-first with NOTA projection behind an explicit `nota-text` feature.

The `Entry` shape enforces **description-only discipline**: typed `Domains`, one
agent-clarified `Description`, `Kind`, required certainty, importance, privacy,
and typed referents. Capture time is not client-authored Entry data. Wire
replies are terse; no verbatim echo of submitted content.

Domain taxonomy types are consumed from the shared `signal-domain` contract and re-exported through `signal_spirit::Domain`, `signal_spirit::schema::domain::*`, and `signal_spirit::schema::signal::*` compatibility paths. `signal-spirit` does not own or regenerate the taxonomy.

Privacy is a second directional `Magnitude` axis, not a named tier enum: `Zero` means open/public, higher magnitudes narrow the audience. `Tap`/`Untap` is the ordinary observation lifecycle, and `SubscribeIntent` opens typed `IntentEvent` delivery. The current schema has no `Watch`/`Unwatch` roots and no Stream or Family declarations.

Daemon startup carries `AuthorizationMode`: `Gating` keeps criome verdicts fail-closed for fan-out; `Observing` emits criome authorization requests and lets the local head proceed for monitoring.

Under cluster authorization, a head-advancing operation the cluster does not grant is refused to the caller as `AdvanceRefused(AdvanceRefusal)` with a closed `AdvanceRefusalReason`: `Denied` — criome reached a terminal deny; `Expired` — the authorization window closed before the quorum completed; `Unavailable` — no operational quorum contract exists (the unfounded-criome loud refusal); `Unreachable` — the local criome could not be reached or its session went dead. The schema language carries no comment syntax, so these reason meanings live here. `AdvanceRefusal` is the intake-gate vocabulary and is distinct from the peer-apply ingress `ApplyRefusal`; the two contact points keep their own closed types.

## Contract/Daemon Boundary

This contract owns only the ordinary public wire vocabulary. The
`spirit` daemon lowers those operations into its own Nexus commands,
SEMA reads or writes, effects, rejections, replies, and observer events.

```text
contract Operation  ->  daemon Nexus/SEMA/effect work
wire vocabulary         daemon executable boundary
```

**Contract operations on the wire (this crate).**
The ordinary contract uses the 25 contract-local verbs enumerated in the
Contract Surface table below. Reads are expressed by `Observe`, the public
read roots, `Lookup`, `Count`, and `LookupStash`; mutations are expressed by
the record, clarification, retirement, certainty, importance, referent, and
authorized-apply roots.

The ordinary socket deliberately has no delete operation. `Remove` and
`CollectRemovalCandidates` are not Input roots, and their former reply roots
are absent. Removal authority belongs to the owner-only meta surface. Removal
record types retained in the vocabulary do not grant ordinary callers an
operation.

Apply the verb-form rule per `intent/naming.nota` 19:45Z:
`State` not `Statement`, `Record` not `Entry`-as-a-verb, `Observe` not
`Observation`.

**`Tap`/`Untap` observability.** The ordinary socket carries
`Tap(ObserverFilter)` and `Untap(SubscriptionToken)`. Intent subscription is
the distinct `SubscribeIntent(Query)` root and produces typed `IntentEvent`
delivery; it does not introduce a parallel schema Stream declaration.

**Component commands (spirit daemon).** The spirit
daemon owns its typed Command enum plus a `CommandExecutor` that knows
the spirit tables. Executable payloads do not live in this contract.

The public intent event stays contract-owned as `IntentEvent`. It does not
carry `SemaObservation` or depend on `signal-sema`.

**Frame layer.** Frame mechanics come from `signal-frame`.

**Daemon startup configuration.** The binary
`SpiritDaemonConfiguration` also carries daemon startup policy that must be set
before process launch. `AuthorizationMode` is explicit: `Gating` means criome
verdicts release or hold fan-out, while `Observing` means spirit emits the
criome authorization request but proceeds without waiting for the verdict.

**Text projection.** The default build is binary/rkyv-only and does not pull
`nota`, `nota-codec`, or `signal-core`. The `nota-text` feature enables
generated Nota derives, the `schema-language` codec integration, and text
round-trip tests for CLI/debug/audit edges. This crate owns no parallel text
decoder. Daemon consumers use the default graph.

References:
- `primary/skills/contract-repo.md` §"Public contracts use contract-local operation verbs"

## Contract Surface

| Operation | Payload |
|---|---|
| `State` | `Statement` |
| `Record` | `RecordRequest` |
| `Propose` | `Proposal` |
| `Clarify` | `Clarification` |
| `Supersede` | `Supersession` |
| `Retire` | `Retirement` |
| `ResolveClarification` | `ClarificationResolution` |
| `Observe` | `Query` |
| `PublicTextSearch` | `SearchText` |
| `PublicRecords` | `RecordSelection` |
| `PrivateRecords` | `RecordSelection` |
| `Lookup` | `RecordIdentifier` |
| `Count` | `Query` |
| `ChangeCertainty` | `CertaintyChange` |
| `BumpImportance` | `ImportanceBump` |
| `ChangeRecord` | `RecordChange` |
| `RegisterReferent` | `ReferentRegistration` |
| `LookupStash` | `StashHandle` |
| `Tap` | `ObserverFilter` |
| `Untap` | `SubscriptionToken` |
| `ApplyAuthorizedRecord` | `AuthorizedRecordApplication` |
| `SubscribeIntent` | `Query` |
| `Version` | unit |
| `Marker` | unit |
| `PublicIntent` | `DomainScopes` |

The wire form carries the contract-local verb only. Database classes and store
effects are daemon-owned lowering, not public operation roots, event payloads,
or dependencies of this crate.

## Constraints

| Constraint | Witness |
|---|---|
| Every request variant is a contract-local verb in the frozen wire order. | `wire_inventory::authored_root_order_and_operation_kind_close_the_wire_inventory` compares all 25 authored Input roots with `OperationKind`; `complete_route_header_and_tag_inventory_is_stable` pins every route, short header, and archived route tag. |
| Agent-facing public intent lookup hides low-level query plumbing. | `PublicIntent(DomainScopes)` carries schema-backed domain selections and validates them with the same non-empty `DomainScopes` rule used by domain matches. |
| Public text lookup retains its exact process boundary. | `wire_inventory::public_text_search_crosses_text_archive_and_process_frame_boundaries` proves the authored root, generated constructor, canonical NOTA, route, short header, archive frame, and signal-frame exchange. |
| The ordinary working socket has no removal authority. | The complete Input/Output inventory rejects `Remove`, `CollectRemovalCandidates`, `RecordRemoved`, and `RemovalCandidatesCollected`; removal belongs to the owner-only meta surface. |
| Retired schema Stream and Family constructs do not return through Help. | `generated_help_model_renders_every_decoded_schema_target` proves every current decoded target renders, while `IntentEventStream` is an explicit unknown target and `TrueSchema` contains no retired stream relation. |
| Intent entries can be nominated for removal without deletion. | `ChangeCertainty(CertaintyChange)` round-trips through RKYV and NOTA and returns `CertaintyChanged`; setting certainty to `Zero` makes the record visible to removal-candidate review. |
| Intent entries can be corrected in place without remove-and-recreate. | `ChangeRecord(RecordChange)` returns `RecordChanged(RecordChangeReceipt)` while keeping removal off the ordinary surface. |
| Default consumers stay binary-only. | `default_dependency_tree_does_not_pull_text_or_legacy_signal_crates` proves the default normal dependency graph has no `nota`, `nota-codec`, or `signal-core`; `nota_text_feature_is_the_only_text_projection_opt_in` proves `nota` appears only when requested. |
| All-feature consumers resolve one immutable schema world. | `all_features_resolve_one_exact_schema_and_nota_world` checks every producer pin, the absence of alternate Nota URLs, patches, branch/path sources, and an empty duplicate tree. |
| Checked-in examples describe the active typed contract. | `wire_inventory::canonical_examples_are_generated_from_current_typed_wire_values` generates every example from real `Input`/`Output` values and decodes each line back through Nota. |
| Domain taxonomy is shared, not duplicated. | `public_domain_paths_are_signal_domain_types` proves the public `signal-spirit` domain paths are the `signal-domain` types, and `public_domain_path_round_trips_through_rkyv` / `public_domain_path_round_trips_through_nota` keep representative codec compatibility covered. |
| A refused head advance surfaces a typed reason and new routes never move existing ones. | `generated_advance_refused_frame_round_trips_without_moving_existing_routes` round-trips every `AdvanceRefusalReason` variant through the signal frame and pins the appended `OUTPUT_ADVANCE_REFUSED` short header beside the unchanged `ApplyRefused`/`Rejected` headers. |
| Established public Rust payload nouns remain compatible. | `established_public_payload_names_remain_exact_type_aliases` checks every public compatibility name against the distinct generated schema payload type. |
| This crate contains no runtime. | Source has no Kameo, Tokio, sockets, database engine, or sema-engine code. |

## Code Map

```text
src/lib.rs              — generated schema re-exports, compatibility aliases/helpers, and StreamingFrame aliases
src/schema/domain.rs    — compatibility shim re-exporting signal-domain schema types
schema/signal.schema    — Spirit wire schema importing signal-domain taxonomy types
tests/generated_contract.rs — frame, Help, domain compatibility, and NOTA witnesses
tests/wire_inventory.rs — complete route/header/tag, public-alias, closed-root, and process-boundary witnesses
tests/validation.rs     — contract validation witnesses
examples/canonical.nota — generated current-contract Input/Output examples
```
