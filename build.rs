//! Bootstrap generation for the spirit Interface through the unified
//! authority pipeline.
//!
//! This build script authorizes `schema/spirit.schema` through
//! `SemaBootstrapAuthority`, generates canonical Ethos source and Rust
//! projections through `BootstrapGeneration`, and installs them atomically
//! through `CommitBootstrap`.
//!
//! The dependency's domain Interface source is authorized first so the
//! spirit source can resolve its `signal/domain` imports.
//!
//! The Observer stream's initiation/termination mapping is held for the
//! psyche — specifically whether `Observe.Query` double-duties as stream
//! initiation and that a termination entry does not yet exist.
//!
// psyche-grasp: unseen

use std::{collections::BTreeMap, env, fs, path::PathBuf};

use core_nomos::InterfaceRoleTraitIdentities;
use name_table::{EncodedName, NameView, TextualMetadata};
use rust_logos::{RustLogos, RustTypePath, RustTypePathResolver};
use schema_rust::{
    bootstrap::{BootstrapGeneration, CommitBootstrap},
    build::CargoEthosSourceMetadata,
};
use sema_translator::bootstrap::{AuthorityNameView, SemaBootstrapAuthority, SourcePlacement};

fn main() {
    SpiritBuild::from_environment().run();
}

struct SpiritBuild {
    crate_root: PathBuf,
}

impl SpiritBuild {
    fn from_environment() -> Self {
        Self {
            crate_root: PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir set")),
        }
    }

    fn run(&self) {
        println!("cargo:rerun-if-changed=schema/spirit.schema");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=src/schema/spirit/generated.rs");

        let domain_metadata = CargoEthosSourceMetadata::new("signal-domain");
        domain_metadata.emit_dependency_rerun_instruction();

        let source_path = self.crate_root.join("schema/spirit.schema");
        let rust_path = self.crate_root.join("src/schema/spirit/generated.rs");
        let source = fs::read_to_string(&source_path).expect("read spirit Interface source");

        let mut authority =
            SemaBootstrapAuthority::new().expect("empty authority owns its seed");

        // Authorize the domain dependency first so its types are visible to the
        // spirit Interface source during planning.
        self.authorize_domain_dependency(&mut authority, &domain_metadata);

        let priors = authority.prior_identities();
        let role_traits = InterfaceRoleTraitIdentities::new(
            priors.input_role.clone(),
            priors.output_role.clone(),
            priors.refusal_role.clone(),
            priors.stream_role.clone(),
        );

        let placement = SourcePlacement::new(
            vec!["signal_spirit".to_owned(), "spirit".to_owned()],
            vec![
                "signal_spirit".to_owned(),
                "spirit".to_owned(),
                "spirit.schema".to_owned(),
            ],
        );

        let assembly = authority
            .authorize(&source, placement)
            .expect("assemble authority-approved spirit Interface transaction");
        let rust = RustLogos::new();
        let type_paths = SpiritRustTypePaths::from_name_view(assembly.name_view());

        let generated = BootstrapGeneration::new(
            &assembly,
            &rust,
            &type_paths,
            &[],
            &source_path,
            &rust_path,
        )
        .with_role_traits(&role_traits)
        .generate()
        .expect("project spirit Interface from the verified transaction");

        CommitBootstrap::single(generated)
            .write_or_check("SIGNAL_SPIRIT_UPDATE_INTERFACE_ARTIFACTS")
            .expect("checked-in spirit Interface source and Rust projection are fresh");

        CargoEthosSourceMetadata::new("signal-spirit")
            .publish_owned_source_directory(self.crate_root.join("schema"));
    }

    fn authorize_domain_dependency(
        &self,
        authority: &mut SemaBootstrapAuthority,
        metadata: &CargoEthosSourceMetadata,
    ) {
        let source_dir = metadata
            .dependency_source_directory()
            .expect("signal-domain must publish its Ethos source directory via `links`");
        let domain_source_path = source_dir.join("domain.schema");
        let domain_source =
            fs::read_to_string(&domain_source_path).expect("read domain Interface source");

        let domain_placement = SourcePlacement::new(
            vec!["signal".to_owned(), "domain".to_owned()],
            vec![
                "signal".to_owned(),
                "domain".to_owned(),
                "domain.schema".to_owned(),
            ],
        );

        authority
            .admit_domain_shape("ScopeOf", 1)
            .expect("authority admits the domain ScopeOf shape constructor");
        authority
            .authorize(&domain_source, domain_placement)
            .expect("authorize the domain dependency Interface for import resolution");
    }
}

/// Resolves external Rust type paths by looking up textual names through the
/// sealed authority name view.
struct SpiritRustTypePaths<'a> {
    name_view: &'a AuthorityNameView,
    overrides: BTreeMap<&'static str, RustTypePath>,
}

impl<'a> SpiritRustTypePaths<'a> {
    fn from_name_view(name_view: &'a AuthorityNameView) -> Self {
        let path = |segments: &[&str]| -> RustTypePath {
            RustTypePath::try_new(segments.iter().map(|s| (*s).to_owned()).collect())
                .expect("static Rust type path segments are valid")
        };
        let overrides = BTreeMap::from([
            // Role traits from protos.
            ("Input", path(&["protos", "Input"])),
            ("Output", path(&["protos", "Output"])),
            ("Refusal", path(&["protos", "Refusal"])),
            ("Stream", path(&["protos", "Stream"])),
            // Imported domain types.
            ("Domain", path(&["signal_domain", "Domain"])),
            ("DomainScopes", path(&["signal_domain", "DomainScopes"])),
        ]);
        Self {
            name_view,
            overrides,
        }
    }
}

impl RustTypePathResolver for SpiritRustTypePaths<'_> {
    fn resolve_type_path(&self, encoded_name: &EncodedName) -> Option<&RustTypePath> {
        let metadata: &TextualMetadata = self.name_view.textual_metadata(encoded_name)?;
        self.overrides.get(metadata.textual_name().as_str())
    }
}
