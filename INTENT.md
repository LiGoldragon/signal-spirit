# INTENT — signal-spirit

*The psyche's intent for the ordinary peer-callable wire contract
for `spirit`. Synthesised from primary workspace intent
records 656-696 and current contract-boundary discipline. Verbatim
psyche quotes in italics where the exact wording is load-bearing;
surrounding prose is agent-composed. Companion to `ARCHITECTURE.md`
and `AGENTS.md`. Maintenance: `skills/repo-intent.md` and
`skills/intent-manifestation.md`.*

## What this crate is

`signal-spirit` is the **ordinary peer-callable** wire
contract for `spirit`. It carries the vocabulary for
submitting psyche statements, observing psyche state, observing
intent records, and subscribing to those streams. Meta-policy
lifecycle/configuration orders live in the sibling meta contract;
runtime actors, sockets, storage, classifier logic, and mind
forwarding live in `spirit`.

This is the active renamed contract. The older
`signal-persona-spirit` name is retired for this surface; consumers
should depend on `signal-spirit`.

## Channel boundary

Per record 668: *"when the psyche describes a major part of the
system, that description IS a warrant to create a schema for that
part."* The ordinary channel is one such part. In the current
implementation this contract expresses the channel through the
current `signal_frame::signal_channel! { ... }` declaration, not
the retired schema-form / box-form dual-emission scaffold. The
meta-policy channel is separate. **One channel = one
contract.** Daemons may hold multiple internal plane schemas;
contract crates like this stay single-channel because the crate
boundary already enforces the channel boundary.

## Description-only discipline

The record shape carries one agent-clarified `Description`, a
`Kind`, required certainty, daemon-stamped time, and one or more
user-creatable topic strings. Verbatim/context payloads from
earlier shapes are gone. The wire reply to a `Record` operation is
`(RecordAccepted N)` — terse; no echo of the submitted content.
Daemon-stamped timestamps: clients do not supply capture time. Any
new topic word a `Record` uses is registered at the wire layer; no
pre-declared enum. Topic queries match membership in the entry's
topic vector, either as no topic filter, partial one-or-more topic
matching, or full every-topic matching. Certainty is the shared
`Magnitude` scale: `Zero` means confidence withdrawn and nominated
for removal in Spirit, while `Minimum` remains weak but real intent.
Privacy is a second directional `Magnitude` axis, not a named tier
enum: `Zero` means open/public intent, and higher magnitudes narrow
the intended audience. Public observation types have no privacy
field and select exact `Zero` privacy by type; elevated reads use
explicit privacy-scoped variants carrying a `PrivacySelection`.
Observation filters can select no certainty filter, exact certainty,
at-most certainty, or at-least certainty. Removal candidate review
uses exact `Zero` certainty. The ordinary maintenance operation
`ChangeCertainty` changes an existing record's certainty, including
lowering it to `Zero` for review without deleting it.
`ChangeRecord` changes an existing record's user-authored entry in place
while preserving the daemon-minted identifier and daemon-owned
provenance, so a mistaken record can be corrected without remove-and-
recreate.
Candidate collection is a separate explicit maintenance operation:
`CollectRemovalCandidates` selects exact-`Zero` records, preserves
compact `RecordSummary` archive material in a sema archive database,
or returns it through an explicit print target, and only then allows
the runtime to retract those records from the hot store.

Historical storage migration shapes belong in this contract crate when
the daemon needs to read prior production records. The v0.3.0 shape is
captured under `migration::v030` and projects into the current
privacy-aware shape with `privacy = Zero`; this keeps the stored-data
bridge typed and reviewable rather than making `spirit` guess at
old row layouts.

Record observation can also filter by daemon-stamped capture time:
any time, inclusive time range, since a recorded moment, until a
recorded moment, or qualitative recency depth. Qualitative depths
(`Shallow`, `Recent`, `Deep`, `VeryDeep`) are deliberately query-local:
the daemon applies topic/kind/certainty filters first, then keeps the
newest matching records at the requested depth, so a quiet topic naturally
reaches farther back than a busy topic without making agents choose exact
counts or time windows.

## Goals

- Carry the **ordinary peer-callable contract** for psyche-state
  observation, intent-record submission/observation, and
  subscription lifecycle.
- Honour the **single-channel-per-crate** boundary: meta-policy
  orders live in a separate contract.
- Keep the wire surface on the current `signal-frame` stack:
  contract-local operation roots, typed frame aliases, streams, and
  observable `Tap`/`Untap` support.
- Keep the default dependency graph binary-first: rkyv frame types are
  always available, while the NOTA projection is an explicit
  `nota-text` feature for CLI/debug/audit edges.
- Project the **description-only discipline** — terse
  acknowledgements, daemon-stamped timestamps, and user-creatable
  topic vectors.
- Carry daemon startup policy that must be binary-configured before the
  daemon starts, including `AuthorizationMode`: `Gating` keeps criome
  verdicts fail-closed for fan-out, while `Observing` emits criome
  authorization requests and lets the local head proceed for monitoring.

## Constraints

- The macro-injected `Tap(ObserverFilter)` /
  `Untap(ObserverSubscriptionToken)` verbs are mandatory on the
  ordinary socket per the component observability
  discipline. Domain-specific `Watch`/`Unwatch` for psyche-state
  and intent-record streams is a separate surface and coexists
  without collision.
- Wire reply shapes stay terse — no verbatim echo of submitted
  content. Record identifiers in replies are daemon-minted opaque
  random values rendered as lowercase base36, not reusable ordinal
  row numbers. Exact identifier lookup remains available; ranges over
  identifiers are removed because history windows belong to recorded-time
  filters.
- Record identifiers are opaque lowercase base36 codes minted by
  `spirit`. The production display/query surface uses the shortest
  collision-free four-to-seven-character code; the underlying wire type stays
  wide enough to decode older long identifier codes during migration.

## Principles

- **Wire vocabulary uses contract-local verbs.** `State` not
  `Statement`, `Record` not `Entry`-as-a-verb, `Observe` not
  `Observation`. Per the verb-form rule in `intent/naming.nota`
  19:45Z.
- **Daemon lowering boundary.** This crate owns only contract operations on the
  wire. Component commands, SEMA reads/writes, storage classification, and
  executable payloads live inside the `spirit` daemon.
- **No compatibility scaffold for retired schema emission.**
  The public root-level `Operation`/`Reply`/`Event` types are the
contract surface. The retired qualified dual path and box-form test
  surface are not restored.

## Open intent

- Cross-crate schema-import resolution (the same deferral the
  primary explainer documents) — when the resolver lands, the
  spirit daemon's actor schemas can `(Import spirit [...])`
  from this crate directly rather than carry hand-written type
  duplicates.

## See also

- `ARCHITECTURE.md` — daemon lowering boundary + wire vocabulary.
- `src/lib.rs` — payload types plus the current explicit
  `signal_channel!` declaration.
- Primary workspace: `repos/spirit/INTENT.md` — the
  daemon-side schema-driven actor architecture.
