use std::{fs, path::Path, process::Command};

#[test]
fn default_dependency_tree_does_not_pull_text_or_legacy_signal_crates() {
    let output = Command::new("cargo")
        .args([
            "tree",
            "--locked",
            "--edges",
            "normal",
            "--no-default-features",
        ])
        .output()
        .expect("run cargo tree");

    assert!(output.status.success(), "status: {:?}", output.status);
    let tree = String::from_utf8(output.stdout).expect("dependency tree");

    for forbidden_crate in ["nota", "nota-codec", "protos", "signal-core"] {
        assert!(
            !tree.contains(forbidden_crate),
            "default dependency tree must not contain {forbidden_crate}:\n{tree}"
        );
    }
}

#[test]
fn nota_text_feature_is_the_only_text_projection_opt_in() {
    let output = Command::new("cargo")
        .args([
            "tree",
            "--locked",
            "--edges",
            "normal",
            "--no-default-features",
            "--features",
            "nota-text",
        ])
        .output()
        .expect("run cargo tree");

    assert!(output.status.success(), "status: {:?}", output.status);
    let tree = String::from_utf8(output.stdout).expect("dependency tree");

    assert!(
        tree.contains("nota"),
        "nota-text feature should opt into nota:\n{tree}"
    );
    for forbidden_crate in ["nota-codec", "signal-core"] {
        assert!(
            !tree.contains(forbidden_crate),
            "nota-text dependency tree must not contain {forbidden_crate}:\n{tree}"
        );
    }
}

#[test]
fn all_features_resolve_one_exact_schema_and_nota_world() {
    let output = Command::new("cargo")
        .args(["tree", "--locked", "--all-features", "--duplicates"])
        .output()
        .expect("run duplicate dependency tree");

    assert!(output.status.success(), "status: {:?}", output.status);
    let duplicates = String::from_utf8(output.stdout).expect("duplicate dependency tree");
    for duplicate in duplicates
        .split("\n\n")
        .filter(|duplicate| !duplicate.trim().is_empty())
    {
        assert!(
            duplicate.starts_with(
                "signal-frame v0.4.0 (https://github.com/LiGoldragon/signal-frame.git?rev=0786fbe8caf27552afcdd5deb85bc82ec6088337#0786fbe8)"
            ),
            "all features must resolve without duplicate producer worlds; the one permitted host/target rebuild is the exact same signal-frame source:\n{duplicates}"
        );
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    let lock = fs::read_to_string(root.join("Cargo.lock")).expect("read Cargo.lock");

    for exact_pin in [
        "4ea4892d717247cce2d0c221b100314077f3fa3d",
        "0786fbe8caf27552afcdd5deb85bc82ec6088337",
        "485be1c609e5f2038fdf54ed0de04cd29d884b06",
        "89dc3c85a9ff96d4e4d53accfd867df672cae5a8",
        "9c217610c4b8d3bdaa9f95542e28c04424a593e3",
        "4cb55c87174db23ba21237f5975bf97b4c0690b5",
        "c322127d85f442eb7a0d3152d8bacea638d3f6ea",
    ] {
        assert!(
            manifest.contains(exact_pin),
            "manifest is missing exact producer pin {exact_pin}"
        );
        assert!(
            lock.contains(exact_pin),
            "lock is missing exact producer pin {exact_pin}"
        );
    }

    for exact_transitive_pin in [
        "acfb386fa638e2979b33bf2d003bb35fed0ceec0",
        "f491b45d7dcb55e5837eddde3d5d7ca8ceaa9f01",
        "ac0075842799b3ece8909ad0eb4b8a92b596b188",
        "a1705ef512efec28925ae3ffc9faa5a2aa4dc4a8",
        "c27a9efabb1981c8b3d887c870fff82fc7daf49c",
        "e5fa1b3bbdde13f3dac205920b16a2e73f3d4487",
    ] {
        assert!(
            lock.contains(exact_transitive_pin),
            "lock is missing exact transitive producer pin {exact_transitive_pin}"
        );
    }

    for superseded_pin in [
        "f5e79ffd1f6985cb12925ddd43addb66d6755b54",
        "e27bbb5752f133589ba3200c3aace1b350c5123e",
        "3721656b0a654d47d9abde31f14d89d01f9305cf",
    ] {
        assert!(
            !manifest.contains(superseded_pin),
            "manifest retains superseded producer pin {superseded_pin}"
        );
        assert!(
            !lock.contains(superseded_pin),
            "lock retains superseded producer pin {superseded_pin}"
        );
    }

    for forbidden_source in ["nota-next", "[patch.", "branch = "] {
        assert!(
            !manifest.contains(forbidden_source),
            "manifest must not contain alternate dependency source {forbidden_source}"
        );
        assert!(
            !lock.contains(forbidden_source),
            "lock must not contain alternate dependency source {forbidden_source}"
        );
    }
    assert!(!manifest.contains("branch = "));
    assert!(!lock.contains("?branch="));
    assert!(!lock.contains("path+"));
}

#[test]
fn generated_and_runtime_contract_surface_do_not_depend_on_protos() {
    assert!(!signal_spirit::SIGNAL_RUST_SOURCE.contains("protos::"));
    assert!(!signal_spirit::SIGNAL_RUST_SOURCE.contains("WireContractFamily"));
}
