# signal-spirit

Ordinary signal layer for the **`spirit`** component. Successor to
`signal-persona-spirit` per intent records 765, 767, 780. The legacy
repo lives at `LiGoldragon/signal-persona-spirit` for historical
reference.

This crate owns the typed vocabulary for psyche statements,
intent-record capture, intent-record queries, intent-topic catalog
queries, and spirit subscriptions. Runtime logic lives in `spirit`.

## Schema-driven contract

The wire surface is declared in
[`schema/signal-spirit.schema`](schema/signal-spirit.schema) per
the three-part schema structure (records 746-753, /353):

```text
{}      ;; Specifying — imports / exports
[]      ;; Input header — operations accepted (in variant order)
[]      ;; Input extras
{}      ;; Namespace — user-defined type definitions
[]      ;; Output — replies / events (in variant order)
```

The schema-rust composer (in the `schema` repo) reads this file plus
the foundational `nota.schema` (in the `nota` repo, `nota-next`
branch) and emits the Rust wire types this crate exposes.

## Component triad

- **`spirit`** — daemon + thin CLI + the authored `spirit.schema`.
- **`signal-spirit`** (this repo) — ordinary signal layer.
- **`core-signal-spirit`** — privileged/control signal layer.

Per record 768: `owner-signal-*` retires to `core-signal-*` across
all triads. The privileged surface is core-named.

## Provenance

- Old: `signal-persona-spirit` v0.3 (still deployed in production).
- New: `signal-spirit` (this repo), schema-driven.
- Side-by-side; cutover lands when the new stack reaches feature
  parity.

## See also

- `ARCHITECTURE.md` — structural shape.
- `INTENT.md` — repo-scope psyche intent.
- `schema/signal-spirit.schema` — the authored contract.
- Primary workspace `AGENTS.md` — agent contract.
