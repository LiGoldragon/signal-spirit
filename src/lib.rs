//! signal-spirit — ordinary signal contract for the spirit component.
//!
//! The wire surface is declared in
//! [`schema/signal-spirit.schema`](../schema/signal-spirit.schema) and
//! emitted via `build.rs` (which invokes schema-next + schema-rust-next
//! to lower the schema and produce Rust source). The generated module
//! is included below.
//!
//! Per the designer running-spirit-concept track (/368): this is the
//! minimum one-operation contract that proves end-to-end communication
//! through the schema-derived stack. See `ARCHITECTURE.md` for the
//! component-triad shape and `INTENT.md` for the psyche-scope intent.

#![forbid(unsafe_code)]

/// Schema-emitted types. Built from `schema/signal-spirit.schema` via
/// `build.rs`. Every type in this module — including the rkyv derives
/// and the NOTA codec impls — is produced by schema-rust-next, not
/// hand-rolled. See /368 §"What's still hand-rolled" for the precise
/// emitted-vs-hand-rolled boundary.
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/signal_spirit_generated.rs"));
}

pub use generated::{
    Description, Entry, Input, NotaDecodeError, Output, RecordIdentifier, Topic, short_header,
};

/// Encode an operation (`Input`) for the wire via rkyv.
///
/// The CLI calls this before writing bytes to the daemon socket. The
/// wire format is rkyv binary (record 695: one rkyv layout, two homes —
/// signal here; SEMA at rest).
pub struct WireCodec;

impl WireCodec {
    /// Encode an `Input` (operation) into rkyv bytes for the wire.
    pub fn encode_input(input: &Input) -> Result<Vec<u8>, rkyv::rancor::Error> {
        rkyv::to_bytes::<rkyv::rancor::Error>(input).map(|bytes| bytes.to_vec())
    }

    /// Decode rkyv bytes from the wire into an `Input` (operation).
    pub fn decode_input(bytes: &[u8]) -> Result<Input, rkyv::rancor::Error> {
        rkyv::from_bytes::<Input, rkyv::rancor::Error>(bytes)
    }

    /// Encode an `Output` (reply) into rkyv bytes for the wire.
    pub fn encode_output(output: &Output) -> Result<Vec<u8>, rkyv::rancor::Error> {
        rkyv::to_bytes::<rkyv::rancor::Error>(output).map(|bytes| bytes.to_vec())
    }

    /// Decode rkyv bytes from the wire into an `Output` (reply).
    pub fn decode_output(bytes: &[u8]) -> Result<Output, rkyv::rancor::Error> {
        rkyv::from_bytes::<Output, rkyv::rancor::Error>(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> Input {
        Input::Record(Entry {
            topic: Topic(String::from("running-concept")),
            description: Description(String::from("designer running spirit concept end-to-end")),
        })
    }

    fn sample_output() -> Output {
        Output::RecordAccepted(RecordIdentifier(42))
    }

    #[test]
    fn schema_derived_input_round_trips_through_rkyv() {
        let input = sample_input();
        let bytes = WireCodec::encode_input(&input).expect("rkyv encode input");
        let decoded = WireCodec::decode_input(&bytes).expect("rkyv decode input");
        assert_eq!(decoded, input);
    }

    #[test]
    fn schema_derived_output_round_trips_through_rkyv() {
        let output = sample_output();
        let bytes = WireCodec::encode_output(&output).expect("rkyv encode output");
        let decoded = WireCodec::decode_output(&bytes).expect("rkyv decode output");
        assert_eq!(decoded, output);
    }

    #[test]
    fn schema_derived_input_round_trips_through_nota_text() {
        let input = sample_input();
        let text = input.to_string();
        let parsed = text.parse::<Input>().expect("parse NOTA");
        assert_eq!(parsed, input);
    }

    #[test]
    fn schema_derived_output_round_trips_through_nota_text() {
        let output = sample_output();
        let text = output.to_string();
        let parsed = text.parse::<Output>().expect("parse NOTA");
        assert_eq!(parsed, output);
    }

    #[test]
    fn short_headers_are_derived_in_surface_variant_order() {
        assert_eq!(short_header::INPUT_RECORD, 0x0000_0000_0000_0000);
        assert_eq!(short_header::OUTPUT_RECORD_ACCEPTED, 0x0100_0000_0000_0000);
    }

    #[test]
    fn nota_text_payload_matches_human_authored_shape() {
        let input = sample_input();
        // The CLI on stdin/stdout sees bracket-form NOTA — never quoted
        // strings, per AGENTS.md hard override (NOTA bracket-only).
        let expected = "(Record ([running-concept] [designer running spirit concept end-to-end]))";
        assert_eq!(input.to_string(), expected);
    }
}
