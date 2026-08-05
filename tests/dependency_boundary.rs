use std::{path::Path, process::Command};

#[test]
fn ordinary_contract_is_generated_from_the_sealed_ethos_root() {
    let build = include_str!("../build.rs");
    let library = include_str!("../src/lib.rs");

    for required in [
        "spirit_ethos::INTERFACE",
        "BatchConfiguration::from_json",
        "generate_bundle",
        "signal-spirit.rs",
    ] {
        assert!(
            build.contains(required),
            "the ordinary build must retain its sealed Ethos generator step {required:?}"
        );
    }
    for forbidden in ["schema_rust::", "schema::", "Nota"] {
        assert!(
            !build.contains(forbidden) && !library.contains(forbidden),
            "the ordinary public contract must not restore the retired schema projection {forbidden:?}"
        );
    }
    assert!(library.contains("include!(concat!(env!(\"OUT_DIR\"), \"/signal-spirit.rs\"))"));

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for retired_artifact in [
        "schema/signal.schema",
        "src/schema/mod.rs",
        "src/schema/domain.rs",
        "src/schema/signal.rs",
    ] {
        assert!(
            !root.join(retired_artifact).exists(),
            "the generated ordinary surface must not retain {retired_artifact}"
        );
    }
}

#[test]
fn dotos_and_archive_sources_are_exact_and_single_world() {
    let manifest = include_str!("../Cargo.toml");
    let lockfile = include_str!("../Cargo.lock");
    let dotos_revision = "80c7b17f7ad3cf547d2624c6a243e5de5f85c9f3";

    for exact_pin in [
        dotos_revision,
        "4cb9cb704965489ea6c25f148bcd8c723c9a84c6",
        "version = \"=0.8.17\"",
    ] {
        assert!(
            manifest.contains(exact_pin),
            "manifest must retain exact source pin {exact_pin}"
        );
    }
    assert_eq!(
        lockfile
            .matches("name = \"dotos\"\nversion = \"0.10.0\"")
            .count(),
        1,
        "the lockfile must contain one Dotos package identity"
    );
    assert_eq!(
        lockfile
            .matches("name = \"rkyv\"\nversion = \"0.8.17\"")
            .count(),
        1,
        "the lockfile must contain one rkyv 0.8.17 package identity"
    );
    assert!(
        !lockfile.contains("?rev=80c7b17f7ad3#"),
        "the short Dotos revision is an invalid distinct source identity"
    );

    let output = Command::new("cargo")
        .args(["tree", "--locked", "--all-features", "-i", "dotos"])
        .output()
        .expect("run the Dotos source-identity query");
    assert!(output.status.success(), "status: {:?}", output.status);
    let tree = String::from_utf8(output.stdout).expect("Dotos source identity");
    assert!(
        tree.contains(&format!(
            "dotos v0.10.0 (https://github.com/LiGoldragon/dotos.git?rev={dotos_revision}#80c7b17f)"
        )),
        "Dotos must resolve from the exact current source:\n{tree}"
    );
    assert!(
        !tree.contains("?rev=80c7b17f7ad3#"),
        "Dotos must reject the short-revision source identity:\n{tree}"
    );
}
