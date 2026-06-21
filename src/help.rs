use std::fmt;

use nota_next::{Block, Delimiter, Document};
use schema_next::{
    Name, SchemaError, SchemaSource, SourceDeclarationValue, SourceEnumBody, SourceField,
    SourceFieldValue, SourceImport, SourceNamespace, SourceReference, SourceRootEnum,
    SourceStructBody, SourceVariantPayload, SourceVariantSignature,
};
use thiserror::Error;

use crate::SIGNAL_SCHEMA_SOURCE;

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
    lines: Vec<String>,
}

impl HelpResponse {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl fmt::Display for HelpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.lines.join("\n"))
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpModel {
    roots: HelpRoots,
    nodes: HelpNodes,
}

impl HelpModel {
    pub fn from_signal_schema_source() -> Result<Self, HelpError> {
        let source = SchemaSource::from_schema_text(SIGNAL_SCHEMA_SOURCE)?;
        Ok(HelpModelBuilder::from_source(&source).into_model())
    }

    pub fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
        match request.target() {
            None => Ok(HelpResponse::new(
                self.roots
                    .roots()
                    .iter()
                    .map(HelpRoot::render)
                    .collect::<Vec<_>>(),
            )),
            Some(target) => self
                .roots
                .find(target)
                .map(|root| HelpResponse::new(vec![root.render()]))
                .or_else(|| {
                    self.nodes
                        .find(target)
                        .map(|node| HelpResponse::new(vec![node.render()]))
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

    fn render(&self) -> String {
        self.body.render_with_name(&self.name)
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

    fn render(&self) -> String {
        self.body.render_with_name(&self.name)
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

    fn render_with_name(&self, name: &HelpName) -> String {
        match self {
            Self::Unit => format!("({name})"),
            Self::Reference(reference) => format!("({name} {})", reference.render()),
            Self::Struct(fields) => format!("({name} {})", fields.render()),
            Self::Enumeration(variants) => format!("({name} {})", variants.render()),
            Self::Text(text) => format!("({name} {})", text),
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

    fn render(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(HelpTypeExpression::render)
            .collect::<Vec<_>>()
            .join(" ");
        format!("{{ {fields} }}")
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

    fn render(&self) -> String {
        let variants = self
            .variants
            .iter()
            .map(HelpTypeExpression::render)
            .collect::<Vec<_>>()
            .join(" ");
        format!("[{variants}]")
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct HelpTypeExpression {
    text: String,
    plain_name: Option<HelpName>,
}

impl HelpTypeExpression {
    fn new(text: impl Into<String>, plain_name: Option<HelpName>) -> Self {
        Self {
            text: text.into(),
            plain_name,
        }
    }

    fn from_name(name: &Name) -> Self {
        Self::new(name.as_str(), Some(HelpName::from(name)))
    }

    fn from_reference(reference: &SourceReference) -> Self {
        match reference {
            SourceReference::Plain(name) => Self::from_name(name),
            _ => Self::new(reference.to_schema_text(), None),
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
            SourceFieldValue::Declaration(value) => Self::new(value.to_schema_text(), None),
        }
    }

    fn from_variant(variant: &SourceVariantSignature) -> Self {
        match variant.payload_source() {
            Some(SourceVariantPayload::Reference(reference)) => Self::new(
                format!("({} {})", variant.name(), reference.to_schema_text()),
                None,
            ),
            Some(SourceVariantPayload::Declaration(value)) => Self::new(
                format!("({} {})", variant.name(), value.to_schema_text()),
                None,
            ),
            None => Self::from_name(variant.name()),
        }
    }

    fn plain_name(&self) -> Option<&HelpName> {
        self.plain_name.as_ref()
    }

    fn render(&self) -> String {
        self.text.clone()
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
