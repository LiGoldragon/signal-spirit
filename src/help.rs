//! Help as a thin view over the resolved schema IR.
//!
//! Help is not a separate language and carries no AST of its own. A Help node
//! is a (re-headed) `schema-next` [`SourceDeclaration`]: the entry name plus
//! the declaration's [`SourceDeclarationValue`] body — the SAME resolved IR
//! that instance-schema rendering and Rust lowering read. There is no
//! `HelpBody` / `HelpTypeExpression`; those duplicated `SourceDeclarationValue`
//! / `SourceReference`, and that fork is exactly where the `Vec` / `Vector`
//! spelling escaped. With the duplicate gone, `(Help Domains)` and the
//! per-instance schema of an empty `Domains` both project the one
//! `SourceReference::Vector(Plain(Domain))` and render `(Vector Domain)`
//! through the one schema encoder.
//!
//! The text codec is schema-next's declaration codec end to end: encode via
//! [`SourceDeclaration::to_schema_text`], decode via
//! [`SourceDeclarations::from_schema_text`]. No hand `format!` printer, no
//! parallel decoder. The rkyv codec is the rkyv derive on these wrappers over
//! the (rkyv-derived) `SourceDeclarationValue`.

use std::fmt;

use nota_next::{Block, Delimiter, Document};
use schema_next::{
    Name, SchemaError, SchemaSource, SourceDeclaration, SourceDeclarationValue, SourceDeclarations,
    SourceField, SourceFieldValue, SourceImport, SourceNamespace, SourceRootEnum,
    SourceVariantPayload, SourceVariantSignature,
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
    /// back into the resolved IR through schema-next's declaration decoder; the
    /// help entry is that [`SourceDeclaration`] with no intermediate AST.
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
    roots: HelpRoots,
    nodes: HelpNodes,
}

impl HelpModel {
    pub fn from_signal_schema_source() -> Result<Self, HelpError> {
        let signal_source = SchemaSource::from_schema_text(SIGNAL_SCHEMA_SOURCE)?;
        let domain_source = SchemaSource::from_schema_text(DOMAIN_SCHEMA_SOURCE)?;
        let mut builder = HelpModelBuilder::from_source(&signal_source);
        builder.insert_namespace(domain_source.namespace());
        Ok(builder.into_model())
    }

    pub fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
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
struct HelpModelBuilder {
    roots: HelpRoots,
    nodes: HelpNodes,
}

impl HelpModelBuilder {
    fn from_source(source: &SchemaSource) -> Self {
        let mut builder = Self {
            roots: HelpRoots::empty(),
            nodes: HelpNodes::empty(),
        };
        builder.insert_imports(source.imports().entries());
        builder.insert_namespace(source.namespace());
        builder.insert_root(HelpPlane::Input, source.input());
        builder.insert_root(HelpPlane::Output, source.output());
        builder
    }

    fn into_model(self) -> HelpModel {
        HelpModel {
            roots: self.roots,
            nodes: self.nodes,
        }
    }

    fn insert_imports(&mut self, imports: &[SourceImport]) {
        for import in imports {
            self.nodes.insert(HelpNode::new(
                HelpName::from(import.local_name()),
                Some(SourceDeclarationValue::Reference(import.source().clone())),
            ));
        }
    }

    fn insert_namespace(&mut self, namespace: &SourceNamespace) {
        for entry in namespace.entries() {
            if let Some(value) = entry.value() {
                let name = HelpName::from(entry.name());
                self.nodes.insert(HelpNode::new(name, Some(value.clone())));
                self.insert_inline_declarations_from_declaration(value);
            }
            if let Some(child_namespace) = entry.namespace() {
                self.insert_namespace(child_namespace);
            }
        }
    }

    fn insert_root(&mut self, plane: HelpPlane, root: &SourceRootEnum) {
        if let Some(body) = root.body().as_enum() {
            for variant in body.variants() {
                let root = HelpRoot::new(
                    plane,
                    HelpName::from(variant.name()),
                    self.body_for_root_variant(variant),
                );
                self.roots.push(root);
            }
        }
    }

    /// Resolve a root variant to the resolved-IR body Help shows for it. A
    /// reference / declaration payload is shown directly; a unit variant that
    /// names a node resolves one level to that node's declared shape (so
    /// `Record` shows its struct, not a bare reference).
    fn body_for_root_variant(
        &self,
        variant: &SourceVariantSignature,
    ) -> Option<SourceDeclarationValue> {
        match variant.payload_source() {
            Some(SourceVariantPayload::Reference(reference)) => {
                Some(SourceDeclarationValue::Reference(reference.clone()))
            }
            Some(SourceVariantPayload::Declaration(value)) => Some(value.clone()),
            None => self
                .nodes
                .find(&HelpName::from(variant.name()))
                .and_then(|node| self.body_for_root_node(node)),
        }
    }

    /// One level of name resolution: if a node is a bare reference to a named
    /// type that is itself a struct or enum, show that struct/enum body
    /// (resolved IR), else keep the node's own body.
    fn body_for_root_node(&self, node: &HelpNode) -> Option<SourceDeclarationValue> {
        match node.body() {
            Some(SourceDeclarationValue::Reference(reference)) => reference
                .plain_name()
                .and_then(|name| self.nodes.find(&HelpName::from(name)))
                .and_then(|target| match target.body() {
                    Some(value @ SourceDeclarationValue::Struct(_))
                    | Some(value @ SourceDeclarationValue::Enum(_)) => Some(value.clone()),
                    _ => None,
                })
                .or_else(|| Some(SourceDeclarationValue::Reference(reference.clone()))),
            body => body.cloned(),
        }
    }

    fn insert_inline_declarations_from_declaration(&mut self, value: &SourceDeclarationValue) {
        match value {
            SourceDeclarationValue::Struct(body) => {
                for field in body.fields() {
                    self.insert_inline_declaration_from_field(field);
                }
            }
            SourceDeclarationValue::Enum(body) => {
                for variant in body.variants() {
                    if let Some(SourceVariantPayload::Declaration(value)) = variant.payload_source()
                    {
                        self.nodes.insert(HelpNode::new(
                            HelpName::from(variant.name()),
                            Some(value.clone()),
                        ));
                        self.insert_inline_declarations_from_declaration(value);
                    }
                }
            }
            SourceDeclarationValue::Reference(_)
            | SourceDeclarationValue::Text(_)
            | SourceDeclarationValue::Stream(_)
            | SourceDeclarationValue::Family(_) => {}
        }
    }

    fn insert_inline_declaration_from_field(&mut self, field: &SourceField) {
        if !HelpName::from(field.name()).is_type_name() {
            return;
        }
        let body = match field.value() {
            SourceFieldValue::Reference(reference) => {
                Some(SourceDeclarationValue::Reference(reference.clone()))
            }
            SourceFieldValue::Declaration(value) => {
                self.insert_inline_declarations_from_declaration(value);
                Some(value.clone())
            }
            SourceFieldValue::Derived => None,
        };
        if let Some(body) = body {
            self.nodes
                .insert(HelpNode::new(HelpName::from(field.name()), Some(body)));
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

/// A single help entry: a re-headed schema declaration. The body is the
/// resolved-IR [`SourceDeclarationValue`] verbatim — the same value
/// instance-schema and Rust lowering consume.
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

    /// The resolved-IR body of this entry — the same `SourceDeclarationValue`
    /// instance-schema and Rust lowering read.
    pub fn body(&self) -> Option<&SourceDeclarationValue> {
        self.body.as_ref()
    }

    /// Re-head the resolved-IR body over the entry name as a source declaration
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

    fn is_type_name(&self) -> bool {
        self.value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
            && !matches!(
                self.value.as_str(),
                "String" | "Integer" | "Boolean" | "Path" | "Bytes"
            )
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
