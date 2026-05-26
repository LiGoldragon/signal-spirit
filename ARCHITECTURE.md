# signal-spirit — architecture

*Ordinary signal contract for the `spirit` component. Successor to
`signal-persona-spirit` per the persona-prefix retirement (records
765, 767, 780). Schema-driven from the start.*

## §1 Role

`signal-spirit` carries the **ordinary** (non-privileged) wire
vocabulary for the `spirit` component. Any peer can construct a
`signal-spirit` operation and ask the spirit daemon to act on it.
The contract covers:

- **Psyche statements** — `State` — the raw free-form input.
- **Intent records** — `Record` — typed capture of a psyche
  statement; this is the dense storage shape.
- **Observation queries** — `Observe` — read intent records, the
  topic catalog, recent state.
- **Subscriptions** — `Watch` / `Unwatch` — long-lived streams of
  events.

Privileged operations (start / drain / identity registration /
bootstrap-policy reload / upgrade handover) live in
`core-signal-spirit`, **not here**.

## §2 Schema-driven contract

The wire surface is declared in `schema/signal-spirit.schema` per
the three-part schema structure (records 746-753):

```text
{}      ;; Specifying
[]      ;; Input header — operations
[]      ;; Input extras
{}      ;; Namespace
[]      ;; Output — replies / events
```

The schema-rust composer reads this file plus `nota.schema` and
emits the Rust wire types. The hand-authored Rust in `src/` shrinks
toward zero as composer output covers it.

## §3 What is in scope

- `Operation` enum — `State`, `Record`, `Observe`, `Watch`, `Unwatch`.
- `Reply` enum — `RecordAccepted`, `RecordsObserved`,
  `RecordProvenancesObserved`, `TopicsObserved`,
  `SubscriptionOpened`, `SubscriptionRetracted`,
  `RequestUnimplemented`.
- `Event` enum — `RecordCaptured` and any future replicas the
  daemon may emit.
- Every payload type — `Entry`, `Topic`, `Description`, `Kind`,
  `Magnitude` (imported from `signal-sema`), `Observation`,
  `Subscription`, `SubscriptionToken`, etc.
- Codec ergonomics for the wire (length-prefixed transport framing
  is the `signal-frame` repo's job).

## §4 What is NOT in scope

- **Owner / core surface.** That lives in `core-signal-spirit`.
- **Daemon runtime.** That lives in `spirit`.
- **CLI.** Bundled into the `spirit` repo as the thin daemon
  client.
- **Features section** — schema declares data types only (records
  730-732). No `EffectTable`, `FanOutTargets`, `StorageDescriptor`.
- **Labeled-record vocabulary** — NOTA records are positional, not
  labeled (workspace hard override).

## §5 Boundaries

| Component | Relationship |
|---|---|
| `spirit` | The daemon that serves operations of this contract. |
| `core-signal-spirit` | Sibling contract carrying the privileged surface. |
| `signal-frame` | Length-prefixed transport framing. |
| `signal-sema` | `Magnitude` import for certainty/intensity. |
| `nota` | Wire encoding via NOTA + the foundational `nota.schema`. |
| `schema` | The composer that turns `signal-spirit.schema` into Rust. |

## §6 References

- Spirit records **765, 767, 780** — persona-prefix retirement; new
  triad repos.
- Spirit records **746-753** (Maximum) — schema-driven NOTA design.
- Spirit records **713-715, 730-732** (Maximum) — Features
  retraction; schema declares data types only.
- Primary `/353`, `/354`, `/355` — design vision, prototype, critique.
- `INTENT.md` — repo-scope psyche intent.
- `schema/signal-spirit.schema` — the authored contract.
- Legacy: `LiGoldragon/signal-persona-spirit` — historical reference.
