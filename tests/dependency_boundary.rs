use std::{path::Path, process::Command};

#[test]
fn authority_sealed_generation_through_bootstrap_pipeline() {
    let build = include_str!("../build.rs");
    let library = include_str!("../src/lib.rs");

    for required in [
        "SemaBootstrapAuthority",
        "BootstrapGeneration",
        "CommitBootstrap",
        "with_role_traits",
        "spirit.schema",
    ] {
        assert!(
            build.contains(required),
            "the authority build must contain {required:?}"
        );
    }
    for retired in [
        "spirit_ethos",
        "nomos_engine",
        "BatchConfiguration",
        "generate_bundle",
        "OUT_DIR",
    ] {
        assert!(
            !build.contains(retired),
            "the authority build must not reference retired pipeline element {retired:?}"
        );
    }
    assert!(
        !library.contains("include!(concat!(env!(\"OUT_DIR\")"),
        "the generated projection must use a checked-in include, not OUT_DIR"
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        root.join("schema/spirit.schema").exists(),
        "the canonical Interface source must exist at schema/spirit.schema"
    );
    assert!(
        root.join("src/schema/spirit/generated.rs").exists(),
        "the checked-in Rust projection must exist at src/schema/spirit/generated.rs"
    );
    for retired_artifact in [
        "schema/signal.schema",
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
fn nomos_engine_and_spirit_ethos_dependencies_removed() {
    let manifest = include_str!("../Cargo.toml");

    for retired in ["nomos-engine", "spirit-ethos"] {
        assert!(
            !manifest.contains(retired),
            "manifest must not depend on retired package {retired:?}"
        );
    }
    for required in ["sema-translator", "schema-rust", "core-nomos", "name-table", "rust-logos"] {
        assert!(
            manifest.contains(required),
            "manifest must contain authority pipeline dependency {required:?}"
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
        "version = \"=0.8.18\"",
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
            .matches("name = \"rkyv\"\nversion = \"0.8.18\"")
            .count(),
        1,
        "the lockfile must contain one rkyv 0.8.18 package identity"
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
