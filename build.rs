use std::{env, path::PathBuf};

use schema_rust::build::{
    CargoSchemaMetadata, DependencySchema, GenerationDriver, GenerationPlan, ModuleEmission,
};

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
        println!("cargo:rerun-if-changed=schema/signal.schema");
        println!("cargo:rerun-if-changed=src/schema/signal.rs");

        let signal_domain_schema =
            DependencySchema::from_cargo_metadata("signal-domain", "signal-domain", "0.1.0")
                .expect("read signal-domain schema metadata");

        let plan = GenerationPlan::new(&self.crate_root, "signal-spirit", "0.12.0")
            .with_optional_dependency_schema(signal_domain_schema)
            .with_module(ModuleEmission::wire_contract_module("signal"));

        GenerationDriver::new(plan)
            .generate()
            .expect("generate signal-spirit schema artifacts")
            .write_or_check("SIGNAL_SPIRIT_UPDATE_SCHEMA_ARTIFACTS")
            .expect("checked-in signal-spirit schema artifacts are fresh");
        CargoSchemaMetadata::new("signal-spirit").emit_schema_directory(&self.crate_root);
    }
}
