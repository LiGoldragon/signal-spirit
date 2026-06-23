# skills — signal-spirit

Read this before editing the ordinary spirit contract.

## Required Context

- `~/primary/skills/contract-repo.md`
- `~/primary/skills/component-triad.md`
- `~/primary/skills/architectural-truth-tests.md`
- `~/primary/skills/nix-discipline.md`
- this repo's `ARCHITECTURE.md`

## Boundary

This crate owns only the ordinary `spirit` Signal vocabulary. It has no
runtime, no actors, no sockets, no storage, and no classifier logic.

## Invariants

- The wire carries contract-local verbs in verb form (`State`,
  `Record`, `Observe`, `Watch`, `Unwatch`, plus mandatory `Tap` /
  `Untap`). The daemon owns typed component commands and SEMA reads/writes;
  this contract does not mirror database-action classes.
- `Entry` is one top-level statement without client-provided capture time.
  It carries one or more user-created topic strings; topic filters match
  membership in that topic vector.
  Restatement is represented by repeated `Entry` records, not by nesting
  vectors. Certainty is required `Magnitude`: `Zero` nominates a
  record for removal, while `Minimum` remains weak but real intent.
  Privacy is also required `Magnitude`: `Zero` is open/public, and
  higher magnitudes narrow the intended audience.
- Capture time appears only in daemon-produced provenance as a bare
  `YYYY-MM-DD` date field and a bare `HH:MM:SS` time field.
- `RecordIdentifier` is output-only and minted by `spirit`; production
  identifiers render as shortest collision-free lowercase base36 codes from
  four to seven characters, while older long codes remain decodable.
- `ChangeCertainty(CertaintyChange)` is the ordinary mutate-shaped
  maintenance verb for replacing an existing record's certainty; `Zero`
  is the review-nomination value, not a delete operation by itself.
- `ChangeRecord(RecordChange)` is the ordinary mutate-shaped maintenance
  verb for replacing an existing record's user-authored `Entry` fields
  while the daemon preserves the `RecordIdentifier` and provenance.
- `CollectRemovalCandidates(RemovalCandidateCollection)` is the ordinary
  capture-before-retract maintenance verb for reviewed records. It must
  be constrained to exact-`Zero` certainty and exact-`Zero` privacy
  candidates, preserve compact `RecordSummary` material through a sema
  archive database selected by `OutputTarget::ArchiveDatabase` or return
  it to the client through `OutputTarget::Print`, and return
  `RemovalCandidatesCollected`.
- Historical migration modules are part of the contract surface when a
  daemon store migration needs the prior production row shape. Keep them
  explicit, version-named (`migration::v030`), and tested; they may
  project into the current contract, but they must not contain daemon
  runtime, sockets, actors, storage, or classifier logic.
- `Observe`-shaped operations stay public read verbs; the durable read plan is
  daemon-owned.
- Stream-open variants (domain `Watch` and mandatory `Tap`) carry explicit
  stream relations without exposing a Sema-class root.
- Stream-close variants (domain `Unwatch` and mandatory `Untap`) close typed
  streams without exposing a Sema-class root.
- Intent observation is description-first unless the caller asks for
  provenance.
- Intent observations can select all topics with `Any`, one-or-more
  requested topics with `Partial`, or every requested topic with
  `Full`.
- Intent observations can filter required `Magnitude` certainty with
  `Any`, `Exact`, `AtMost`, or `AtLeast`. Removal-candidate review is
  the exact `Zero` query.
- Public intent observations cannot carry a privacy selector and mean exact
  `Zero` privacy. Elevated records must be requested through explicit
  privacy-scoped observation variants carrying `PrivacySelection`.
- Intent observations can filter daemon-stamped capture time with
  `RecordedTimeSelection`: `Any`, `Between`, `Since`, `Until`,
  `Recent`, `Shallow`, `Deep`, or `VeryDeep`. The qualitative recency
  depths are interpreted by the daemon after the topic, kind, and
  certainty filters have already narrowed the candidate set.
- Intent observations can select records by exact opaque
  `RecordIdentifier` through `Observation::RecordIdentifiers`.
  Identifier ranges are not part of the random-identifier era;
  agents use `RecordedTimeSelection` for recency/history windows.
- Intent entries are removed through the ordinary `Remove` verb by
  `RecordIdentifier`; this is intent-store maintenance, not owner
  lifecycle policy.
- Mandatory `Tap`/`Untap` observability surface is part of the
  contract per component observability discipline.
- Default builds must stay binary/rkyv-only: no `nota`, no
  `nota-codec`, and no `signal-core` in normal dependencies. Enable
  `nota-text` only at CLI/debug/audit edges.
