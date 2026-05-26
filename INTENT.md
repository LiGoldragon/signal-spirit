# INTENT — signal-spirit

*The psyche's intent for `signal-spirit`, synthesised from Spirit
records that apply to this repo. Verbatim psyche quotes in italics
where the exact wording is load-bearing. Companion to
`ARCHITECTURE.md` and `AGENTS.md`. Maintenance:
`skills/repo-intent.md`.*

## Repo-scope only

Per record 717: this file carries only the intent that is FOR
`signal-spirit`. Workspace-shape intent stays in the primary
workspace `/INTENT.md`.

## Why this repo exists

*Record 765 directive:* "Rename away unnecessary persona ancestry
for Spirit-facing repositories and surfaces … signal-spirit as its
ordinary signal layer."

*Record 767 concrete mapping:* "signal-persona-spirit becomes
signal-spirit (the ordinary signal layer for spirit)."

*Record 780 authorization (Maximum):* Three new repos for the
persona-prefix retirement, side-by-side with the existing
persona-prefixed triad. Contract-focused at this stage; the
schema-driven typed surface lives here.

## The ordinary signal layer

*The shape principle (records 765-766):* core-signal-* carries the
important control and library layer; **signal-*** is the ordinary
messaging layer for safer callers and extensions. This repo IS the
ordinary layer for the spirit component — the everyday API surface
that any peer can call.

Privileged operations — supervisor start/drain, identity
registration, bootstrap-policy reload, upgrade handover — do NOT
land here. Those live in `core-signal-spirit` per the same record.

## Schema-driven from the start

*The schema-driven directive (records 746-753):* "Take schema all
the way back to NOTA itself." The signal surface is no exception —
it is declared in `schema/signal-spirit.schema` and the
schema-rust composer emits the Rust. The hand-authored Rust in
`src/` shrinks toward the bootstrap kernel boundary.

This contrasts with the legacy `signal-persona-spirit` crate, which
hand-authored the entire wire surface and was the consumer (not the
declarer) of the schema. The new repo inverts that: the schema IS
the contract; the Rust is emitted.

## No Features-section drift

*The retraction (records 713-715, 730-732):* schema declares data
types only. No `EffectTable`, `FanOutTargets`, `StorageDescriptor`
as authored schema content. The legacy `signal-persona-spirit`
namespace shows the right shape (typed records + enums + payloads);
the retracted drift never lived here, and is forbidden from
returning.

## Positional, not labeled

*The hard override:* NOTA records are positional, not labeled.
Every payload type declared in `schema/signal-spirit.schema` IS a
positional record. `(key value)` records are not NOTA.

## Topics are user-creatable strings

*Carried from the legacy v0.3 deployment intent:* topics in
intent-record entries are user-creatable strings, not a pre-declared
enum. The schema declares `Topic [String]`; any new topic word a
`Record` operation uses is registered. The vocabulary grows as
psyche statements introduce new topics.

## References

- Spirit records **765, 766, 767, 768, 780** — persona-prefix
  retirement + the core/signal split discipline.
- Spirit records **746-753** — schema-driven NOTA design.
- Spirit records **713-715, 730-732** — Features retraction.
- Spirit record **717** — file-ownership discipline.
- Primary `/353`, `/354`, `/355`.
- Companion files: `ARCHITECTURE.md`, `schema/signal-spirit.schema`.
- Legacy: `LiGoldragon/signal-persona-spirit` — historical reference
  for the v0.3 wire shape this repo succeeds.
