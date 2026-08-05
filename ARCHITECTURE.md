# signal-spirit architecture

`signal-spirit` is the ordinary peer-callable contract for `spirit`. It owns
no runtime policy, actor, socket, persistence, or migration behavior. Those
remain in `spirit`; meta orders remain in `meta-signal-spirit`; admission
verdicts remain in `signal-spirit-judge`.

## Source-to-artifact pipeline

`spirit-ethos::INTERFACE` is the only authored ordinary contract input.
`build.rs` validates its sealed batch configuration through `nomos-engine` and
emits one Interface Rust artifact into `OUT_DIR`. `src/lib.rs` exposes that
artifact directly. Universal identities are rendered as the canonical encoded
Rust identifiers prescribed by production Rust Logos; readable compatibility
aliases are not introduced.

The Ethos Interface declares the ordinary inputs, outputs, refusals, values,
and observer stream. `Entry` remains the four-field value
`{ Domains Kind Description Importance }`; `signal-domain` supplies its
shared `Domain` and `DomainScopes` types. The source package, not this crate,
is authoritative for the complete closed operation inventory and ordering.

## Boundaries

- There is no local `.schema` input or generated `src/schema` projection.
- `dotos-text` is the optional Dotos projection; binary-only users select
  `--no-default-features`.
- Dotos resolves from one full immutable source revision and rkyv resolves
  exactly to 0.8.17, so the archive ABI has one package world.
- The contract does not provide legacy operation decoders, alternate arities,
  defaults, or name aliases.

## Proof obligations

- building the crate regenerates the ordinary artifact from sealed Ethos;
- source-boundary tests reject restoration of the local schema projection;
- the locked dependency test rejects a short Dotos source identity and asserts
  one current Dotos and rkyv package identity;
- both all-feature and binary-only Nix checks compile the same generated
  artifact.
