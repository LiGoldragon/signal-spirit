# signal-spirit

The ordinary Signal contract for `spirit`, generated during the build from the
sealed `spirit-ethos` `Interface` root. Runtime behavior, storage, admission,
and migration remain in `spirit`; this crate exposes only the generated wire
artifact.

The default `dotos-text` feature supplies the Dotos text codec alongside the
rkyv archive contract. `--no-default-features` is the binary-only consumer
surface. Both use the same generated types and the fixed rkyv 0.8.17 archive
ABI.

`spirit-ethos` is the sole authored contract source. Its allocation-backed
universal identities become the public canonical encoded Rust identifiers; this
crate deliberately provides no legacy schema projection or compatibility
aliases.
