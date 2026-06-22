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
    declarations: SourceDeclarations,
}

impl HelpResponse {
    pub fn new(declarations: SourceDeclarations) -> Self {
        Self { declarations }
    }

    pub fn single(entry: HelpEntry) -> Self {
        Self::new(SourceDeclarations::new(vec![entry.into_declaration()]))
    }

    pub fn declarations(&self) -> &SourceDeclarations {
        &self.declarations
    }

    pub fn entries(&self) -> HelpEntries<'_> {
        HelpEntries::new(self.declarations.declarations())
    }

    pub fn from_schema_text(source: &str) -> Result<Self, HelpError> {
        SourceDeclarations::from_schema_text(source)
            .map(Self::new)
            .map_err(HelpError::from)
    }

    pub fn to_schema_text(&self) -> Result<String, HelpError> {
        Ok(self.declarations.to_schema_text())
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
            None => Ok(HelpResponse::new(SourceDeclarations::new(
                self.roots
                    .roots()
                    .iter()
                    .map(HelpRoot::to_source_declaration)
                    .collect(),
            ))),
            Some(target) => self
                .roots
                .find(target)
                .map(|root| HelpResponse::single(HelpEntry::from_root(root)))
                .or_else(|| {
                    self.nodes
                        .find(target)
                        .map(|node| HelpResponse::single(HelpEntry::from_node(node)))
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
            self.nodes.insert(HelpNode::new(SourceDeclaration::new(
                import.local_name().clone(),
                Some(SourceDeclarationValue::Reference(import.source().clone())),
            )));
        }
    }

    fn insert_namespace(&mut self, namespace: &SourceNamespace) {
        for entry in namespace.entries() {
            if let Some(value) = entry.value() {
                self.nodes.insert(HelpNode::new(SourceDeclaration::new(
                    entry.name().clone(),
                    Some(self.value_for_help_declaration(value)),
                )));
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
                let declaration = SourceDeclaration::new(
                    variant.name().clone(),
                    self.value_for_root_variant(variant),
                );
                self.roots.push(HelpRoot::new(plane, declaration));
            }
        }
    }

    fn value_for_root_variant(
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
                .and_then(|node| self.value_for_root_node(node)),
        }
    }

    fn value_for_root_node(&self, node: &HelpNode) -> Option<SourceDeclarationValue> {
        let Some(value) = node.declaration().value() else {
            return None;
        };
        match value {
            SourceDeclarationValue::Reference(reference) => self
                .plain_reference_name(reference)
                .and_then(|name| self.nodes.find(&name))
                .and_then(HelpNode::schema_body_value)
                .or_else(|| Some(value.clone())),
            SourceDeclarationValue::Struct(_) | SourceDeclarationValue::Enum(_) => {
                Some(value.clone())
            }
            SourceDeclarationValue::Text(_)
            | SourceDeclarationValue::Stream(_)
            | SourceDeclarationValue::Family(_) => Some(value.clone()),
        }
    }

    fn plain_reference_name(&self, reference: &SourceReference) -> Option<HelpName> {
        match reference {
            SourceReference::Plain(name) => Some(HelpName::from(name)),
            SourceReference::FixedBytes(_)
            | SourceReference::Vector(_)
            | SourceReference::Optional(_)
            | SourceReference::ScopeOf(_)
            | SourceReference::Map(..)
            | SourceReference::Application { .. } => None,
        }
    }

    fn insert_inline_declarations_from_declaration(&mut self, value: &SourceDeclarationValue) {
        match value {
            SourceDeclarationValue::Struct(body) => {
                for field in body.fields() {
                    self.insert_inline_declaration_from_field(field.name(), field.value());
                }
            }
            SourceDeclarationValue::Enum(body) => {
                for variant in body.variants() {
                    if let Some(SourceVariantPayload::Declaration(value)) = variant.payload_source()
                    {
                        self.nodes.insert(HelpNode::new(SourceDeclaration::new(
                            variant.name().clone(),
                            Some(self.value_for_help_declaration(value)),
                        )));
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

    fn insert_inline_declaration_from_field(&mut self, name: &Name, value: &SourceFieldValue) {
        if !HelpName::from(name).is_type_name() {
            return;
        }
        let value = match value {
            SourceFieldValue::Reference(reference) => {
                Some(SourceDeclarationValue::Reference(reference.clone()))
            }
            SourceFieldValue::Declaration(value) => {
                self.insert_inline_declarations_from_declaration(value);
                Some(value.clone())
            }
            SourceFieldValue::Derived => None,
        };
        if let Some(value) = value {
            self.nodes.insert(HelpNode::new(SourceDeclaration::new(
                name.clone(),
                Some(self.value_for_help_declaration(&value)),
            )));
        }
    }

    fn value_for_help_declaration(&self, value: &SourceDeclarationValue) -> SourceDeclarationValue {
        match value {
            SourceDeclarationValue::Reference(reference) => {
                SourceDeclarationValue::Reference(reference.clone())
            }
            SourceDeclarationValue::Struct(body) => {
                SourceDeclarationValue::Struct(SourceStructBody::new(
                    body.fields()
                        .iter()
                        .map(|field| self.field_for_help_declaration(field))
                        .collect(),
                ))
            }
            SourceDeclarationValue::Enum(body) => {
                SourceDeclarationValue::Enum(SourceEnumBody::new(
                    body.variants()
                        .iter()
                        .map(|variant| self.variant_for_help_declaration(variant))
                        .collect(),
                ))
            }
            SourceDeclarationValue::Text(text) => SourceDeclarationValue::Text(text.clone()),
            SourceDeclarationValue::Stream(body) => SourceDeclarationValue::Stream(body.clone()),
            SourceDeclarationValue::Family(body) => SourceDeclarationValue::Family(body.clone()),
        }
    }

    fn field_for_help_declaration(&self, field: &SourceField) -> SourceField {
        if HelpName::from(field.name()).is_type_name() {
            return SourceField::derived(field.name().clone());
        }
        match field.value() {
            SourceFieldValue::Reference(reference) => {
                SourceField::from_reference(field.name().clone(), reference.clone())
            }
            SourceFieldValue::Declaration(_) | SourceFieldValue::Derived => {
                SourceField::derived(field.name().clone())
            }
        }
    }

    fn variant_for_help_declaration(
        &self,
        variant: &SourceVariantSignature,
    ) -> SourceVariantSignature {
        match variant.payload_source() {
            Some(SourceVariantPayload::Reference(reference)) => {
                SourceVariantSignature::from_payload(
                    variant.name().clone(),
                    SourceVariantPayload::Reference(reference.clone()),
                )
            }
            Some(SourceVariantPayload::Declaration(_)) => SourceVariantSignature::from_payload(
                variant.name().clone(),
                SourceVariantPayload::Reference(SourceReference::Plain(variant.name().clone())),
            ),
            None => SourceVariantSignature::from_name(variant.name().clone()),
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
    declaration: SourceDeclaration,
}

impl HelpRoot {
    fn new(plane: HelpPlane, declaration: SourceDeclaration) -> Self {
        Self { plane, declaration }
    }

    fn name(&self) -> &Name {
        self.declaration.name()
    }

    fn declaration(&self) -> &SourceDeclaration {
        &self.declaration
    }

    fn to_source_declaration(&self) -> SourceDeclaration {
        self.declaration.clone()
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
    declaration: SourceDeclaration,
}

impl HelpNode {
    fn new(declaration: SourceDeclaration) -> Self {
        Self { declaration }
    }

    fn name(&self) -> &Name {
        self.declaration.name()
    }

    fn declaration(&self) -> &SourceDeclaration {
        &self.declaration
    }

    fn schema_body_value(&self) -> Option<SourceDeclarationValue> {
        match self.declaration.value()? {
            SourceDeclarationValue::Struct(_) | SourceDeclarationValue::Enum(_) => {
                self.declaration.value().cloned()
            }
            SourceDeclarationValue::Reference(_)
            | SourceDeclarationValue::Text(_)
            | SourceDeclarationValue::Stream(_)
            | SourceDeclarationValue::Family(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HelpEntries<'declarations> {
    entries: &'declarations [SourceDeclaration],
}

impl<'declarations> HelpEntries<'declarations> {
    fn new(entries: &'declarations [SourceDeclaration]) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[SourceDeclaration] {
        self.entries
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpEntry {
    declaration: SourceDeclaration,
}

impl HelpEntry {
    fn from_root(root: &HelpRoot) -> Self {
        Self {
            declaration: root.declaration().clone(),
        }
    }

    fn from_node(node: &HelpNode) -> Self {
        Self {
            declaration: node.declaration().clone(),
        }
    }

    fn into_declaration(self) -> SourceDeclaration {
        self.declaration
    }

    pub fn declaration(&self) -> &SourceDeclaration {
        &self.declaration
    }

    pub fn to_schema_text(&self) -> Result<String, HelpError> {
        Ok(self.declaration.to_schema_text())
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

impl PartialEq<HelpName> for Name {
    fn eq(&self, other: &HelpName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl fmt::Display for HelpName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
