use std::{env, path::PathBuf};

use schema_rust_next::build::{GenerationDriver, GenerationPlan, ModuleEmission};

fn main() {
    SchemaBuild::from_environment().run();
}

struct SchemaBuild {
    crate_root: PathBuf,
}

impl SchemaBuild {
    fn from_environment() -> Self {
        Self {
            crate_root: PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir set")),
        }
    }

    fn run(&self) {
        println!("cargo:rerun-if-changed=schema/domain.schema");
        println!("cargo:rerun-if-changed=schema/signal.schema");
        println!("cargo:rerun-if-changed=src/schema/domain.rs");
        println!("cargo:rerun-if-changed=src/schema/signal.rs");

        let plan = GenerationPlan::new(&self.crate_root, "signal-spirit", "0.6.0")
            .with_module(ModuleEmission::declaration_module("domain"))
            .with_module(ModuleEmission::wire_contract_module("signal"));

        GenerationDriver::new(plan)
            .generate()
            .expect("generate signal-spirit schema artifacts")
            .write_or_check("SIGNAL_SPIRIT_UPDATE_SCHEMA_ARTIFACTS")
            .expect("checked-in signal-spirit schema artifacts are fresh");
    }
}
