use std::fmt;

use nota_next::{Block, Delimiter, Document};
use schema_next::{
    Name, SchemaError, SchemaSource, SourceDeclaration, SourceDeclarationValue, SourceDeclarations,
    SourceEnumBody, SourceField, SourceFieldValue, SourceImport, SourceNamespace, SourceReference,
    SourceRootEnum, SourceStructBody, SourceVariantPayload, SourceVariantSignature,
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

    #[error("help expression cannot be represented as a schema declaration: {0}")]
    InvalidExpression(String),
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
    /// [`Self::to_schema_text`]. Each re-headed declaration parses back into a
    /// typed [`HelpEntry`] through schema-next's declaration decoder, never a
    /// hand-rolled delimiter walk.
    pub fn from_schema_text(source: &str) -> Result<Self, HelpError> {
        let declarations = SourceDeclarations::from_schema_text(source)?;
        Self::from_source_declarations(&declarations)
    }

    /// Encode the response as canonical schema text through schema-next's
    /// declaration encoder.
    pub fn to_schema_text(&self) -> Result<String, HelpError> {
        Ok(self.to_source_declarations()?.to_schema_text())
    }

    fn from_source_declarations(declarations: &SourceDeclarations) -> Result<Self, HelpError> {
        declarations
            .declarations()
            .iter()
            .map(HelpEntry::from_source_declaration)
            .collect::<Result<Vec<_>, _>>()
            .map(HelpEntries::new)
            .map(Self::new)
    }

    fn to_source_declarations(&self) -> Result<SourceDeclarations, HelpError> {
        self.entries()
            .entries()
            .iter()
            .map(HelpEntry::to_source_declaration)
            .collect::<Result<Vec<_>, _>>()
            .map(SourceDeclarations::new)
    }
}

impl fmt::Display for HelpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_schema_text() {
            Ok(text) => formatter.write_str(&text),
            Err(_) => Err(fmt::Error),
        }
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
                HelpBody::Reference(HelpTypeExpression::from_reference(import.source())),
            ));
        }
    }

    fn insert_namespace(&mut self, namespace: &SourceNamespace) {
        for entry in namespace.entries() {
            if let Some(value) = entry.value() {
                let name = HelpName::from(entry.name());
                let body = HelpBody::from_declaration(value);
                self.nodes.insert(HelpNode::new(name, body));
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

    fn body_for_root_variant(&self, variant: &SourceVariantSignature) -> HelpBody {
        match variant.payload_source() {
            Some(SourceVariantPayload::Reference(reference)) => {
                HelpBody::Reference(HelpTypeExpression::from_reference(reference))
            }
            Some(SourceVariantPayload::Declaration(value)) => HelpBody::from_declaration(value),
            None => self
                .nodes
                .find(&HelpName::from(variant.name()))
                .map(|node| self.body_for_root_node(node))
                .unwrap_or(HelpBody::Unit),
        }
    }

    fn body_for_root_node(&self, node: &HelpNode) -> HelpBody {
        match node.body() {
            HelpBody::Reference(reference) => reference
                .plain_name()
                .and_then(|name| self.nodes.find(name))
                .and_then(|target| match target.body() {
                    HelpBody::Struct(fields) => Some(HelpBody::Struct(fields.clone())),
                    HelpBody::Enumeration(variants) => {
                        Some(HelpBody::Enumeration(variants.clone()))
                    }
                    HelpBody::Unit | HelpBody::Reference(_) | HelpBody::Text(_) => None,
                })
                .unwrap_or_else(|| HelpBody::Reference(reference.clone())),
            body => body.clone(),
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
                            HelpBody::from_declaration(value),
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
            SourceFieldValue::Reference(reference) => Some(HelpBody::Reference(
                HelpTypeExpression::from_reference(reference),
            )),
            SourceFieldValue::Declaration(value) => {
                self.insert_inline_declarations_from_declaration(value);
                Some(HelpBody::from_declaration(value))
            }
            SourceFieldValue::Derived => None,
        };
        if let Some(body) = body {
            self.nodes
                .insert(HelpNode::new(HelpName::from(field.name()), body));
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
pub struct HelpRoot {
    plane: HelpPlane,
    name: HelpName,
    body: HelpBody,
}

impl HelpRoot {
    fn new(plane: HelpPlane, name: HelpName, body: HelpBody) -> Self {
        Self { plane, name, body }
    }

    fn name(&self) -> &HelpName {
        &self.name
    }

    fn body(&self) -> &HelpBody {
        &self.body
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
pub struct HelpNode {
    name: HelpName,
    body: HelpBody,
}

impl HelpNode {
    fn new(name: HelpName, body: HelpBody) -> Self {
        Self { name, body }
    }

    fn name(&self) -> &HelpName {
        &self.name
    }

    fn body(&self) -> &HelpBody {
        &self.body
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

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpEntry {
    name: HelpName,
    body: HelpBody,
}

impl HelpEntry {
    fn from_root(root: &HelpRoot) -> Self {
        Self {
            name: root.name().clone(),
            body: root.body().clone(),
        }
    }

    fn from_node(node: &HelpNode) -> Self {
        Self {
            name: node.name().clone(),
            body: node.body().clone(),
        }
    }

    fn new(name: HelpName, body: HelpBody) -> Self {
        Self { name, body }
    }

    pub fn name(&self) -> &HelpName {
        &self.name
    }

    pub fn body(&self) -> &HelpBody {
        &self.body
    }

    /// Decode a single help entry from a (re-headed) source declaration — the
    /// `Head` is the entry name, the body is the schema declaration value the
    /// schema decoder already parsed.
    fn from_source_declaration(declaration: &SourceDeclaration) -> Result<Self, HelpError> {
        let body = match declaration.value() {
            Some(value) => HelpBody::from_declaration(value),
            None => HelpBody::Unit,
        };
        Ok(Self::new(HelpName::from(declaration.name()), body))
    }

    /// Re-head the typed body over the entry name as a source declaration so
    /// the schema encoder produces `(Head <body-schema-text>)`.
    fn to_source_declaration(&self) -> Result<SourceDeclaration, HelpError> {
        Ok(SourceDeclaration::new(
            Name::new(self.name.as_str()),
            self.body.to_source_declaration_value()?,
        ))
    }

    /// Encode this entry as canonical schema text through schema-next's
    /// declaration encoder.
    pub fn to_schema_text(&self) -> Result<String, HelpError> {
        Ok(self.to_source_declaration()?.to_schema_text())
    }
}

impl fmt::Display for HelpEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_schema_text() {
            Ok(text) => formatter.write_str(&text),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum HelpBody {
    Unit,
    Reference(HelpTypeExpression),
    Struct(HelpFieldTypes),
    Enumeration(HelpVariantTypes),
    Text(String),
}

impl HelpBody {
    fn from_declaration(value: &SourceDeclarationValue) -> Self {
        match value {
            SourceDeclarationValue::Reference(reference) => {
                Self::Reference(HelpTypeExpression::from_reference(reference))
            }
            SourceDeclarationValue::Struct(body) => Self::from_struct_body(body),
            SourceDeclarationValue::Enum(body) => Self::from_enum_body(body),
            SourceDeclarationValue::Text(text) => Self::Text(text.clone()),
            SourceDeclarationValue::Stream(_) | SourceDeclarationValue::Family(_) => {
                Self::Text(value.to_schema_text())
            }
        }
    }

    fn from_struct_body(body: &SourceStructBody) -> Self {
        Self::Struct(HelpFieldTypes::from_struct_body(body))
    }

    fn from_enum_body(body: &SourceEnumBody) -> Self {
        Self::Enumeration(HelpVariantTypes::from_enum_body(body))
    }

    /// Project the typed body back onto a [`SourceDeclarationValue`] so the
    /// schema encoder owns the text form. `Unit` has no body; the `Text`
    /// fallback (Stream/Family, etc.) re-parses through the schema decoder so
    /// even the escape hatch round-trips at the schema layer.
    fn to_source_declaration_value(&self) -> Result<Option<SourceDeclarationValue>, HelpError> {
        match self {
            Self::Unit => Ok(None),
            Self::Reference(reference) => Ok(Some(SourceDeclarationValue::Reference(
                reference.to_source_reference()?,
            ))),
            Self::Struct(fields) => Ok(Some(SourceDeclarationValue::Struct(
                fields.to_source_struct_body()?,
            ))),
            Self::Enumeration(variants) => Ok(Some(SourceDeclarationValue::Enum(
                variants.to_source_enum_body()?,
            ))),
            Self::Text(text) => SourceDeclarationValue::from_schema_text(text)
                .map(Some)
                .map_err(HelpError::from),
        }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpFieldTypes {
    fields: Vec<HelpTypeExpression>,
}

impl HelpFieldTypes {
    fn from_struct_body(body: &SourceStructBody) -> Self {
        Self {
            fields: body
                .fields()
                .iter()
                .map(HelpTypeExpression::from_field)
                .collect(),
        }
    }

    pub fn fields(&self) -> &[HelpTypeExpression] {
        &self.fields
    }

    fn to_source_struct_body(&self) -> Result<SourceStructBody, HelpError> {
        self.fields
            .iter()
            .map(HelpTypeExpression::to_source_field)
            .collect::<Result<Vec<_>, _>>()
            .map(SourceStructBody::new)
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpVariantTypes {
    variants: Vec<HelpTypeExpression>,
}

impl HelpVariantTypes {
    fn from_enum_body(body: &SourceEnumBody) -> Self {
        Self {
            variants: body
                .variants()
                .iter()
                .map(HelpTypeExpression::from_variant)
                .collect(),
        }
    }

    pub fn variants(&self) -> &[HelpTypeExpression] {
        &self.variants
    }

    fn to_source_enum_body(&self) -> Result<SourceEnumBody, HelpError> {
        self.variants
            .iter()
            .map(HelpTypeExpression::to_source_variant_signature)
            .collect::<Result<Vec<_>, _>>()
            .map(SourceEnumBody::new)
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpTypeExpression {
    expression: HelpTypeExpressionKind,
}

impl HelpTypeExpression {
    fn new(expression: HelpTypeExpressionKind) -> Self {
        Self { expression }
    }

    fn from_name(name: &Name) -> Self {
        Self::new(HelpTypeExpressionKind::Name(HelpName::from(name)))
    }

    fn from_reference(reference: &SourceReference) -> Self {
        match reference {
            SourceReference::Plain(name) => Self::from_name(name),
            SourceReference::FixedBytes(width) => {
                Self::new(HelpTypeExpressionKind::FixedBytes(*width))
            }
            SourceReference::Vector(reference) => Self::new(HelpTypeExpressionKind::Vector(
                Box::new(Self::from_reference(reference)),
            )),
            SourceReference::Optional(reference) => Self::new(HelpTypeExpressionKind::Optional(
                Box::new(Self::from_reference(reference)),
            )),
            SourceReference::ScopeOf(reference) => Self::new(HelpTypeExpressionKind::ScopeOf(
                Box::new(Self::from_reference(reference)),
            )),
            SourceReference::Map(key, value) => Self::new(HelpTypeExpressionKind::Map(
                Box::new(Self::from_reference(key)),
                Box::new(Self::from_reference(value)),
            )),
            SourceReference::Application { head, arguments } => {
                Self::new(HelpTypeExpressionKind::Application {
                    head: HelpName::from(head),
                    arguments: HelpTypeExpressions::from_references(arguments),
                })
            }
        }
    }

    fn from_field(field: &SourceField) -> Self {
        match field.value() {
            SourceFieldValue::Derived => Self::from_name(field.name()),
            SourceFieldValue::Reference(_) if HelpName::from(field.name()).is_type_name() => {
                Self::from_name(field.name())
            }
            SourceFieldValue::Reference(reference) => Self::from_reference(reference),
            SourceFieldValue::Declaration(_) if HelpName::from(field.name()).is_type_name() => {
                Self::from_name(field.name())
            }
            SourceFieldValue::Declaration(value) => Self::inline(value.to_schema_text()),
        }
    }

    fn from_variant(variant: &SourceVariantSignature) -> Self {
        match variant.payload_source() {
            Some(SourceVariantPayload::Reference(reference)) => {
                Self::application(variant.name(), vec![Self::from_reference(reference)])
            }
            Some(SourceVariantPayload::Declaration(_value)) => {
                Self::application(variant.name(), vec![Self::from_name(variant.name())])
            }
            None => Self::from_name(variant.name()),
        }
    }

    pub fn kind(&self) -> &HelpTypeExpressionKind {
        &self.expression
    }

    fn plain_name(&self) -> Option<&HelpName> {
        match &self.expression {
            HelpTypeExpressionKind::Name(name) => Some(name),
            HelpTypeExpressionKind::FixedBytes(_)
            | HelpTypeExpressionKind::Vector(_)
            | HelpTypeExpressionKind::Optional(_)
            | HelpTypeExpressionKind::ScopeOf(_)
            | HelpTypeExpressionKind::Map(..)
            | HelpTypeExpressionKind::Application { .. }
            | HelpTypeExpressionKind::Inline(_) => None,
        }
    }

    /// Project a type expression back onto a [`SourceReference`] so the schema
    /// encoder renders it. An inline declaration has no reference form.
    fn to_source_reference(&self) -> Result<SourceReference, HelpError> {
        match &self.expression {
            HelpTypeExpressionKind::Name(name) => {
                Ok(SourceReference::Plain(Name::new(name.as_str())))
            }
            HelpTypeExpressionKind::FixedBytes(width) => Ok(SourceReference::FixedBytes(*width)),
            HelpTypeExpressionKind::Vector(reference) => Ok(SourceReference::Vector(Box::new(
                reference.to_source_reference()?,
            ))),
            HelpTypeExpressionKind::Optional(reference) => Ok(SourceReference::Optional(Box::new(
                reference.to_source_reference()?,
            ))),
            HelpTypeExpressionKind::ScopeOf(reference) => Ok(SourceReference::ScopeOf(Box::new(
                reference.to_source_reference()?,
            ))),
            HelpTypeExpressionKind::Map(key, value) => Ok(SourceReference::Map(
                Box::new(key.to_source_reference()?),
                Box::new(value.to_source_reference()?),
            )),
            HelpTypeExpressionKind::Application { head, arguments } => {
                Ok(SourceReference::Application {
                    head: Name::new(head.as_str()),
                    arguments: arguments.to_source_references()?,
                })
            }
            HelpTypeExpressionKind::Inline(text) => {
                Err(HelpError::InvalidExpression(text.clone()))
            }
        }
    }

    /// A struct field in help is a bare schema role name; the field's declared
    /// type is the node with that name.
    fn to_source_field(&self) -> Result<SourceField, HelpError> {
        match &self.expression {
            HelpTypeExpressionKind::Name(name) => {
                Ok(SourceField::derived(Name::new(name.as_str())))
            }
            other => Err(HelpError::InvalidExpression(format!(
                "struct field help must be a schema role name, found {other:?}"
            ))),
        }
    }

    fn to_source_variant_signature(&self) -> Result<SourceVariantSignature, HelpError> {
        match &self.expression {
            HelpTypeExpressionKind::Name(name) => {
                Ok(SourceVariantSignature::from_name(Name::new(name.as_str())))
            }
            HelpTypeExpressionKind::Application { head, arguments } => {
                match arguments.expressions() {
                    [] => Ok(SourceVariantSignature::from_name(Name::new(head.as_str()))),
                    [payload] => Ok(SourceVariantSignature::from_payload(
                        Name::new(head.as_str()),
                        SourceVariantPayload::Reference(payload.to_source_reference()?),
                    )),
                    arguments => Err(HelpError::InvalidExpression(format!(
                        "enum variant help accepts zero or one payload reference, found {}",
                        arguments.len()
                    ))),
                }
            }
            other => Err(HelpError::InvalidExpression(format!(
                "enum variant help must be a schema variant signature, found {other:?}"
            ))),
        }
    }

    fn inline(text: impl Into<String>) -> Self {
        Self::new(HelpTypeExpressionKind::Inline(text.into()))
    }

    fn application(name: &Name, arguments: Vec<Self>) -> Self {
        Self::new(HelpTypeExpressionKind::Application {
            head: HelpName::from(name),
            arguments: HelpTypeExpressions::new(arguments),
        })
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
pub enum HelpTypeExpressionKind {
    Name(HelpName),
    FixedBytes(u64),
    Vector(#[rkyv(omit_bounds)] Box<HelpTypeExpression>),
    Optional(#[rkyv(omit_bounds)] Box<HelpTypeExpression>),
    ScopeOf(#[rkyv(omit_bounds)] Box<HelpTypeExpression>),
    Map(
        #[rkyv(omit_bounds)] Box<HelpTypeExpression>,
        #[rkyv(omit_bounds)] Box<HelpTypeExpression>,
    ),
    Application {
        head: HelpName,
        #[rkyv(omit_bounds)]
        arguments: HelpTypeExpressions,
    },
    Inline(String),
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
pub struct HelpTypeExpressions {
    #[rkyv(omit_bounds)]
    expressions: Vec<HelpTypeExpression>,
}

impl HelpTypeExpressions {
    fn new(expressions: Vec<HelpTypeExpression>) -> Self {
        Self { expressions }
    }

    fn from_references(references: &[SourceReference]) -> Self {
        Self {
            expressions: references
                .iter()
                .map(HelpTypeExpression::from_reference)
                .collect(),
        }
    }

    pub fn expressions(&self) -> &[HelpTypeExpression] {
        &self.expressions
    }

    fn to_source_references(&self) -> Result<Vec<SourceReference>, HelpError> {
        self.expressions
            .iter()
            .map(HelpTypeExpression::to_source_reference)
            .collect()
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
