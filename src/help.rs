//! Help as a thin typed view over `schema`'s TrueSchema model.
//!
//! Help is not a separate language and carries no AST of its own. The model
//! stores [`TrueSchema`] values — authored `.schema` sugar decoded into the
//! canonical semantic data tree. Rendering is a final projection from that
//! typed data into positional schema rows encoded by schema's declaration-body
//! codec.
//!
//! The text codec is schema's body codec end to end: each [`HelpBody`] encodes
//! via [`SourceDeclarationValue::to_schema_text`] and decodes via
//! [`SourceDeclarationValue::from_block`]. No hand `format!` printer, no
//! parallel decoder. The rkyv codec is the rkyv derive on the Help wrappers and
//! on the stored [`TrueSchema`] values.

use std::fmt;

use ::schema::{
    EnumDeclaration, EnumVariant, FamilyDeclaration, ImportResolver, Name, Root, SchemaEngine,
    SchemaError, SchemaIdentity, SchemaSource, SourceDeclarationValue, SourceEnumBody,
    SourceFamilyBody, SourceField, SourceReference, SourceStreamBody, SourceStructBody,
    SourceVariantPayload, SourceVariantSignature, StreamDeclaration, TrueSchema, TypeDeclaration,
    TypeReference,
};
use nota::{Block, Delimiter, Document};
use thiserror::Error;

use crate::{DOMAIN_SCHEMA_SOURCE, SIGNAL_SCHEMA_SOURCE};

#[derive(Debug, Error)]
pub enum HelpError {
    #[error("schema source error: {0}")]
    Schema(#[from] SchemaError),

    #[error("NOTA parse error: {0}")]
    Nota(#[from] nota::NotaError),

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

/// A help response is a list of typed help entries. Rendering projects only the
/// row bodies: no `(Name Body)` wrapper, no field labels, and no name/type
/// pairs. The entry names remain available to callers that need machine
/// navigation.
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

    /// Decode a displayed help response from positional schema rows — the
    /// inverse of [`Self::to_schema_text`]. Each row parses straight back into
    /// schema declaration-body data through schema's own body decoder.
    pub fn from_schema_text(source: &str) -> Result<Self, HelpError> {
        let document = Document::parse(source)?;
        let rows = document
            .root_objects()
            .iter()
            .map(HelpBody::from_block)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(vec![HelpEntry::new(Name::new("Help"), rows)]))
    }

    /// Encode the response as positional schema rows through schema's body
    /// encoder. This is the final display boundary.
    pub fn to_schema_text(&self) -> String {
        self.entries
            .iter()
            .flat_map(HelpEntry::rows)
            .map(HelpBody::to_schema_text)
            .collect::<Vec<_>>()
            .join("\n")
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
    schemas: Vec<TrueSchema>,
}

impl HelpModel {
    pub fn from_signal_schema_source() -> Result<Self, HelpError> {
        let engine = SchemaEngine::default();
        let resolver = ImportResolver::new().with_module_source(
            "signal-domain",
            "domain",
            "0.1.0",
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
            SchemaIdentity::new("signal-domain:domain", "0.1.0"),
        )?;
        Ok(Self::from_true_schemas(vec![signal_schema, domain_schema]))
    }

    pub fn from_true_schemas(schemas: Vec<TrueSchema>) -> Self {
        Self { schemas }
    }

    pub fn schemas(&self) -> &[TrueSchema] {
        &self.schemas
    }

    pub fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
        HelpCatalog::from_schemas(self.schemas()).render(request)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HelpCatalog<'schema> {
    schemas: &'schema [TrueSchema],
}

impl<'schema> HelpCatalog<'schema> {
    fn from_schemas(schemas: &'schema [TrueSchema]) -> Self {
        Self { schemas }
    }

    fn render(&self, request: &HelpRequest) -> Result<HelpResponse, HelpError> {
        match request.target() {
            None => Ok(HelpResponse::new(self.root_entries())),
            Some(target) => self
                .entry_named(target)
                .map(|entry| HelpResponse::new(vec![entry]))
                .ok_or_else(|| HelpError::UnknownTarget(target.as_str().to_owned())),
        }
    }

    fn root_entries(&self) -> Vec<HelpEntry> {
        self.schemas
            .iter()
            .flat_map(TrueSchema::input_and_output)
            .filter_map(Root::as_enum)
            .flat_map(|root| root.variants.iter())
            .map(|variant| self.entry_for_root_variant(variant))
            .collect()
    }

    fn entry_named(&self, target: &Name) -> Option<HelpEntry> {
        self.root_variant_named(target)
            .map(|variant| self.entry_for_root_variant(variant))
            .or_else(|| {
                self.root_named(target)
                    .map(|root| self.entry_for_root(root))
            })
            .or_else(|| {
                self.type_declaration_named(target)
                    .map(|declaration| self.entry_for_type_declaration(target.clone(), declaration))
            })
            .or_else(|| {
                self.stream_named(target)
                    .map(|stream| HelpEntry::one(target.clone(), HelpBody::from_stream(stream)))
            })
            .or_else(|| {
                self.family_named(target)
                    .map(|family| HelpEntry::one(target.clone(), HelpBody::from_family(family)))
            })
    }

    fn entry_for_root(&self, root: &Root) -> HelpEntry {
        match root {
            Root::Enum(root) => HelpEntry::one(root.name.clone(), HelpBody::from_enum(root)),
            Root::Application(application) => HelpEntry::one(
                application.name().clone(),
                HelpBody::from_type_reference(&TypeReference::from(application.as_ref())),
            ),
        }
    }

    fn entry_for_root_variant(&self, variant: &EnumVariant) -> HelpEntry {
        let rows = variant
            .payload
            .as_ref()
            .map(|payload| self.rows_for_root_payload(payload))
            .unwrap_or_else(|| {
                vec![HelpBody::from_type_reference(&TypeReference::Plain(
                    variant.name.clone(),
                ))]
            });
        HelpEntry::new(variant.name.clone(), rows)
    }

    fn entry_for_type_declaration(&self, name: Name, declaration: &TypeDeclaration) -> HelpEntry {
        HelpEntry::new(name, self.rows_for_type_declaration(declaration))
    }

    fn rows_for_type_declaration(&self, declaration: &TypeDeclaration) -> Vec<HelpBody> {
        let mut rows = vec![HelpBody::from_type_declaration(declaration)];
        if let TypeDeclaration::Struct(declaration) = declaration {
            rows.extend(
                declaration
                    .fields
                    .iter()
                    .map(|field| self.body_for_reference(&field.reference)),
            );
        }
        rows
    }

    fn rows_for_root_payload(&self, reference: &TypeReference) -> Vec<HelpBody> {
        match reference
            .plain_name()
            .and_then(|name| self.type_declaration_named(name))
        {
            Some(TypeDeclaration::Newtype(declaration)) => {
                self.rows_for_root_wrapper_reference(&declaration.reference)
            }
            Some(declaration) => self.rows_for_type_declaration(declaration),
            None => vec![HelpBody::from_type_reference(reference)],
        }
    }

    fn rows_for_root_wrapper_reference(&self, reference: &TypeReference) -> Vec<HelpBody> {
        match reference
            .plain_name()
            .and_then(|name| self.type_declaration_named(name))
        {
            Some(TypeDeclaration::Struct(declaration)) => {
                let mut rows = vec![HelpBody::from_struct(declaration)];
                rows.extend(
                    declaration
                        .fields
                        .iter()
                        .map(|field| self.body_for_reference(&field.reference)),
                );
                rows
            }
            Some(TypeDeclaration::Enum(declaration)) => vec![HelpBody::from_enum(declaration)],
            Some(TypeDeclaration::Newtype(_)) | None => {
                vec![HelpBody::from_type_reference(reference)]
            }
        }
    }

    fn body_for_reference(&self, reference: &TypeReference) -> HelpBody {
        reference
            .plain_name()
            .and_then(|name| self.type_declaration_named(name))
            .map(HelpBody::from_type_declaration)
            .unwrap_or_else(|| HelpBody::from_type_reference(reference))
    }

    fn root_named(&self, target: &Name) -> Option<&'schema Root> {
        self.schemas
            .iter()
            .find_map(|schema| schema.root_named(target.as_str()))
    }

    fn root_variant_named(&self, target: &Name) -> Option<&'schema EnumVariant> {
        self.schemas
            .iter()
            .flat_map(TrueSchema::input_and_output)
            .filter_map(Root::as_enum)
            .flat_map(|root| root.variants.iter())
            .find(|variant| variant.name == *target)
    }

    fn type_declaration_named(&self, target: &Name) -> Option<&'schema TypeDeclaration> {
        self.schemas
            .iter()
            .find_map(|schema| schema.type_named(target.as_str()))
    }

    fn stream_named(&self, target: &Name) -> Option<&'schema StreamDeclaration> {
        self.schemas
            .iter()
            .flat_map(TrueSchema::streams)
            .find(|stream| stream.name == *target)
    }

    fn family_named(&self, target: &Name) -> Option<&'schema FamilyDeclaration> {
        self.schemas
            .iter()
            .flat_map(TrueSchema::families)
            .find(|family| family.name == *target)
    }
}

/// A single named help subject with positional rows. The name is navigation
/// metadata; rendering emits rows only.
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
    rows: Vec<HelpBody>,
}

impl HelpEntry {
    fn new(name: Name, rows: Vec<HelpBody>) -> Self {
        Self { name, rows }
    }

    fn one(name: Name, body: HelpBody) -> Self {
        Self::new(name, vec![body])
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    /// The first positional row projected for this help subject.
    pub fn body(&self) -> Option<&HelpBody> {
        self.rows.first()
    }

    pub fn rows(&self) -> &[HelpBody] {
        &self.rows
    }

    /// Encode this entry as positional schema rows through schema's body
    /// encoder.
    pub fn to_schema_text(&self) -> String {
        self.rows
            .iter()
            .map(HelpBody::to_schema_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl fmt::Display for HelpEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_schema_text())
    }
}

/// A Help response row owned by `signal-spirit`.
///
/// The schema-codec value is kept private so clients do not consume schema
/// source nouns as the public Help API. It is typed declaration-body data, not a
/// rendered string.
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
    fn new(value: SourceDeclarationValue) -> Self {
        Self { value }
    }

    fn from_block(block: &Block) -> Result<Self, HelpError> {
        Ok(Self::new(SourceDeclarationValue::from_block(block)?))
    }

    fn from_type_declaration(declaration: &TypeDeclaration) -> Self {
        match declaration {
            TypeDeclaration::Struct(declaration) => Self::from_struct(declaration),
            TypeDeclaration::Enum(declaration) => Self::from_enum(declaration),
            TypeDeclaration::Newtype(declaration) => {
                Self::from_type_reference(&declaration.reference)
            }
        }
    }

    fn from_struct(declaration: &::schema::StructDeclaration) -> Self {
        Self::new(SourceDeclarationValue::Struct(SourceStructBody::new(
            declaration
                .fields
                .iter()
                .map(|field| SourceField::from_type_reference(field.name.clone(), &field.reference))
                .collect(),
        )))
    }

    fn from_enum(declaration: &EnumDeclaration) -> Self {
        Self::new(SourceDeclarationValue::Enum(SourceEnumBody::new(
            declaration
                .variants
                .iter()
                .map(Self::source_variant_from_enum_variant)
                .collect(),
        )))
    }

    fn from_stream(stream: &StreamDeclaration) -> Self {
        Self::new(SourceDeclarationValue::Stream(SourceStreamBody::new(
            SourceReference::from_type_reference(&stream.token),
            SourceReference::from_type_reference(&stream.opened),
            SourceReference::from_type_reference(&stream.event),
            SourceReference::from_type_reference(&stream.close),
        )))
    }

    fn from_family(family: &FamilyDeclaration) -> Self {
        Self::new(SourceDeclarationValue::Family(SourceFamilyBody::new(
            family.record.clone(),
            family.table.clone(),
            family.key,
        )))
    }

    fn from_type_reference(reference: &TypeReference) -> Self {
        Self::new(SourceDeclarationValue::Reference(
            SourceReference::from_type_reference(reference),
        ))
    }

    fn source_variant_from_enum_variant(variant: &EnumVariant) -> SourceVariantSignature {
        SourceVariantSignature::from_projected(
            variant.name.clone(),
            variant.payload.as_ref().map(|payload| {
                SourceVariantPayload::Reference(SourceReference::from_type_reference(payload))
            }),
            variant.stream_relation.as_ref(),
        )
    }

    pub fn to_schema_text(&self) -> String {
        self.value.to_schema_text()
    }
}
