# signal-spirit

Signal contract for the ordinary `spirit` surface. Version 0.14.0 defines wire
revision 2.

This crate owns the typed vocabulary for psyche statements, psyche-state
queries, uniform intent-record queries, and spirit subscriptions. Its record
payload is `Entry { Domains Kind Description Importance }`. Runtime logic lives
in `spirit`.

Default builds expose the binary rkyv frame surface only. Enable
`nota-text` for CLI/debug/audit NOTA projection.
