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
    target: Option<HelpName>,
}

impl HelpRequest {
    pub fn new(target: Option<HelpName>) -> Self {
        Self { target }
    }

    pub fn for_name(name: impl Into<String>) -> Self {
        Self::new(Some(HelpName::new(name)))
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

    pub fn target(&self) -> Option<&HelpName> {
        self.target.as_ref()
    }
}

/// A help response is a list of (re-headed) schema declarations — the schema IR
/// for the requested roots/type. It round-trips through the one schema codec.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpResponse {
    entries: HelpEntries,
}

impl HelpResponse {
    pub fn new(entries: HelpEntries) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &HelpEntries {
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
        Self::new(HelpEntries::new(
            declarations
                .declarations()
                .iter()
                .map(HelpEntry::from_source_declaration)
                .collect(),
        ))
    }

    fn to_source_declarations(&self) -> SourceDeclarations {
        SourceDeclarations::new(
            self.entries()
                .entries()
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
pub struct HelpModel {
    schemas: HelpSchemas,
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
        Self {
            schemas: HelpSchemas::new(schemas),
        }
    }

    pub fn schemas(&self) -> &HelpSchemas {
        &self.schemas
    }

    pub fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
        HelpCatalog::from_schemas(self.schemas.schemas()).render(request)
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
pub struct HelpSchemas {
    #[rkyv(omit_bounds)]
    schemas: Vec<SpecifiedSchema>,
}

impl HelpSchemas {
    pub fn new(schemas: Vec<SpecifiedSchema>) -> Self {
        Self { schemas }
    }

    pub fn schemas(&self) -> &[SpecifiedSchema] {
        &self.schemas
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HelpCatalog {
    roots: HelpRoots,
    nodes: HelpNodes,
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
            None => Ok(HelpResponse::new(HelpEntries::from_roots(
                self.roots.roots(),
            ))),
            Some(target) => self
                .roots
                .find(target)
                .map(|root| HelpResponse::new(HelpEntries::single(HelpEntry::from_root(root))))
                .or_else(|| {
                    self.nodes.find(target).map(|node| {
                        HelpResponse::new(HelpEntries::single(HelpEntry::from_node(node)))
                    })
                })
                .ok_or_else(|| HelpError::UnknownTarget(target.as_str().to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HelpCatalogBuilder {
    roots: HelpRoots,
    nodes: HelpNodes,
}

impl HelpCatalogBuilder {
    fn empty() -> Self {
        Self {
            roots: HelpRoots::empty(),
            nodes: HelpNodes::empty(),
        }
    }

    fn into_catalog(self) -> HelpCatalog {
        HelpCatalog {
            roots: self.roots,
            nodes: self.nodes,
        }
    }

    fn insert_schema(&mut self, schema: &SpecifiedSchema) {
        self.insert_root(HelpPlane::Input, schema.input());
        self.insert_root(HelpPlane::Output, schema.output());
        for declaration in schema.declarations() {
            self.insert_declaration(declaration);
        }
        for stream in schema.streams() {
            self.nodes.insert(HelpNode::new(
                HelpName::from(&stream.name),
                Some(SourceDeclarationValue::from(stream)),
            ));
        }
        for family in schema.families() {
            self.nodes.insert(HelpNode::new(
                HelpName::from(&family.name),
                Some(SourceDeclarationValue::from(family)),
            ));
        }
    }

    fn insert_declaration(&mut self, declaration: &SpecifiedDeclaration) {
        self.nodes.insert(HelpNode::new(
            HelpName::from(declaration.name()),
            Some(declaration.body().to_source_declaration_value()),
        ));
    }

    fn insert_root(&mut self, plane: HelpPlane, root: &SpecifiedRoot) {
        if let Some(root) = root.as_enum() {
            self.insert_root_enum(plane, root);
        }
    }

    fn insert_root_enum(&mut self, plane: HelpPlane, root: &SpecifiedRootEnum) {
        for variant in root.variants() {
            let root = HelpRoot::new(
                plane,
                HelpName::from(variant.name()),
                variant
                    .payload()
                    .map(|payload| payload.to_help_source_declaration_value()),
            );
            self.roots.push(root);
        }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpRoots {
    roots: Vec<HelpRoot>,
}

impl HelpRoots {
    fn empty() -> Self {
        Self { roots: Vec::new() }
    }

    fn push(&mut self, root: HelpRoot) {
        self.roots.push(root);
    }

    fn roots(&self) -> &[HelpRoot] {
        &self.roots
    }

    fn find(&self, name: &HelpName) -> Option<&HelpRoot> {
        self.roots.iter().find(|root| root.name() == name)
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpPlane {
    Input,
    Output,
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
pub struct HelpRoot {
    plane: HelpPlane,
    name: HelpName,
    #[rkyv(omit_bounds)]
    body: Option<SourceDeclarationValue>,
}

impl HelpRoot {
    fn new(plane: HelpPlane, name: HelpName, body: Option<SourceDeclarationValue>) -> Self {
        Self { plane, name, body }
    }

    pub fn plane(&self) -> HelpPlane {
        self.plane
    }

    fn name(&self) -> &HelpName {
        &self.name
    }

    fn body(&self) -> Option<&SourceDeclarationValue> {
        self.body.as_ref()
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpNodes {
    nodes: Vec<HelpNode>,
}

impl HelpNodes {
    fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    fn insert(&mut self, node: HelpNode) {
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

    fn find(&self, name: &HelpName) -> Option<&HelpNode> {
        self.nodes.iter().find(|node| node.name() == name)
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
pub struct HelpNode {
    name: HelpName,
    #[rkyv(omit_bounds)]
    body: Option<SourceDeclarationValue>,
}

impl HelpNode {
    fn new(name: HelpName, body: Option<SourceDeclarationValue>) -> Self {
        Self { name, body }
    }

    fn name(&self) -> &HelpName {
        &self.name
    }

    fn body(&self) -> Option<&SourceDeclarationValue> {
        self.body.as_ref()
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpEntries {
    entries: Vec<HelpEntry>,
}

impl HelpEntries {
    fn new(entries: Vec<HelpEntry>) -> Self {
        Self { entries }
    }

    fn single(entry: HelpEntry) -> Self {
        Self::new(vec![entry])
    }

    fn from_roots(roots: &[HelpRoot]) -> Self {
        Self::new(roots.iter().map(HelpEntry::from_root).collect())
    }

    pub fn entries(&self) -> &[HelpEntry] {
        &self.entries
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
    name: HelpName,
    #[rkyv(omit_bounds)]
    body: Option<SourceDeclarationValue>,
}

impl HelpEntry {
    fn new(name: HelpName, body: Option<SourceDeclarationValue>) -> Self {
        Self { name, body }
    }

    fn from_root(root: &HelpRoot) -> Self {
        Self::new(root.name().clone(), root.body().cloned())
    }

    fn from_node(node: &HelpNode) -> Self {
        Self::new(node.name().clone(), node.body().cloned())
    }

    fn from_source_declaration(declaration: &SourceDeclaration) -> Self {
        Self::new(
            HelpName::from(declaration.name()),
            declaration.value().cloned(),
        )
    }

    pub fn name(&self) -> &HelpName {
        &self.name
    }

    /// The schema declaration body projected from the stored `SpecifiedSchema`.
    pub fn body(&self) -> Option<&SourceDeclarationValue> {
        self.body.as_ref()
    }

    /// Re-head the projected body over the entry name as a source declaration
    /// so the schema encoder produces `(Head <body-schema-text>)`.
    fn to_source_declaration(&self) -> SourceDeclaration {
        SourceDeclaration::new(Name::new(self.name.as_str()), self.body.clone())
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

#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct HelpName {
    value: String,
}

impl HelpName {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<&Name> for HelpName {
    fn from(value: &Name) -> Self {
        Self::new(value.as_str())
    }
}

impl fmt::Display for HelpName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
