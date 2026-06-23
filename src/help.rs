//! Help as a thin view over `schema-next`'s fully specified schema IR.
//!
//! Help is not a separate language and carries no AST of its own. The model
//! stores `SpecifiedSchema` values — authored `.schema` sugar decoded into the
//! explicit semantic data tree. Rendering is a projection from that data into
//! re-headed schema declarations, encoded by schema-next's declaration codec.
//!
//! The text codec is schema-next's declaration codec end to end: encode via
//! [`SourceDeclaration::to_schema_text`], decode via
//! [`SourceDeclarations::from_schema_text`]. No hand `format!` printer, no
//! parallel decoder. The rkyv codec is the rkyv derive on the Help wrappers and
//! on the stored `SpecifiedSchema` values.

use std::fmt;

use nota_next::{Block, Delimiter, Document};
use schema_next::{
    ImportResolver, Name, SchemaEngine, SchemaError, SchemaIdentity, SchemaSource,
    SourceDeclaration, SourceDeclarationValue, SourceDeclarations, SpecifiedDeclaration,
    SpecifiedRoot, SpecifiedRootEnum, SpecifiedSchema,
};
use thiserror::Error;

use crate::{DOMAIN_SCHEMA_SOURCE, SIGNAL_SCHEMA_SOURCE};

#[derive(Debug, Error)]
pub enum HelpError {
    #[error("schema source error: {0}")]
    Schema(#[from] SchemaError),

    #[error("NOTA parse error: {0}")]
    Nota(#[from] nota_next::NotaError),

    #[error("invalid Help request: {0}")]
    InvalidRequest(String),

    #[error("unknown Help target: {0}")]
    UnknownTarget(String),
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpRequest {
    target: Option<Name>,
}

impl HelpRequest {
    pub fn new(target: Option<Name>) -> Self {
        Self { target }
    }

    pub fn for_name(name: impl Into<String>) -> Self {
        Self::new(Some(Name::new(name)))
    }

    pub fn from_text(source: &str) -> Result<Option<Self>, HelpError> {
        let document = Document::parse(source)?;
        if document.holds_root_objects() != 1 {
            return Ok(None);
        }
        let root = document
            .root_object_at(0)
            .expect("checked root object count");
        if root.demote_to_string() == Some("Help") {
            return Ok(Some(Self::new(None)));
        }
        let Some(objects) = root.as_delimited(Delimiter::Parenthesis) else {
            return Ok(None);
        };
        let Some(head) = objects.first().and_then(Block::demote_to_string) else {
            return Ok(None);
        };
        if head != "Help" {
            return Ok(None);
        }
        match objects {
            [_] => Ok(Some(Self::new(None))),
            [_, target] => {
                let Some(target) = target.demote_to_string() else {
                    return Err(HelpError::InvalidRequest(source.to_owned()));
                };
                Ok(Some(Self::for_name(target)))
            }
            _ => Err(HelpError::InvalidRequest(source.to_owned())),
        }
    }

    pub fn target(&self) -> Option<&Name> {
        self.target.as_ref()
    }
}

/// A help response is a list of (re-headed) schema declarations — the schema IR
/// for the requested roots/type. It round-trips through the one schema codec.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpResponse {
    entries: Vec<HelpEntry>,
}

impl HelpResponse {
    pub fn new(entries: Vec<HelpEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[HelpEntry] {
        &self.entries
    }

    /// Decode a help response from its schema text — the inverse of
    /// [`Self::to_schema_text`]. Each re-headed declaration parses straight
    /// back into schema declaration data through schema-next's declaration
    /// decoder; the help entry is that [`SourceDeclaration`] with no
    /// intermediate Help AST.
    pub fn from_schema_text(source: &str) -> Result<Self, HelpError> {
        let declarations = SourceDeclarations::from_schema_text(source)?;
        Ok(Self::from_source_declarations(&declarations))
    }

    /// Encode the response as canonical schema text through schema-next's
    /// declaration encoder.
    pub fn to_schema_text(&self) -> String {
        self.to_source_declarations().to_schema_text()
    }

    fn from_source_declarations(declarations: &SourceDeclarations) -> Self {
        Self::new(
            declarations
                .declarations()
                .iter()
                .map(HelpEntry::from_source_declaration)
                .collect(),
        )
    }

    fn to_source_declarations(&self) -> SourceDeclarations {
        SourceDeclarations::new(
            self.entries()
                .iter()
                .map(HelpEntry::to_source_declaration)
                .collect(),
        )
    }
}

impl fmt::Display for HelpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_schema_text())
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext,
        __C::Error: rkyv::rancor::Source
    )),
    serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator,
        __S::Error: rkyv::rancor::Source
    ),
    deserialize_bounds(__D::Error: rkyv::rancor::Source)
)]
pub struct HelpModel {
    #[rkyv(omit_bounds)]
    schemas: Vec<SpecifiedSchema>,
}

impl HelpModel {
    pub fn from_signal_schema_source() -> Result<Self, HelpError> {
        let engine = SchemaEngine::default();
        let resolver = ImportResolver::new().with_module_source(
            "signal-spirit",
            "domain",
            env!("CARGO_PKG_VERSION"),
            DOMAIN_SCHEMA_SOURCE,
        );
        let signal_source = SchemaSource::from_schema_text(SIGNAL_SCHEMA_SOURCE)?;
        let signal_schema = engine.lower_schema_source_with_resolver(
            &signal_source,
            SchemaIdentity::new("signal-spirit:signal", env!("CARGO_PKG_VERSION")),
            &resolver,
        )?;
        let domain_source = SchemaSource::from_schema_text(DOMAIN_SCHEMA_SOURCE)?;
        let domain_schema = engine.lower_schema_source(
            &domain_source,
            SchemaIdentity::new("signal-spirit:domain", env!("CARGO_PKG_VERSION")),
        )?;
        let signal_schema = SpecifiedSchema::from(&signal_schema);
        let domain_schema = SpecifiedSchema::from(&domain_schema);
        Ok(Self::from_specified_schemas(vec![
            signal_schema,
            domain_schema,
        ]))
    }

    pub fn from_specified_schemas(schemas: Vec<SpecifiedSchema>) -> Self {
        Self { schemas }
    }

    pub fn schemas(&self) -> &[SpecifiedSchema] {
        &self.schemas
    }

    pub fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
        HelpCatalog::from_schemas(self.schemas()).render(request)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HelpCatalog {
    roots: Vec<HelpEntry>,
    nodes: Vec<HelpEntry>,
}

impl HelpCatalog {
    fn from_schemas(schemas: &[SpecifiedSchema]) -> Self {
        let mut builder = HelpCatalogBuilder::empty();
        for schema in schemas {
            builder.insert_schema(schema);
        }
        builder.into_catalog()
    }

    fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
        match request.target() {
            None => Ok(HelpResponse::new(self.roots.clone())),
            Some(target) => self
                .roots
                .iter()
                .find(|entry| entry.name() == target)
                .or_else(|| self.nodes.iter().find(|entry| entry.name() == target))
                .cloned()
                .map(|entry| HelpResponse::new(vec![entry]))
                .ok_or_else(|| HelpError::UnknownTarget(target.as_str().to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HelpCatalogBuilder {
    roots: Vec<HelpEntry>,
    nodes: Vec<HelpEntry>,
}

impl HelpCatalogBuilder {
    fn empty() -> Self {
        Self {
            roots: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn into_catalog(self) -> HelpCatalog {
        HelpCatalog {
            roots: self.roots,
            nodes: self.nodes,
        }
    }

    fn insert_schema(&mut self, schema: &SpecifiedSchema) {
        self.insert_root(schema.input(), schema);
        self.insert_root(schema.output(), schema);
        for declaration in schema.declarations() {
            self.insert_declaration(declaration);
        }
        for stream in schema.streams() {
            self.insert_node(HelpEntry::new(
                stream.name.clone(),
                Some(HelpBody::from_source_declaration_value(
                    SourceDeclarationValue::from(stream),
                )),
            ));
        }
        for family in schema.families() {
            self.insert_node(HelpEntry::new(
                family.name.clone(),
                Some(HelpBody::from_source_declaration_value(
                    SourceDeclarationValue::from(family),
                )),
            ));
        }
    }

    fn insert_declaration(&mut self, declaration: &SpecifiedDeclaration) {
        self.insert_node(HelpEntry::new(
            declaration.name().clone(),
            Some(HelpBody::from_source_declaration_value(
                declaration.body().to_source_declaration_value(),
            )),
        ));
    }

    fn insert_root(&mut self, root: &SpecifiedRoot, schema: &SpecifiedSchema) {
        if let Some(root) = root.as_enum() {
            self.insert_root_enum(root, schema);
        }
    }

    fn insert_root_enum(&mut self, root: &SpecifiedRootEnum, schema: &SpecifiedSchema) {
        for variant in root.variants() {
            let root = HelpEntry::new(
                variant.name().clone(),
                variant.payload().map(|payload| {
                    HelpBody::from_source_declaration_value(
                        payload.to_help_source_declaration_value(schema),
                    )
                }),
            );
            self.roots.push(root);
        }
    }

    fn insert_node(&mut self, node: HelpEntry) {
        if let Some(existing) = self
            .nodes
            .iter_mut()
            .find(|existing| existing.name() == node.name())
        {
            *existing = node;
        } else {
            self.nodes.push(node);
        }
    }
}

/// A single help entry: a re-headed schema declaration projected from
/// `SpecifiedSchema` for schema text encoding and decoding.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext,
        __C::Error: rkyv::rancor::Source
    )),
    serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator,
        __S::Error: rkyv::rancor::Source
    ),
    deserialize_bounds(__D::Error: rkyv::rancor::Source)
)]
pub struct HelpEntry {
    name: Name,
    body: Option<HelpBody>,
}

impl HelpEntry {
    fn new(name: Name, body: Option<HelpBody>) -> Self {
        Self { name, body }
    }

    fn from_source_declaration(declaration: &SourceDeclaration) -> Self {
        Self::new(
            declaration.name().clone(),
            declaration
                .value()
                .cloned()
                .map(HelpBody::from_source_declaration_value),
        )
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    /// The schema declaration body projected from the stored `SpecifiedSchema`.
    pub fn body(&self) -> Option<&HelpBody> {
        self.body.as_ref()
    }

    /// Re-head the projected body over the entry name as a source declaration
    /// so the schema encoder produces `(Head <body-schema-text>)`.
    fn to_source_declaration(&self) -> SourceDeclaration {
        SourceDeclaration::new(
            self.name.clone(),
            self.body
                .as_ref()
                .map(HelpBody::to_source_declaration_value),
        )
    }

    /// Encode this entry as canonical schema text through schema-next's
    /// declaration encoder.
    pub fn to_schema_text(&self) -> String {
        self.to_source_declaration().to_schema_text()
    }
}

impl fmt::Display for HelpEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_schema_text())
    }
}

/// A Help response body owned by `signal-spirit`.
///
/// The schema-codec value is kept private so clients do not consume
/// schema-next source nouns as the public Help API.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext,
        __C::Error: rkyv::rancor::Source
    )),
    serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator,
        __S::Error: rkyv::rancor::Source
    ),
    deserialize_bounds(__D::Error: rkyv::rancor::Source)
)]
pub struct HelpBody {
    #[rkyv(omit_bounds)]
    value: SourceDeclarationValue,
}

impl HelpBody {
    fn from_source_declaration_value(value: SourceDeclarationValue) -> Self {
        Self { value }
    }

    fn to_source_declaration_value(&self) -> SourceDeclarationValue {
        self.value.clone()
    }

    pub fn to_schema_text(&self) -> String {
        self.value.to_schema_text()
    }
}
