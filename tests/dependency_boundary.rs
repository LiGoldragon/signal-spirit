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

    for forbidden_crate in ["nota", "nota-codec", "signal-core"] {
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
    assert!(
        duplicates.trim().is_empty(),
        "all features must resolve without duplicate package worlds:\n{duplicates}"
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    let lock = fs::read_to_string(root.join("Cargo.lock")).expect("read Cargo.lock");

    for exact_pin in [
        "f5e79ffd1f6985cb12925ddd43addb66d6755b54",
        "e27bbb5752f133589ba3200c3aace1b350c5123e",
        "485be1c609e5f2038fdf54ed0de04cd29d884b06",
        "89dc3c85a9ff96d4e4d53accfd867df672cae5a8",
        "9c217610c4b8d3bdaa9f95542e28c04424a593e3",
        "3721656b0a654d47d9abde31f14d89d01f9305cf",
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
