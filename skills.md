# signal-spirit editing contract

Read `ARCHITECTURE.md` before editing.

- This crate owns wire vocabulary only; runtime behavior belongs in `spirit`.
- Version 0.14.0 is wire revision 2 and accepts only its native shapes.
- `Entry` is exactly `Domains`, `Kind`, `Description`, `Importance`.
- Uniform reads are `Observe`, `Intent`, `TextSearch`, `Lookup`, and `Count`.
- Importance selection and `BumpImportance` remain part of the contract.
- Identifiers and provenance are daemon-produced; clients provide neither.
- Keep the default graph binary/rkyv-only and text projection opt-in.
- Edit `schema/signal.schema`, regenerate `src/schema/signal.rs`, and prove
  generated/schema convergence plus old-syntax rejection.
