use std::{env, fs, path::PathBuf};

use nomos_engine::batch::{
    BatchComponent, BatchConfiguration, OfflineBatchConfiguration, OfflineBatchGeneration,
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let source = spirit_ethos::source_root();
    for file in ["batch-config.json", "interface.ethos"] {
        println!("cargo:rerun-if-changed={}", source.join(file).display());
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("Cargo did not set OUT_DIR")?);
    let generator = BatchConfiguration::from_json(spirit_ethos::BATCH_CONFIGURATION)?.prepare()?;
    let outcomes = generator
        .generate_bundle(&[BatchComponent::named("interface", spirit_ethos::INTERFACE)])?;
    let [interface] = outcomes.as_slice() else {
        return Err("the sealed ordinary bundle must emit exactly one Interface artifact".into());
    };
    if interface.kind().spelling() != "Interface" {
        return Err("the sealed ordinary root did not emit an Interface artifact".into());
    }
    fs::write(output.join("signal-spirit.rs"), interface.rust())?;
    Ok(())
}
