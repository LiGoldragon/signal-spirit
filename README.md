# signal-spirit

Signal contract for the ordinary `spirit` surface. Version 0.14.0 defines wire
revision 2.

This crate owns the typed vocabulary for psyche statements, psyche-state
queries, uniform intent-record queries, and spirit subscriptions. Its record
payload is `Entry { Domains Kind Description Importance }`. Runtime logic lives
in `spirit`.

Default builds expose the binary rkyv frame surface only. Enable
`nota-text` for CLI/debug/audit NOTA projection.

`examples/canonical.nota` contains objects, not a shell-option grammar. The
public `spirit` CLI accepts exactly one ordinary object (a bare `Version` or
`Marker` is an object); paths and Unix flags are not contract inputs. The
authoritative root list is the schema, not a generated help command.
