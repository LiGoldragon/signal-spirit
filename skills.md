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

- The wire carries exactly the contract-local Input roots declared in
  `schema/signal.schema`. `Tap` / `Untap` own ordinary operation/effect
  observation, while `SubscribeIntent(Query)` opens typed `IntentEvent`
  delivery. The active schema has no `Watch` / `Unwatch` roots and no Stream
  or Family declaration.
- `Entry` is one top-level statement without client-provided capture time.
  It carries typed domains, kind, description, certainty, importance, privacy,
  and referents.
  Restatement is represented by repeated `Entry` records, not by nesting
  vectors. Certainty is required `Magnitude`: `Zero` nominates a
  record for removal, while `Minimum` remains weak but real intent.
  Privacy is also required `Magnitude`: `Zero` is open/public, and
  higher magnitudes narrow the intended audience.
- Capture time appears only in daemon-produced provenance as a bare
  `YYYY-MM-DD` date field and a bare `HH:MM:SS` time field.
- `RecordIdentifier` is minted by `spirit` and returned to callers, then reused
  by typed lookup, clarification, supersession, retirement, certainty,
  importance, and record-change payloads.
- `ChangeCertainty(CertaintyChange)` is the ordinary mutate-shaped
  maintenance verb for replacing an existing record's certainty; `Zero`
  is the review-nomination value, not a delete operation by itself.
- `ChangeRecord(RecordChange)` is the ordinary mutate-shaped maintenance
  verb for replacing an existing record's user-authored `Entry` fields
  while the daemon preserves the `RecordIdentifier` and provenance.
- The ordinary working socket has no removal operation.
  `Remove` / `CollectRemovalCandidates` and their former reply roots are
  closed. Removal authority belongs to the owner-only meta surface. Retained
  removal record types are data vocabulary, not ordinary caller authority.
- Historical migration modules are part of the contract surface when a
  daemon store migration needs the prior production row shape. Keep them
  explicit, version-named (`migration::v030`), and tested; they may
  project into the current contract, but they must not contain daemon
  runtime, sockets, actors, storage, or classifier logic.
- `Observe`-shaped operations stay public read verbs; the durable read plan is
  daemon-owned.
- `Tap` / `Untap` carry ordinary observer lifecycle without exposing a
  Sema-class root.
- Intent observation is description-first unless the caller asks for
  provenance.
- Intent observations select domains with `Any`, `Partial`, or `Full`.
- Intent observations can filter required `Magnitude` certainty with
  `Any`, `Exact`, `AtMost`, or `AtLeast`. Removal-candidate review is
  the exact `Zero` query.
- Public intent observations cannot carry a privacy selector and mean exact
  `Zero` privacy. Elevated records must be requested through explicit
  privacy-scoped observation variants carrying `PrivacySelection`.
- `Query` carries only the sealed domain, keyword, text, referent, kind,
  privacy, certainty, and importance selectors declared by the active schema.
  `PublicIntent(DomainScopes)` and `PublicTextSearch(SearchText)` are the
  purpose-built public lookup roots.
- Exact opaque records are selected with `Lookup(RecordIdentifier)`;
  identifier ranges are not part of the contract.
- Mandatory `Tap`/`Untap` observability surface is part of the
  contract per component observability discipline.
- Default builds must stay binary/rkyv-only: no `nota`, no
  `nota-codec`, and no `signal-core` in normal dependencies. Enable
  `nota-text` only at CLI/debug/audit edges.
