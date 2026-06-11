//! Signal contract for the ordinary `spirit` surface.
//!
//! This crate carries the peer-callable vocabulary for psyche statements,
//! psyche-state observations, intent-record observations, and subscriptions.
//! Runtime actors, sockets, storage, classifier logic, and downstream
//! meta-policy forwarding live in `spirit`.

#[cfg(feature = "nota-text")]
use nota_next::{Block, Delimiter, NotaBlock, NotaDecode, NotaDecodeError, NotaEncode, NotaString};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;

pub mod migration;

const RECORD_IDENTIFIER_BYTES: usize = 12;
const RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH: usize = 4;
const RECORD_IDENTIFIER_RADIX: u128 = 36;

/// Ordered qualitative strength used for intent certainty and privacy.
#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Magnitude {
    Minimum = 0,
    VeryLow = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    VeryHigh = 5,
    Maximum = 6,
    /// Neutral bottom rung. Kept physically last so old archived
    /// discriminants for the original seven values stay stable.
    Zero = 7,
}

impl Magnitude {
    pub const fn as_record_head(self) -> &'static str {
        match self {
            Self::Minimum => "Minimum",
            Self::VeryLow => "VeryLow",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::VeryHigh => "VeryHigh",
            Self::Maximum => "Maximum",
            Self::Zero => "Zero",
        }
    }

    pub fn from_record_head(name: &str) -> Option<Self> {
        match name {
            "Minimum" => Some(Self::Minimum),
            "VeryLow" => Some(Self::VeryLow),
            "Low" => Some(Self::Low),
            "Medium" => Some(Self::Medium),
            "High" => Some(Self::High),
            "VeryHigh" => Some(Self::VeryHigh),
            "Maximum" => Some(Self::Maximum),
            "Zero" => Some(Self::Zero),
            _ => None,
        }
    }

    const fn order_rank(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::Minimum => 1,
            Self::VeryLow => 2,
            Self::Low => 3,
            Self::Medium => 4,
            Self::High => 5,
            Self::VeryHigh => 6,
            Self::Maximum => 7,
        }
    }
}

impl PartialOrd for Magnitude {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Magnitude {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order_rank().cmp(&other.order_rank())
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StatementText(String);

impl StatementText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topic(String);

impl Topic {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topics(Vec<Topic>);

impl Topics {
    pub fn new(value: Vec<Topic>) -> Self {
        Self(value)
    }

    pub fn single(topic: Topic) -> Self {
        Self(vec![topic])
    }

    pub fn as_slice(&self) -> &[Topic] {
        &self.0
    }

    pub fn contains(&self, topic: &Topic) -> bool {
        self.0.iter().any(|candidate| candidate == topic)
    }

    pub fn contains_any(&self, topics: &Topics) -> bool {
        topics.as_slice().iter().any(|topic| self.contains(topic))
    }

    pub fn contains_all(&self, topics: &Topics) -> bool {
        topics.as_slice().iter().all(|topic| self.contains(topic))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[cfg(feature = "nota-text")]
    fn validate(value: &[Topic]) -> Result<(), NotaDecodeError> {
        if value.is_empty() {
            return Err(NotaDecodeError::Parse(
                "Topics: record must carry at least one topic".to_owned(),
            ));
        }

        let mut seen = std::collections::BTreeSet::<&str>::new();
        for topic in value {
            if !seen.insert(topic.as_str()) {
                return Err(NotaDecodeError::Parse(format!(
                    "Topics: record repeats topic {}",
                    topic.as_str()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for Topics {
    fn to_nota(&self) -> String {
        Delimiter::SquareBracket.wrap(self.0.iter().map(|topic| topic.as_str().to_owned()))
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for Topics {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let value = Vec::<Topic>::from_nota_block(block)?;
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct RecordIdentifier([u8; RECORD_IDENTIFIER_BYTES]);

impl RecordIdentifier {
    pub const fn new(value: u64) -> Self {
        let octets = value.to_be_bytes();
        Self([
            0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
            octets[6], octets[7],
        ])
    }

    pub const fn from_bytes(bytes: [u8; RECORD_IDENTIFIER_BYTES]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; RECORD_IDENTIFIER_BYTES] {
        self.0
    }

    pub fn value(self) -> u128 {
        u128::from_be_bytes([
            0, 0, 0, 0, self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
            self.0[6], self.0[7], self.0[8], self.0[9], self.0[10], self.0[11],
        ])
    }

    pub fn code(self) -> String {
        RecordIdentifierCode::from_identifier(self).into_string()
    }

    #[cfg(feature = "nota-text")]
    pub fn from_code(code: &str) -> Result<Self, NotaDecodeError> {
        RecordIdentifierCode::new(code).into_identifier()
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for RecordIdentifier {
    fn to_nota(&self) -> String {
        NotaString::new(&self.code()).format()
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for RecordIdentifier {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        Self::from_code(&NotaBlock::new(block).parse_string()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordIdentifierCode {
    value: String,
}

impl RecordIdentifierCode {
    fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    fn from_identifier(identifier: RecordIdentifier) -> Self {
        let mut value = identifier.value();
        if value == 0 {
            return Self::new("0".repeat(RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH));
        }

        let mut digits = Vec::new();
        while value > 0 {
            let digit = (value % RECORD_IDENTIFIER_RADIX) as u8;
            digits.push(Self::digit_character(digit));
            value /= RECORD_IDENTIFIER_RADIX;
        }
        while digits.len() < RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH {
            digits.push('0');
        }
        digits.reverse();
        Self::new(digits.into_iter().collect::<String>())
    }

    fn into_string(self) -> String {
        self.value
    }

    #[cfg(feature = "nota-text")]
    fn into_identifier(self) -> Result<RecordIdentifier, NotaDecodeError> {
        if self.value.len() < RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH {
            return Err(NotaDecodeError::Parse(format!(
                "RecordIdentifier: record identifier code must be at least {RECORD_IDENTIFIER_MINIMUM_CODE_LENGTH} characters"
            )));
        }

        let mut value = 0_u128;
        for character in self.value.chars() {
            let digit = Self::digit_value(character)?;
            value = value
                .checked_mul(RECORD_IDENTIFIER_RADIX)
                .and_then(|accumulated| accumulated.checked_add(digit))
                .ok_or_else(|| {
                    NotaDecodeError::Parse(
                        "RecordIdentifier: record identifier exceeds 96-bit range".to_owned(),
                    )
                })?;
        }

        let bytes = value.to_be_bytes();
        if bytes[0..4] != [0, 0, 0, 0] {
            return Err(NotaDecodeError::Parse(
                "RecordIdentifier: record identifier exceeds 96-bit range".to_owned(),
            ));
        }

        Ok(RecordIdentifier::from_bytes([
            bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    fn digit_character(value: u8) -> char {
        match value {
            0..=9 => (b'0' + value) as char,
            10..=35 => (b'a' + (value - 10)) as char,
            _ => unreachable!("base36 digit outside alphabet"),
        }
    }

    #[cfg(feature = "nota-text")]
    fn digit_value(character: char) -> Result<u128, NotaDecodeError> {
        match character {
            '0'..='9' => Ok((character as u8 - b'0') as u128),
            'a'..='z' => Ok((character as u8 - b'a' + 10) as u128),
            _ => Err(NotaDecodeError::Parse(format!(
                "RecordIdentifier: record identifier code uses unsupported character {character:?}; use lowercase base36"
            ))),
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Description(String);

impl Description {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub const fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    #[cfg(feature = "nota-text")]
    fn parse_part(
        part: Option<&str>,
        name: &'static str,
        source: &str,
    ) -> Result<u16, NotaDecodeError> {
        part.ok_or_else(|| NotaDecodeError::Parse(format!("Date: missing {name} in {source:?}")))?
            .parse::<u16>()
            .map_err(|_| NotaDecodeError::Parse(format!("Date: invalid {name} in {source:?}")))
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for Date {
    fn to_nota(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for Date {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let source = NotaBlock::new(block).parse_string()?;
        let mut parts = source.split('-');
        let year = Self::parse_part(parts.next(), "year", &source)?;
        let month = Self::parse_part(parts.next(), "month", &source)?;
        let day = Self::parse_part(parts.next(), "day", &source)?;
        if parts.next().is_some() {
            return Err(NotaDecodeError::Parse(format!(
                "Date: too many fields in {source:?}"
            )));
        }
        let month = u8::try_from(month)
            .map_err(|_| NotaDecodeError::Parse(format!("Date: invalid month in {source:?}")))?;
        let day = u8::try_from(day)
            .map_err(|_| NotaDecodeError::Parse(format!("Date: invalid day in {source:?}")))?;
        Ok(Self { year, month, day })
    }
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Time {
    pub const fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }

    #[cfg(feature = "nota-text")]
    fn parse_part(
        part: Option<&str>,
        name: &'static str,
        source: &str,
    ) -> Result<u8, NotaDecodeError> {
        part.ok_or_else(|| NotaDecodeError::Parse(format!("Time: missing {name} in {source:?}")))?
            .parse::<u8>()
            .map_err(|_| NotaDecodeError::Parse(format!("Time: invalid {name} in {source:?}")))
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for Time {
    fn to_nota(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for Time {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let source = NotaBlock::new(block).parse_string()?;
        let mut parts = source.split(':');
        let hour = Self::parse_part(parts.next(), "hour", &source)?;
        let minute = Self::parse_part(parts.next(), "minute", &source)?;
        let second = Self::parse_part(parts.next(), "second", &source)?;
        if parts.next().is_some() {
            return Err(NotaDecodeError::Parse(format!(
                "Time: too many fields in {source:?}"
            )));
        }
        Ok(Self {
            hour,
            minute,
            second,
        })
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct RecordedTime {
    pub date: Date,
    pub time: Time,
}

impl RecordedTime {
    pub const fn new(date: Date, time: Time) -> Self {
        Self { date, time }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordedTimeRange {
    pub first: RecordedTime,
    pub last: RecordedTime,
}

impl RecordedTimeRange {
    pub const fn new(first: RecordedTime, last: RecordedTime) -> Self {
        Self { first, last }
    }

    pub fn contains(self, recorded_time: RecordedTime) -> bool {
        recorded_time >= self.first && recorded_time <= self.last
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FocusArea(String);

impl FocusArea {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchivePath(String);

impl ArchivePath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigurationPath(String);

impl ConfigurationPath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpiritGuardianProviderName(String);

impl SpiritGuardianProviderName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpiritGuardianModelName(String);

impl SpiritGuardianModelName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpiritGuardianTimeoutMilliseconds(u64);

impl SpiritGuardianTimeoutMilliseconds {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpiritGuardianMaximumOutputTokens(u64);

impl SpiritGuardianMaximumOutputTokens {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn into_u64(self) -> u64 {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct SpiritGuardianAgentConfiguration {
    agent_socket_path: ConfigurationPath,
    provider_name: Option<SpiritGuardianProviderName>,
    model_name: Option<SpiritGuardianModelName>,
    timeout_milliseconds: SpiritGuardianTimeoutMilliseconds,
    maximum_output_tokens: Option<SpiritGuardianMaximumOutputTokens>,
}

impl SpiritGuardianAgentConfiguration {
    pub fn new(
        agent_socket_path: ConfigurationPath,
        provider_name: Option<SpiritGuardianProviderName>,
        model_name: Option<SpiritGuardianModelName>,
        timeout_milliseconds: SpiritGuardianTimeoutMilliseconds,
        maximum_output_tokens: Option<SpiritGuardianMaximumOutputTokens>,
    ) -> Self {
        Self {
            agent_socket_path,
            provider_name,
            model_name,
            timeout_milliseconds,
            maximum_output_tokens,
        }
    }

    pub fn agent_socket_path(&self) -> &str {
        self.agent_socket_path.as_str()
    }

    pub fn provider_name(&self) -> Option<&str> {
        self.provider_name
            .as_ref()
            .map(SpiritGuardianProviderName::as_str)
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name
            .as_ref()
            .map(SpiritGuardianModelName::as_str)
    }

    pub fn timeout_milliseconds(&self) -> u64 {
        self.timeout_milliseconds.into_u64()
    }

    pub fn maximum_output_tokens(&self) -> Option<u64> {
        self.maximum_output_tokens
            .map(SpiritGuardianMaximumOutputTokens::into_u64)
    }
}

/// The ordinary-contract daemon configuration the `spirit` daemon decodes from
/// its single binary startup argument. Per the component-triad pipeline the
/// configuration *type* lives in this contract crate and the daemon imports it;
/// `spirit` wraps this value in its runtime configuration and implements the
/// runtime binding surface on that wrapper.
#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct SpiritDaemonConfiguration {
    socket_path: ConfigurationPath,
    meta_socket_path: Option<ConfigurationPath>,
    database_path: ConfigurationPath,
    trace_socket_path: Option<ConfigurationPath>,
    guardian_agent_configuration: Option<SpiritGuardianAgentConfiguration>,
}

impl SpiritDaemonConfiguration {
    pub fn new(socket_path: ConfigurationPath, database_path: ConfigurationPath) -> Self {
        Self {
            socket_path,
            meta_socket_path: None,
            database_path,
            trace_socket_path: None,
            guardian_agent_configuration: None,
        }
    }

    pub fn with_meta_socket_path(mut self, meta_socket_path: ConfigurationPath) -> Self {
        self.meta_socket_path = Some(meta_socket_path);
        self
    }

    pub fn with_guardian_agent_configuration(
        mut self,
        guardian_agent_configuration: SpiritGuardianAgentConfiguration,
    ) -> Self {
        self.guardian_agent_configuration = Some(guardian_agent_configuration);
        self
    }

    pub fn with_trace_socket_path(mut self, trace_socket_path: ConfigurationPath) -> Self {
        self.trace_socket_path = Some(trace_socket_path);
        self
    }

    pub fn socket_path(&self) -> &str {
        self.socket_path.as_str()
    }

    pub fn meta_socket_path(&self) -> Option<&str> {
        self.meta_socket_path
            .as_ref()
            .map(ConfigurationPath::as_str)
    }

    pub fn database_path(&self) -> &str {
        self.database_path.as_str()
    }

    pub fn trace_socket_path(&self) -> Option<&str> {
        self.trace_socket_path
            .as_ref()
            .map(ConfigurationPath::as_str)
    }

    pub fn guardian_agent_configuration(&self) -> Option<&SpiritGuardianAgentConfiguration> {
        self.guardian_agent_configuration.as_ref()
    }

    pub fn from_rkyv_bytes(bytes: &[u8]) -> Result<Self, SpiritDaemonConfigurationArchiveError> {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)
            .map_err(|_| SpiritDaemonConfigurationArchiveError::Decode)
    }

    pub fn to_rkyv_bytes(&self) -> Result<Vec<u8>, SpiritDaemonConfigurationArchiveError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|_| SpiritDaemonConfigurationArchiveError::Encode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpiritDaemonConfigurationArchiveError {
    #[error("failed to encode spirit daemon configuration archive")]
    Encode,

    #[error("failed to decode spirit daemon configuration archive")]
    Decode,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StateSubscriptionToken {
    pub identifier: u64,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscriptionToken {
    pub identifier: u64,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Decision,
    Principle,
    Correction,
    Clarification,
    Constraint,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationMode {
    SummaryOnly,
    WithProvenance,
}

impl ObservationMode {
    #[cfg(feature = "nota-text")]
    fn block_is_mode(block: &Block) -> bool {
        matches!(
            block.demote_to_string(),
            Some("SummaryOnly" | "WithProvenance" | "DescriptionOnly")
        )
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for ObservationMode {
    fn to_nota(&self) -> String {
        match self {
            Self::SummaryOnly => "SummaryOnly".to_owned(),
            Self::WithProvenance => "WithProvenance".to_owned(),
        }
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for ObservationMode {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        match block.demote_to_string() {
            Some("SummaryOnly" | "DescriptionOnly") => Ok(Self::SummaryOnly),
            Some("WithProvenance") => Ok(Self::WithProvenance),
            Some(other) => Err(NotaDecodeError::UnknownVariant {
                enum_name: "ObservationMode",
                variant: other.to_string(),
            }),
            None => Err(NotaDecodeError::ExpectedAtom {
                type_name: "ObservationMode",
            }),
        }
    }
}

pub type Mode = ObservationMode;

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Presence {
    Active,
    Absent,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub text: StatementText,
}

pub type Certainty = Magnitude;
pub type Privacy = Magnitude;

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub certainty: Certainty,
    pub privacy: Privacy,
}

impl Entry {
    pub fn open(
        topics: Topics,
        kind: Kind,
        description: Description,
        certainty: Certainty,
    ) -> Self {
        Self {
            topics,
            kind,
            description,
            certainty,
            privacy: Magnitude::Zero,
        }
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for Entry {
    fn to_nota(&self) -> String {
        Delimiter::Parenthesis.wrap([
            self.topics.to_nota(),
            self.kind.to_nota(),
            self.description.to_nota(),
            self.certainty.to_nota(),
            self.privacy.to_nota(),
        ])
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for Entry {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let fields = NotaBlock::new(block).expect_delimited(Delimiter::Parenthesis, "Entry")?;
        if !(4..=5).contains(&fields.len()) {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "Entry",
                expected: 5,
                found: fields.len(),
            });
        }
        let topics = Topics::from_nota_block(&fields[0])?;
        let kind = Kind::from_nota_block(&fields[1])?;
        let description = Description::from_nota_block(&fields[2])?;
        let certainty = Certainty::from_nota_block(&fields[3])?;
        let privacy = if fields.len() == 4 {
            Magnitude::Zero
        } else {
            Magnitude::from_nota_block(&fields[4])?
        };
        Ok(Self {
            topics,
            kind,
            description,
            certainty,
            privacy,
        })
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CertaintyChange {
    pub identifier: RecordIdentifier,
    pub certainty: Certainty,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordChange {
    pub record_identifier: RecordIdentifier,
    pub entry: Entry,
}

impl RecordChange {
    pub const fn identifier(&self) -> RecordIdentifier {
        self.record_identifier
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchKind {
    Any,
    Partial,
    Full,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct TopicSelection {
    pub match_kind: MatchKind,
    pub topics: Vec<Topic>,
}

impl TopicSelection {
    pub fn any() -> Self {
        Self {
            match_kind: MatchKind::Any,
            topics: Vec::new(),
        }
    }

    pub fn partial(topics: Vec<Topic>) -> Self {
        Self {
            match_kind: MatchKind::Partial,
            topics,
        }
    }

    pub fn full(topics: Vec<Topic>) -> Self {
        Self {
            match_kind: MatchKind::Full,
            topics,
        }
    }

    pub fn matches(&self, topics: &Topics) -> bool {
        match self.match_kind {
            MatchKind::Any => true,
            MatchKind::Partial => self.topics.iter().any(|topic| topics.contains(topic)),
            MatchKind::Full => {
                !self.topics.is_empty() && self.topics.iter().all(|topic| topics.contains(topic))
            }
        }
    }

    #[cfg(feature = "nota-text")]
    fn validate(&self) -> Result<(), NotaDecodeError> {
        match self.match_kind {
            MatchKind::Any if self.topics.is_empty() => Ok(()),
            MatchKind::Any => Err(NotaDecodeError::Parse(
                "TopicSelection: Any topic selection must not carry topics".to_owned(),
            )),
            MatchKind::Partial | MatchKind::Full if self.topics.is_empty() => Err(
                NotaDecodeError::Parse(
                    "TopicSelection: Partial and Full topic selections must carry at least one topic"
                        .to_owned(),
                ),
            ),
            MatchKind::Partial | MatchKind::Full => {
                let mut seen = std::collections::BTreeSet::<&str>::new();
                for topic in &self.topics {
                    if !seen.insert(topic.as_str()) {
                        return Err(NotaDecodeError::Parse(format!(
                            "TopicSelection: topic selection repeats topic {}",
                            topic.as_str()
                        )));
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for TopicSelection {
    fn to_nota(&self) -> String {
        self.validate()
            .expect("TopicSelection must be valid before NOTA encoding");
        Delimiter::Parenthesis.wrap([
            self.match_kind.to_nota(),
            Delimiter::SquareBracket
                .wrap(self.topics.iter().map(|topic| topic.as_str().to_owned())),
        ])
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for TopicSelection {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let fields =
            NotaBlock::new(block).expect_children(Delimiter::Parenthesis, "TopicSelection", 2)?;
        let match_kind = MatchKind::from_nota_block(&fields[0])?;
        let topics = Vec::<Topic>::from_nota_block(&fields[1])?;
        let selection = Self { match_kind, topics };
        selection.validate()?;
        Ok(selection)
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CertaintySelection {
    Any,
    Exact(Certainty),
    AtMost(Certainty),
    AtLeast(Certainty),
}

impl CertaintySelection {
    pub const fn removal_candidates() -> Self {
        Self::Exact(Magnitude::Zero)
    }

    pub fn matches(self, certainty: Certainty) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => certainty == expected,
            Self::AtMost(maximum) => certainty <= maximum,
            Self::AtLeast(minimum) => certainty >= minimum,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacySelection {
    Any,
    Exact(Privacy),
    AtMost(Privacy),
    AtLeast(Privacy),
}

impl PrivacySelection {
    pub const fn default_observation_privacy() -> Self {
        Self::Exact(Magnitude::Zero)
    }

    pub fn matches(self, privacy: Privacy) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(expected) => privacy == expected,
            Self::AtMost(maximum) => privacy <= maximum,
            Self::AtLeast(minimum) => privacy >= minimum,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedTimeSelection {
    Any,
    Between(RecordedTimeRange),
    Since(RecordedTime),
    Until(RecordedTime),
    Recent,
    Shallow,
    Deep,
    VeryDeep,
}

impl RecordedTimeSelection {
    pub const fn any() -> Self {
        Self::Any
    }

    pub const fn recent() -> Self {
        Self::Recent
    }

    pub fn matches(self, recorded_time: RecordedTime) -> bool {
        match self {
            Self::Any | Self::Recent | Self::Shallow | Self::Deep | Self::VeryDeep => true,
            Self::Between(range) => range.contains(recorded_time),
            Self::Since(first) => recorded_time >= first,
            Self::Until(last) => recorded_time <= last,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordQuery {
    pub topic_selection: TopicSelection,
    pub kind: Option<Kind>,
    pub certainty_selection: CertaintySelection,
    pub recorded_time_selection: RecordedTimeSelection,
    pub privacy_selection: PrivacySelection,
    pub mode: ObservationMode,
}

impl RecordQuery {
    pub fn removal_candidates(mode: ObservationMode) -> Self {
        Self {
            topic_selection: TopicSelection::any(),
            kind: None,
            certainty_selection: CertaintySelection::removal_candidates(),
            recorded_time_selection: RecordedTimeSelection::Any,
            privacy_selection: PrivacySelection::default_observation_privacy(),
            mode,
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PublicRecordQuery {
    pub topic_selection: TopicSelection,
    pub kind: Option<Kind>,
    pub certainty_selection: CertaintySelection,
    pub recorded_time_selection: RecordedTimeSelection,
    pub mode: ObservationMode,
}

impl PublicRecordQuery {
    pub fn new(
        topic_selection: TopicSelection,
        kind: Option<Kind>,
        certainty_selection: CertaintySelection,
        recorded_time_selection: RecordedTimeSelection,
        mode: ObservationMode,
    ) -> Self {
        Self {
            topic_selection,
            kind,
            certainty_selection,
            recorded_time_selection,
            mode,
        }
    }

    pub fn any(mode: ObservationMode) -> Self {
        Self::new(
            TopicSelection::any(),
            None,
            CertaintySelection::Any,
            RecordedTimeSelection::Any,
            mode,
        )
    }

    pub fn removal_candidates(mode: ObservationMode) -> Self {
        Self::new(
            TopicSelection::any(),
            None,
            CertaintySelection::removal_candidates(),
            RecordedTimeSelection::Any,
            mode,
        )
    }

    pub fn into_record_query(self) -> RecordQuery {
        RecordQuery {
            topic_selection: self.topic_selection,
            kind: self.kind,
            certainty_selection: self.certainty_selection,
            recorded_time_selection: self.recorded_time_selection,
            privacy_selection: PrivacySelection::default_observation_privacy(),
            mode: self.mode,
        }
    }
}

impl From<PublicRecordQuery> for RecordQuery {
    fn from(query: PublicRecordQuery) -> Self {
        query.into_record_query()
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for PublicRecordQuery {
    fn to_nota(&self) -> String {
        Delimiter::Parenthesis.wrap([
            self.topic_selection.to_nota(),
            self.kind.to_nota(),
            self.certainty_selection.to_nota(),
            self.recorded_time_selection.to_nota(),
            self.mode.to_nota(),
        ])
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for PublicRecordQuery {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let fields =
            NotaBlock::new(block).expect_delimited(Delimiter::Parenthesis, "PublicRecordQuery")?;
        if !(3..=6).contains(&fields.len()) {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "PublicRecordQuery",
                expected: 5,
                found: fields.len(),
            });
        }
        let topic_selection = TopicSelection::from_nota_block(&fields[0])?;
        let kind = Option::<Kind>::from_nota_block(&fields[1])?;
        let mut index = 2;

        let certainty_selection = if ObservationMode::block_is_mode(&fields[index]) {
            CertaintySelection::Any
        } else {
            let value = CertaintySelection::from_nota_block(&fields[index])?;
            index += 1;
            value
        };
        let recorded_time_selection = if ObservationMode::block_is_mode(&fields[index]) {
            RecordedTimeSelection::Any
        } else {
            let value = RecordedTimeSelection::from_nota_block(&fields[index])?;
            index += 1;
            value
        };
        if !ObservationMode::block_is_mode(&fields[index]) {
            let privacy_selection = PrivacySelection::from_nota_block(&fields[index])?;
            if privacy_selection != PrivacySelection::default_observation_privacy() {
                return Err(NotaDecodeError::Parse(
                    "PublicRecordQuery: public record queries cannot carry elevated privacy"
                        .to_owned(),
                ));
            }
            index += 1;
        }
        if index >= fields.len() {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "PublicRecordQuery",
                expected: 5,
                found: fields.len(),
            });
        }
        let mode = ObservationMode::from_nota_block(&fields[index])?;
        if index + 1 != fields.len() {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "PublicRecordQuery",
                expected: index + 1,
                found: fields.len(),
            });
        }
        Ok(Self {
            topic_selection,
            kind,
            certainty_selection,
            recorded_time_selection,
            mode,
        })
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PrivacyScopedRecordQuery {
    pub privacy_selection: PrivacySelection,
    pub public_record_query: PublicRecordQuery,
}

impl PrivacyScopedRecordQuery {
    pub fn new(privacy_selection: PrivacySelection, query: PublicRecordQuery) -> Self {
        Self {
            privacy_selection,
            public_record_query: query,
        }
    }

    pub fn at_most(privacy: Privacy, query: PublicRecordQuery) -> Self {
        Self::new(PrivacySelection::AtMost(privacy), query)
    }

    pub fn into_record_query(self) -> RecordQuery {
        let mut query = self.public_record_query.into_record_query();
        query.privacy_selection = self.privacy_selection;
        query
    }
}

#[cfg(feature = "nota-text")]
impl NotaEncode for RecordQuery {
    fn to_nota(&self) -> String {
        Delimiter::Parenthesis.wrap([
            self.topic_selection.to_nota(),
            self.kind.to_nota(),
            self.certainty_selection.to_nota(),
            self.recorded_time_selection.to_nota(),
            self.privacy_selection.to_nota(),
            self.mode.to_nota(),
        ])
    }
}

#[cfg(feature = "nota-text")]
impl NotaDecode for RecordQuery {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        let fields =
            NotaBlock::new(block).expect_delimited(Delimiter::Parenthesis, "RecordQuery")?;
        if !(3..=6).contains(&fields.len()) {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "RecordQuery",
                expected: 6,
                found: fields.len(),
            });
        }
        let topic_selection = TopicSelection::from_nota_block(&fields[0])?;
        let kind = Option::<Kind>::from_nota_block(&fields[1])?;
        let mut index = 2;
        let certainty_selection = if ObservationMode::block_is_mode(&fields[index]) {
            CertaintySelection::Any
        } else {
            let value = CertaintySelection::from_nota_block(&fields[index])?;
            index += 1;
            value
        };
        let recorded_time_selection = if ObservationMode::block_is_mode(&fields[index]) {
            RecordedTimeSelection::Any
        } else {
            let value = RecordedTimeSelection::from_nota_block(&fields[index])?;
            index += 1;
            value
        };
        let privacy_selection = if ObservationMode::block_is_mode(&fields[index]) {
            PrivacySelection::default_observation_privacy()
        } else {
            let value = PrivacySelection::from_nota_block(&fields[index])?;
            index += 1;
            value
        };
        if index >= fields.len() {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "RecordQuery",
                expected: 6,
                found: fields.len(),
            });
        }
        let mode = ObservationMode::from_nota_block(&fields[index])?;
        if index + 1 != fields.len() {
            return Err(NotaDecodeError::ExpectedRootCount {
                type_name: "RecordQuery",
                expected: index + 1,
                found: fields.len(),
            });
        }
        Ok(Self {
            topic_selection,
            kind,
            certainty_selection,
            recorded_time_selection,
            privacy_selection,
            mode,
        })
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordIdentifierSelection {
    Exact(RecordIdentifier),
}

impl RecordIdentifierSelection {
    pub fn contains(self, identifier: RecordIdentifier) -> bool {
        match self {
            Self::Exact(expected) => identifier == expected,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordIdentifierQuery {
    pub record_identifier_selection: RecordIdentifierSelection,
    pub mode: ObservationMode,
}

impl RecordIdentifierQuery {
    pub const fn new(
        record_identifier_selection: RecordIdentifierSelection,
        mode: ObservationMode,
    ) -> Self {
        Self {
            record_identifier_selection,
            mode,
        }
    }

    pub fn contains(self, identifier: RecordIdentifier) -> bool {
        self.record_identifier_selection.contains(identifier)
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivacyScopedRecordIdentifierQuery {
    pub privacy_selection: PrivacySelection,
    pub record_identifier_query: RecordIdentifierQuery,
}

impl PrivacyScopedRecordIdentifierQuery {
    pub const fn new(privacy_selection: PrivacySelection, query: RecordIdentifierQuery) -> Self {
        Self {
            privacy_selection,
            record_identifier_query: query,
        }
    }

    pub const fn at_most(privacy: Privacy, query: RecordIdentifierQuery) -> Self {
        Self::new(PrivacySelection::AtMost(privacy), query)
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordObservation {
    pub query: RecordQuery,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    StandardOutput,
    StandardError,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum ArchiveDatabaseTarget {
    Default,
    Path(ArchivePath),
}

impl ArchiveDatabaseTarget {
    pub fn path(path: impl Into<String>) -> Self {
        Self::Path(ArchivePath::new(path))
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum OutputTarget {
    ArchiveDatabase(ArchiveDatabaseTarget),
    Print(OutputStream),
}

impl OutputTarget {
    pub const fn default_archive_database() -> Self {
        Self::ArchiveDatabase(ArchiveDatabaseTarget::Default)
    }

    pub fn archive_database(path: impl Into<String>) -> Self {
        Self::ArchiveDatabase(ArchiveDatabaseTarget::path(path))
    }

    pub const fn print_standard_output() -> Self {
        Self::Print(OutputStream::StandardOutput)
    }

    pub const fn print_standard_error() -> Self {
        Self::Print(OutputStream::StandardError)
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RemovalCandidateCollection {
    pub record_query: RecordQuery,
    pub output_target: OutputTarget,
}

impl RemovalCandidateCollection {
    pub fn new(record_query: RecordQuery, output_target: OutputTarget) -> Self {
        Self {
            record_query,
            output_target,
        }
    }

    pub fn default_archive_database() -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::default_archive_database(),
        )
    }

    pub fn archive_database(path: impl Into<String>) -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::archive_database(path),
        )
    }

    pub fn print_standard_output() -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::print_standard_output(),
        )
    }

    pub fn print_standard_error() -> Self {
        Self::new(
            RecordQuery::removal_candidates(ObservationMode::SummaryOnly),
            OutputTarget::print_standard_error(),
        )
    }

    pub fn is_exact_zero_candidate_query(&self) -> bool {
        matches!(
            self.record_query.certainty_selection,
            CertaintySelection::Exact(Magnitude::Zero)
        ) && matches!(
            self.record_query.privacy_selection,
            PrivacySelection::Exact(Magnitude::Zero)
        )
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordSubscription {
    pub topic: Option<Topic>,
    pub mode: ObservationMode,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PrivacyScopedRecordSubscription {
    pub privacy_selection: PrivacySelection,
    pub record_subscription: RecordSubscription,
}

impl PrivacyScopedRecordSubscription {
    pub fn new(privacy_selection: PrivacySelection, subscription: RecordSubscription) -> Self {
        Self {
            privacy_selection,
            record_subscription: subscription,
        }
    }

    pub fn at_most(privacy: Privacy, subscription: RecordSubscription) -> Self {
        Self::new(PrivacySelection::AtMost(privacy), subscription)
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordSummary {
    pub identifier: RecordIdentifier,
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub certainty: Certainty,
    pub privacy: Privacy,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenance {
    pub summary: RecordSummary,
    pub date: Date,
    pub time: Time,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemovalCandidateSkipReason {
    ArchiveFailed,
    RecordChanged,
    RecordAlreadyRemoved,
    NoLongerCandidate,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SkippedRemovalCandidate {
    pub identifier: RecordIdentifier,
    pub reason: RemovalCandidateSkipReason,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RemovalCandidatesCollected {
    pub archived_records: Vec<RecordSummary>,
    pub removed_identifiers: Vec<RecordIdentifier>,
    pub skipped_candidates: Vec<SkippedRemovalCandidate>,
}

impl RemovalCandidatesCollected {
    pub fn new(
        archived_records: Vec<RecordSummary>,
        removed_identifiers: Vec<RecordIdentifier>,
        skipped_candidates: Vec<SkippedRemovalCandidate>,
    ) -> Self {
        Self {
            archived_records,
            removed_identifiers,
            skipped_candidates,
        }
    }

    pub fn archived_records(&self) -> &[RecordSummary] {
        &self.archived_records
    }

    pub fn removed_identifiers(&self) -> &[RecordIdentifier] {
        &self.removed_identifiers
    }

    pub fn skipped_candidates(&self) -> &[SkippedRemovalCandidate] {
        &self.skipped_candidates
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct TopicCount {
    pub topic: Topic,
    pub entries: u64,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PresenceView {
    pub presence: Presence,
    pub focus: Option<FocusArea>,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionIdentifier(String);

impl QuestionIdentifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionText(String);

impl QuestionText {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct QuestionSummary {
    pub identifier: QuestionIdentifier,
    pub question: QuestionText,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordAccepted(RecordIdentifier);

impl RecordAccepted {
    pub const fn new(identifier: RecordIdentifier) -> Self {
        Self(identifier)
    }

    pub const fn identifier(self) -> RecordIdentifier {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordRemoved(RecordIdentifier);

impl RecordRemoved {
    pub const fn new(identifier: RecordIdentifier) -> Self {
        Self(identifier)
    }

    pub const fn identifier(self) -> RecordIdentifier {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordMutationApplied(RecordIdentifier);

impl RecordMutationApplied {
    pub const fn new(identifier: RecordIdentifier) -> Self {
        Self(identifier)
    }

    pub const fn identifier(self) -> RecordIdentifier {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CertaintyChanged {
    pub identifier: RecordIdentifier,
    pub certainty: Certainty,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StateObserved(PresenceView);

impl StateObserved {
    pub fn new(state: PresenceView) -> Self {
        Self(state)
    }

    pub fn state(&self) -> &PresenceView {
        &self.0
    }

    pub fn into_state(self) -> PresenceView {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordsObserved(Vec<RecordSummary>);

impl RecordsObserved {
    pub fn new(records: Vec<RecordSummary>) -> Self {
        Self(records)
    }

    pub fn records(&self) -> &[RecordSummary] {
        &self.0
    }

    pub fn into_records(self) -> Vec<RecordSummary> {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordProvenancesObserved(Vec<RecordProvenance>);

impl RecordProvenancesObserved {
    pub fn new(records: Vec<RecordProvenance>) -> Self {
        Self(records)
    }

    pub fn records(&self) -> &[RecordProvenance] {
        &self.0
    }

    pub fn into_records(self) -> Vec<RecordProvenance> {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct TopicsObserved(Vec<TopicCount>);

impl TopicsObserved {
    pub fn new(topics: Vec<TopicCount>) -> Self {
        Self(topics)
    }

    pub fn topics(&self) -> &[TopicCount] {
        &self.0
    }

    pub fn into_topics(self) -> Vec<TopicCount> {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct QuestionsObserved(Vec<QuestionSummary>);

impl QuestionsObserved {
    pub fn new(questions: Vec<QuestionSummary>) -> Self {
        Self(questions)
    }

    pub fn questions(&self) -> &[QuestionSummary] {
        &self.0
    }

    pub fn into_questions(self) -> Vec<QuestionSummary> {
        self.0
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    State,
    Records(PublicRecordQuery),
    PrivateRecords(PrivacyScopedRecordQuery),
    RecordIdentifiers(RecordIdentifierQuery),
    PrivateRecordIdentifiers(PrivacyScopedRecordIdentifierQuery),
    Topics,
    Questions,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum Subscription {
    State,
    Records(RecordSubscription),
    PrivateRecords(PrivacyScopedRecordSubscription),
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionToken {
    State(StateSubscriptionToken),
    Records(RecordSubscriptionToken),
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionSnapshot {
    State(PresenceView),
    Records(Vec<RecordSummary>),
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionOpened {
    pub token: SubscriptionToken,
    pub snapshot: SubscriptionSnapshot,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionRetracted {
    pub token: SubscriptionToken,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnimplementedReason {
    NotBuiltYet,
    IntegrationNotLanded,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RequestUnimplemented {
    pub reason: UnimplementedReason,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StateChanged {
    pub state: PresenceView,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct RecordCaptured {
    pub record: RecordSummary,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct OperationReceived {
    pub operation: OperationKind,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectOutcome {
    StateChanged,
    RecordCaptured,
    RecordRemoved,
    RecordChanged,
    CertaintyChanged,
    RemovalCandidatesCollected,
    Observed,
    StreamOpened,
    StreamClosed,
    NoChange,
}

#[cfg_attr(
    feature = "nota-text",
    derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
)]
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct EffectEmitted {
    pub operation: OperationKind,
    pub outcome: EffectOutcome,
}

signal_channel! {
    channel Spirit {
        operation State(Statement),
        operation Record(Entry),
        operation Observe(Observation),
        operation Watch(Subscription) opens DomainStream,
        operation Unwatch(SubscriptionToken),
        operation Remove(RecordIdentifier),
        operation ChangeRecord(RecordChange),
        operation ChangeCertainty(CertaintyChange),
        operation CollectRemovalCandidates(RemovalCandidateCollection),
    }
    reply Reply {
        RecordAccepted(RecordAccepted),
        RecordRemoved(RecordRemoved),
        RecordMutationApplied(RecordMutationApplied),
        StateObserved(StateObserved),
        RecordsObserved(RecordsObserved),
        RecordProvenancesObserved(RecordProvenancesObserved),
        TopicsObserved(TopicsObserved),
        QuestionsObserved(QuestionsObserved),
        SubscriptionOpened(SubscriptionOpened),
        SubscriptionRetracted(SubscriptionRetracted),
        RequestUnimplemented(RequestUnimplemented),
        CertaintyChanged(CertaintyChanged),
        RemovalCandidatesCollected(RemovalCandidatesCollected),
    }
    event Event {
        StateChanged(StateChanged) belongs DomainStream,
        RecordCaptured(RecordCaptured) belongs DomainStream,
    }
    stream DomainStream {
        token SubscriptionToken;
        opened SubscriptionOpened;
        event StateChanged;
        event RecordCaptured;
        close Unwatch;
    }
    observable {
        filter default;
        operation_event OperationReceived;
        effect_event EffectEmitted;
    }
}
