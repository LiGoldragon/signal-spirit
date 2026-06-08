# signal-spirit

Signal contract for the ordinary `spirit` surface.

This crate owns the typed vocabulary for psyche statements, psyche-state
queries, intent-record queries, intent-topic catalog queries, and spirit
subscriptions. Runtime logic lives in `spirit`.

Default builds expose the binary rkyv frame surface only. Enable
`nota-text` for CLI/debug/audit NOTA projection.
