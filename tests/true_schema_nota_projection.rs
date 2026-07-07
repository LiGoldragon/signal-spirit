//! Direct structured NOTA projection for the decoded Spirit signal TrueSchema.
//!
//! This proves the authored Spirit signal schema can lower to schema-language's
//! semantic `TrueSchema`, render as structured NOTA, decode back from that NOTA,
//! and feed the Help display projection without generated Rust being the source of
//! truth.

#![cfg(feature = "nota-text")]

use nota::{Document, NotaDecode, NotaEncode};
use schema_language::{ImportResolver, SchemaEngine, SchemaIdentity, SchemaSource, TrueSchema};
use signal_spirit::{DOMAIN_SCHEMA_SOURCE, HelpModel, HelpRequest, SIGNAL_SCHEMA_SOURCE};

const DOMAIN_HELP_ROW: &str = "[All (Health Health) (Food Food) (Home Home) (Finance Finance) (Work Work) (Craft Craft) (Knowledge Knowledge) (Education Education) (Language Language) (Art Art) (Kinship Kinship) (Selfhood Selfhood) (Spirituality Spirituality) (Governance Governance) (Law Law) (Community Community) (Nature Nature) (Travel Travel) (Commerce Commerce) (Leisure Leisure) (Appearance Appearance) (Safety Safety) (Information Information) (Technology Technology)]";

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedSpiritSchemas {
    signal: TrueSchema,
    domain: TrueSchema,
}

impl DecodedSpiritSchemas {
    fn from_authored_sources() -> Self {
        let engine = SchemaEngine::default();
        let resolver = ImportResolver::new().with_module_source(
            "signal-domain",
            "domain",
            "0.1.0",
            DOMAIN_SCHEMA_SOURCE,
        );
        let signal_source = SchemaSource::from_schema_text(SIGNAL_SCHEMA_SOURCE)
            .expect("signal schema source decodes");
        let signal = engine
            .lower_schema_source_with_resolver(
                &signal_source,
                SchemaIdentity::new("signal-spirit:signal", env!("CARGO_PKG_VERSION")),
                &resolver,
            )
            .expect("signal schema lowers to TrueSchema");
        let domain_source = SchemaSource::from_schema_text(DOMAIN_SCHEMA_SOURCE)
            .expect("domain schema source decodes");
        let domain = engine
            .lower_schema_source(
                &domain_source,
                SchemaIdentity::new("signal-domain:domain", "0.1.0"),
            )
            .expect("domain schema lowers to TrueSchema");
        Self { signal, domain }
    }

    fn help_model(&self) -> HelpModel {
        HelpModel::from_true_schemas(vec![self.signal.clone(), self.domain.clone()])
    }
}

#[test]
fn decoded_spirit_signal_true_schema_projects_to_structured_nota() {
    let schemas = DecodedSpiritSchemas::from_authored_sources();
    let rendered = schemas.signal.to_nota();

    let expected_prefix = format!(
        "((signal-spirit:signal {}) [(Domain (Plain signal-domain:domain:Domain)) (DomainScope (Plain signal-domain:domain:DomainScope)) (DomainScopes (Plain signal-domain:domain:DomainScopes)) (ScopeSet (Plain signal-domain:domain:ScopeSet))",
        env!("CARGO_PKG_VERSION")
    );
    let prefix_excerpt = rendered.chars().take(256).collect::<String>();
    assert!(
        rendered.starts_with(&expected_prefix),
        "structured TrueSchema NOTA should begin with the signal identity and resolved domain imports; prefix was {prefix_excerpt}"
    );
    assert!(
        rendered.contains(
            "(Enum (Input [(State (Some (Plain State)) None) (Record (Some (Plain Record)) None)"
        ),
        "structured TrueSchema NOTA should expose the decoded Input root enum"
    );
    assert!(
        rendered.contains(
            "(SubscribeIntent (Some (Plain SubscribeIntent)) (Some (Opens IntentEventStream)))"
        ),
        "structured TrueSchema NOTA should preserve stream relations on root variants"
    );
    assert!(
        rendered.contains(
            "(Public Entry [] (Struct (Entry {domains (Plain Domains) kind (Plain Kind) description (Plain Description) certainty (Plain Certainty) importance (Plain Importance) privacy (Plain Privacy) referents (Plain Referents)})) ([]))"
        ),
        "structured TrueSchema NOTA should include the semantic Entry declaration"
    );

    let document = Document::parse(&rendered).expect("structured TrueSchema NOTA parses");
    assert_eq!(
        document.holds_root_objects(),
        1,
        "TrueSchema projection should be one NOTA root object"
    );
    let decoded = TrueSchema::from_nota_block(&document.root_objects()[0])
        .expect("structured TrueSchema NOTA decodes");
    assert_eq!(
        decoded, schemas.signal,
        "structured NOTA projection should decode back to the same TrueSchema"
    );

    assert!(
        !rendered.contains("(Public Domain []"),
        "structured signal TrueSchema NOTA should not include a contract-local domain taxonomy"
    );
    assert!(
        rendered.contains("(PublicIntent (Some (Plain PublicIntent)) None)"),
        "structured signal TrueSchema NOTA should keep PublicIntent while resolving its payload through the shared domain import"
    );

    let rendered_domain = schemas.domain.to_nota();
    let domain_document =
        Document::parse(&rendered_domain).expect("structured Domain TrueSchema NOTA parses");
    let decoded_domain = TrueSchema::from_nota_block(&domain_document.root_objects()[0])
        .expect("structured Domain TrueSchema NOTA decodes");
    assert_eq!(
        decoded_domain, schemas.domain,
        "structured Domain NOTA projection should decode back to the same TrueSchema"
    );
}

#[test]
fn decoded_true_schema_feeds_label_free_help_rows() {
    let schemas = DecodedSpiritSchemas::from_authored_sources();
    let model = schemas.help_model();

    assert_eq!(
        model,
        HelpModel::from_signal_schema_source().expect("build deployed help model"),
        "the deployed Help model should store the same decoded TrueSchema values"
    );
    assert_eq!(
        model
            .render(&HelpRequest::for_name("Record"))
            .expect("render Record help")
            .to_string(),
        "{ Entry Justification }\n{ Domains Kind Description Certainty Importance Privacy Referents }\n{ Testimony Reasoning }",
        "Record Help should be projected from decoded TrueSchema rows"
    );
    assert_eq!(
        model
            .render(&HelpRequest::for_name("Entry"))
            .expect("render Entry help")
            .to_string(),
        "{ Domains Kind Description Certainty Importance Privacy Referents }\n(Vector Domain)\n[Decision Principle Correction Clarification Constraint]\nString\nMagnitude\nMagnitude\nMagnitude\n(Vector Referent)",
        "Entry Help should be projected from decoded TrueSchema rows"
    );
    assert_eq!(
        model
            .render(&HelpRequest::for_name("Domain"))
            .expect("render Domain help")
            .to_string(),
        DOMAIN_HELP_ROW,
        "Domain Help should include top-level All from decoded TrueSchema rows"
    );
}
