use schema_next::{
    Name, SchemaError, SchemaSource, SourceDeclarationValue, SourceEnumBody, SourceField,
    SourceFieldValue, SourceNamespace, SourceReference, SourceVariantPayload,
};
use thiserror::Error;

use crate::{DOMAIN_SCHEMA_SOURCE, Input, InputRoute, SIGNAL_SCHEMA_SOURCE};

#[derive(Debug, Error)]
pub enum InstanceSchemaError {
    #[error("schema source error: {0}")]
    Schema(#[from] SchemaError),

    #[error("Input root is not an enum")]
    InputRootNotEnum,

    #[error("unknown Input variant in source schema: {0}")]
    UnknownInputVariant(String),
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct InstanceSchema {
    root: InstanceSchemaElement,
}

impl InstanceSchema {
    pub fn new(root: InstanceSchemaElement) -> Self {
        Self { root }
    }

    pub fn from_decoded_input(input: &Input) -> Result<Self, InstanceSchemaError> {
        InstanceSchemaModel::from_signal_schema_source()?.schema_for_input(input)
    }

    pub fn root(&self) -> &InstanceSchemaElement {
        &self.root
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
pub enum InstanceSchemaElement {
    Name(InstanceSchemaName),
    Parenthesized(#[rkyv(omit_bounds)] InstanceSchemaElements),
    Braced(#[rkyv(omit_bounds)] InstanceSchemaElements),
    Bracketed(#[rkyv(omit_bounds)] InstanceSchemaElements),
}

impl InstanceSchemaElement {
    pub fn name(name: impl Into<String>) -> Self {
        Self::Name(InstanceSchemaName::new(name))
    }

    pub fn parenthesized(elements: Vec<Self>) -> Self {
        Self::Parenthesized(InstanceSchemaElements::new(elements))
    }

    pub fn braced(elements: Vec<Self>) -> Self {
        Self::Braced(InstanceSchemaElements::new(elements))
    }

    pub fn bracketed(elements: Vec<Self>) -> Self {
        Self::Bracketed(InstanceSchemaElements::new(elements))
    }

    pub fn as_name(&self) -> Option<&InstanceSchemaName> {
        match self {
            Self::Name(name) => Some(name),
            Self::Parenthesized(_) | Self::Braced(_) | Self::Bracketed(_) => None,
        }
    }

    pub fn as_parenthesized(&self) -> Option<&InstanceSchemaElements> {
        match self {
            Self::Parenthesized(elements) => Some(elements),
            Self::Name(_) | Self::Braced(_) | Self::Bracketed(_) => None,
        }
    }

    pub fn as_braced(&self) -> Option<&InstanceSchemaElements> {
        match self {
            Self::Braced(elements) => Some(elements),
            Self::Name(_) | Self::Parenthesized(_) | Self::Bracketed(_) => None,
        }
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
pub struct InstanceSchemaElements {
    #[rkyv(omit_bounds)]
    elements: Vec<InstanceSchemaElement>,
}

impl InstanceSchemaElements {
    pub fn new(elements: Vec<InstanceSchemaElement>) -> Self {
        Self { elements }
    }

    pub fn elements(&self) -> &[InstanceSchemaElement] {
        &self.elements
    }
}

#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct InstanceSchemaName {
    value: String,
}

impl InstanceSchemaName {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstanceSchemaModel {
    input_root_name: InstanceSchemaName,
    input_variants: SourceEnumBody,
    declarations: InstanceSchemaDeclarations,
}

impl InstanceSchemaModel {
    pub fn from_signal_schema_source() -> Result<Self, InstanceSchemaError> {
        let signal_source = SchemaSource::from_schema_text(SIGNAL_SCHEMA_SOURCE)?;
        let domain_source = SchemaSource::from_schema_text(DOMAIN_SCHEMA_SOURCE)?;
        Self::from_sources(&signal_source, &domain_source)
    }

    pub fn schema_for_input(&self, input: &Input) -> Result<InstanceSchema, InstanceSchemaError> {
        let root_name = self.input_root_name.clone();
        let variant_name = self.input_route_name(input);
        let Some(payload) = self.payload_element_for_input_variant(&variant_name)? else {
            return Ok(InstanceSchema::new(InstanceSchemaElement::name(
                root_name.as_str(),
            )));
        };
        Ok(InstanceSchema::new(InstanceSchemaElement::parenthesized(
            vec![InstanceSchemaElement::name(root_name.as_str()), payload],
        )))
    }

    fn from_sources(
        signal_source: &SchemaSource,
        domain_source: &SchemaSource,
    ) -> Result<Self, InstanceSchemaError> {
        let input_variants = signal_source
            .input()
            .body()
            .as_enum()
            .cloned()
            .ok_or(InstanceSchemaError::InputRootNotEnum)?;
        let mut declarations = InstanceSchemaDeclarations::empty();
        declarations.insert_namespace(signal_source.namespace());
        declarations.insert_namespace(domain_source.namespace());
        Ok(Self {
            input_root_name: InstanceSchemaName::new(signal_source.input().name().as_str()),
            input_variants,
            declarations,
        })
    }

    fn payload_element_for_input_variant(
        &self,
        variant_name: &InstanceSchemaName,
    ) -> Result<Option<InstanceSchemaElement>, InstanceSchemaError> {
        let variant = self
            .input_variants
            .variants()
            .iter()
            .find(|variant| variant.name().as_str() == variant_name.as_str())
            .ok_or_else(|| {
                InstanceSchemaError::UnknownInputVariant(variant_name.as_str().to_owned())
            })?;
        match variant.payload_source() {
            Some(SourceVariantPayload::Reference(reference)) => {
                Ok(Some(self.payload_element_for_reference(reference)))
            }
            Some(SourceVariantPayload::Declaration(value)) => {
                Ok(Some(self.payload_element_for_declaration_value(value)))
            }
            None => Ok(self
                .declarations
                .value_named(variant.name())
                .and_then(|value| self.payload_element_for_root_alias(value))),
        }
    }

    fn payload_element_for_root_alias(
        &self,
        value: &SourceDeclarationValue,
    ) -> Option<InstanceSchemaElement> {
        match value {
            SourceDeclarationValue::Reference(reference) => {
                Some(self.payload_element_for_reference(reference))
            }
            SourceDeclarationValue::Struct(_)
            | SourceDeclarationValue::Enum(_)
            | SourceDeclarationValue::Text(_)
            | SourceDeclarationValue::Stream(_)
            | SourceDeclarationValue::Family(_) => {
                Some(self.payload_element_for_declaration_value(value))
            }
        }
    }

    fn payload_element_for_reference(&self, reference: &SourceReference) -> InstanceSchemaElement {
        match reference {
            SourceReference::Plain(name) => self.payload_element_for_name(name),
            SourceReference::Vector(reference) => {
                InstanceSchemaElement::bracketed(vec![self.leaf_element_for_reference(reference)])
            }
            SourceReference::Optional(reference) | SourceReference::ScopeOf(reference) => {
                InstanceSchemaElement::parenthesized(vec![
                    self.leaf_element_for_reference(reference),
                ])
            }
            SourceReference::Application { head, arguments } => {
                let mut elements = Vec::with_capacity(arguments.len() + 1);
                elements.push(InstanceSchemaElement::name(head.as_str()));
                elements.extend(
                    arguments
                        .iter()
                        .map(|reference| self.leaf_element_for_reference(reference)),
                );
                InstanceSchemaElement::parenthesized(elements)
            }
            SourceReference::Map(key, value) => InstanceSchemaElement::parenthesized(vec![
                self.leaf_element_for_reference(key),
                self.leaf_element_for_reference(value),
            ]),
            SourceReference::FixedBytes(_) => InstanceSchemaElement::name("Bytes"),
        }
    }

    fn payload_element_for_name(&self, name: &Name) -> InstanceSchemaElement {
        match self.declarations.value_named(name) {
            Some(SourceDeclarationValue::Struct(body)) => InstanceSchemaElement::parenthesized(
                body.fields()
                    .iter()
                    .map(|field| self.field_element(field))
                    .collect(),
            ),
            Some(SourceDeclarationValue::Reference(_))
            | Some(SourceDeclarationValue::Enum(_))
            | Some(SourceDeclarationValue::Text(_))
            | Some(SourceDeclarationValue::Stream(_))
            | Some(SourceDeclarationValue::Family(_))
            | None => InstanceSchemaElement::name(name.as_str()),
        }
    }

    fn payload_element_for_declaration_value(
        &self,
        value: &SourceDeclarationValue,
    ) -> InstanceSchemaElement {
        match value {
            SourceDeclarationValue::Reference(reference) => {
                self.payload_element_for_reference(reference)
            }
            SourceDeclarationValue::Struct(body) => InstanceSchemaElement::parenthesized(
                body.fields()
                    .iter()
                    .map(|field| self.field_element(field))
                    .collect(),
            ),
            SourceDeclarationValue::Enum(_)
            | SourceDeclarationValue::Text(_)
            | SourceDeclarationValue::Stream(_)
            | SourceDeclarationValue::Family(_) => InstanceSchemaElement::name("Value"),
        }
    }

    fn field_element(&self, field: &SourceField) -> InstanceSchemaElement {
        match field.value() {
            SourceFieldValue::Derived => self.field_element_for_name(field.name()),
            SourceFieldValue::Reference(reference) => {
                if self.declarations.is_struct_named(field.name()) {
                    self.field_element_for_name(field.name())
                } else {
                    self.leaf_element_for_reference(reference)
                }
            }
            SourceFieldValue::Declaration(value) => match value {
                SourceDeclarationValue::Struct(body) => InstanceSchemaElement::braced(
                    body.fields()
                        .iter()
                        .map(|field| self.field_element(field))
                        .collect(),
                ),
                SourceDeclarationValue::Reference(reference) => {
                    self.leaf_element_for_reference(reference)
                }
                SourceDeclarationValue::Enum(_)
                | SourceDeclarationValue::Text(_)
                | SourceDeclarationValue::Stream(_)
                | SourceDeclarationValue::Family(_) => {
                    InstanceSchemaElement::name(field.name().as_str())
                }
            },
        }
    }

    fn field_element_for_name(&self, name: &Name) -> InstanceSchemaElement {
        match self.declarations.value_named(name) {
            Some(SourceDeclarationValue::Struct(body)) => InstanceSchemaElement::braced(
                body.fields()
                    .iter()
                    .map(|field| self.field_element(field))
                    .collect(),
            ),
            Some(SourceDeclarationValue::Reference(_))
            | Some(SourceDeclarationValue::Enum(_))
            | Some(SourceDeclarationValue::Text(_))
            | Some(SourceDeclarationValue::Stream(_))
            | Some(SourceDeclarationValue::Family(_))
            | None => InstanceSchemaElement::name(name.as_str()),
        }
    }

    fn leaf_element_for_reference(&self, reference: &SourceReference) -> InstanceSchemaElement {
        match reference {
            SourceReference::Plain(name) => InstanceSchemaElement::name(name.as_str()),
            SourceReference::Vector(reference) => {
                InstanceSchemaElement::bracketed(vec![self.leaf_element_for_reference(reference)])
            }
            SourceReference::Optional(reference) | SourceReference::ScopeOf(reference) => {
                InstanceSchemaElement::parenthesized(vec![
                    self.leaf_element_for_reference(reference),
                ])
            }
            SourceReference::Map(key, value) => InstanceSchemaElement::parenthesized(vec![
                self.leaf_element_for_reference(key),
                self.leaf_element_for_reference(value),
            ]),
            SourceReference::Application { head, arguments } => {
                let mut elements = Vec::with_capacity(arguments.len() + 1);
                elements.push(InstanceSchemaElement::name(head.as_str()));
                elements.extend(
                    arguments
                        .iter()
                        .map(|reference| self.leaf_element_for_reference(reference)),
                );
                InstanceSchemaElement::parenthesized(elements)
            }
            SourceReference::FixedBytes(_) => InstanceSchemaElement::name("Bytes"),
        }
    }

    fn input_route_name(&self, input: &Input) -> InstanceSchemaName {
        let name = match input.route() {
            InputRoute::State => "State",
            InputRoute::Record => "Record",
            InputRoute::Propose => "Propose",
            InputRoute::Clarify => "Clarify",
            InputRoute::Supersede => "Supersede",
            InputRoute::Retire => "Retire",
            InputRoute::ResolveClarification => "ResolveClarification",
            InputRoute::Observe => "Observe",
            InputRoute::PublicTextSearch => "PublicTextSearch",
            InputRoute::PublicRecords => "PublicRecords",
            InputRoute::PrivateRecords => "PrivateRecords",
            InputRoute::Lookup => "Lookup",
            InputRoute::Count => "Count",
            InputRoute::Remove => "Remove",
            InputRoute::ChangeCertainty => "ChangeCertainty",
            InputRoute::BumpImportance => "BumpImportance",
            InputRoute::ChangeRecord => "ChangeRecord",
            InputRoute::RegisterReferent => "RegisterReferent",
            InputRoute::LookupStash => "LookupStash",
            InputRoute::CollectRemovalCandidates => "CollectRemovalCandidates",
            InputRoute::Tap => "Tap",
            InputRoute::Untap => "Untap",
            InputRoute::SubscribeIntent => "SubscribeIntent",
            InputRoute::Version => "Version",
            InputRoute::Marker => "Marker",
        };
        InstanceSchemaName::new(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstanceSchemaDeclarations {
    declarations: Vec<InstanceSchemaDeclaration>,
}

impl InstanceSchemaDeclarations {
    fn empty() -> Self {
        Self {
            declarations: Vec::new(),
        }
    }

    fn insert_namespace(&mut self, namespace: &SourceNamespace) {
        for entry in namespace.entries() {
            if let Some(value) = entry.value() {
                self.declarations.push(InstanceSchemaDeclaration::new(
                    entry.name().clone(),
                    value.clone(),
                ));
            }
            if let Some(namespace) = entry.namespace() {
                self.insert_namespace(namespace);
            }
        }
    }

    fn value_named(&self, name: &Name) -> Option<&SourceDeclarationValue> {
        self.declarations
            .iter()
            .find(|declaration| declaration.name() == name)
            .map(InstanceSchemaDeclaration::value)
    }

    fn is_struct_named(&self, name: &Name) -> bool {
        matches!(
            self.value_named(name),
            Some(SourceDeclarationValue::Struct(_))
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstanceSchemaDeclaration {
    name: Name,
    value: SourceDeclarationValue,
}

impl InstanceSchemaDeclaration {
    fn new(name: Name, value: SourceDeclarationValue) -> Self {
        Self { name, value }
    }

    fn name(&self) -> &Name {
        &self.name
    }

    fn value(&self) -> &SourceDeclarationValue {
        &self.value
    }
}
