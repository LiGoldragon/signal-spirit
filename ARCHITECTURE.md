# signal-spirit architecture

`signal-spirit` is the ordinary peer-callable contract for `spirit`. Version
0.14.0 defines wire revision 2. Revision 1 frames and payload shapes are not
accepted or upgraded online.

The crate owns vocabulary only. Runtime actors, sockets, persistence,
classification, authorization, query execution, and migration live in
`spirit`. Meta-policy orders live in `meta-signal-spirit`; admission verdicts
live in `signal-spirit-judge`.

## Record shape

`Entry` has exactly four fields, in order:

```text
Entry { Domains Kind Description Importance }
```

It is one top-level statement. Clients do not supply capture time or record
identifiers. `spirit` mints identifiers and owns provenance.

## Ordinary surface

The revision-2 read roots are uniform over all live records:

| Operation | Payload |
|---|---|
| `Observe` | `Query` |
| `Intent` | `DomainScopes` |
| `TextSearch` | `SearchText` |
| `Lookup` | `RecordIdentifier` |
| `Count` | `Query` |

`Query` has exactly five predicates in order: `DomainMatch`, `KeywordMatch`,
`TextMatch`, `SelectedKind`, and `ImportanceSelection`. It selects domains,
description keywords or text, kind, and importance.
`TextSearch` searches descriptions. `Intent` retains the domain-scope
shorthand. Any ordering policy is daemon-owned.

Write and lifecycle roots are `State`, `Record`, `Propose`, `Clarify`,
`ResolveClarification`, `Supersede`, `Retire`, `BumpImportance`, and
`ChangeRecord`. `ApplyAuthorizedRecord` accepts only revision-2 v14 record
bodies. `SubscribeIntent`, `Tap`, and `Untap` carry the stream and observation
lifecycle. `Version`, `Marker`, and `LookupStash` retain their existing roles.

## Boundaries

- `schema/signal.schema` is authoritative; `src/schema/signal.rs` is generated.
- Shared domains are imported from `signal-domain` and re-exported.
- Default builds are binary/rkyv-only. `nota-text` is an explicit CLI,
  diagnostic, and audit projection.
- The public `spirit` executable accepts one inline ordinary object. Bare
  selectors are objects; flags and file paths are not alternate grammar.
- `SpiritDaemonConfiguration` is a separate stable archive and is not coupled
  to the breaking ordinary wire revision.
- Dense route discriminants are permitted in revision 2. No compatibility
  variants, alternate arities, or invented defaults are exposed.

## Proof obligations

- generated artifacts equal the authoritative schema;
- four-field entries and revision-2 operations round-trip through rkyv and the
  optional text projection;
- revision-1 operations and seven-field entries fail to decode;
- the active schema and generated contract reject removed vocabulary;
- the default dependency graph contains no text codec or runtime component.
