//! Adjacent-version projection witnesses for the Spirit contract.

use version_projection::{ProjectionError, VersionProjection};

use crate::{Description, Entry, Kind, Magnitude, Operation, Statement, Topic, Topics};

pub mod v010 {
    use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

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

        pub fn into_current(self) -> crate::Topic {
            crate::Topic::new(self.0)
        }
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Summary(String);

    impl Summary {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }

        pub fn into_description(self) -> crate::Description {
            crate::Description::new(self.0)
        }
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Context(String);

    impl Context {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Quote(String);

    impl Quote {
        pub fn new(value: impl Into<String>) -> Self {
            Self(value.into())
        }
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

    impl From<Kind> for crate::Kind {
        fn from(value: Kind) -> Self {
            match value {
                Kind::Decision => Self::Decision,
                Kind::Principle => Self::Principle,
                Kind::Correction => Self::Correction,
                Kind::Clarification => Self::Clarification,
                Kind::Constraint => Self::Constraint,
            }
        }
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Certainty {
        Maximum,
        Medium,
        Minimum,
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Entry {
        pub topic: Topic,
        pub kind: Kind,
        pub summary: Summary,
        pub context: Context,
        pub certainty: Certainty,
        pub quote: Quote,
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
    pub enum Operation {
        Record(Entry),
    }
}

pub mod v020 {
    use crate::Magnitude;
    use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

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

        pub fn into_current(self) -> crate::Topic {
            crate::Topic::new(self.0)
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

        pub fn into_current(self) -> crate::Description {
            crate::Description::new(self.0)
        }
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

    impl From<Kind> for crate::Kind {
        fn from(value: Kind) -> Self {
            match value {
                Kind::Decision => Self::Decision,
                Kind::Principle => Self::Principle,
                Kind::Correction => Self::Correction,
                Kind::Clarification => Self::Clarification,
                Kind::Constraint => Self::Constraint,
            }
        }
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Entry {
        pub topic: Topic,
        pub kind: Kind,
        pub description: Description,
        pub certainty: Magnitude,
    }

    #[cfg_attr(
        feature = "nota-text",
        derive(::nota_next::NotaEncode, ::nota_next::NotaDecode)
    )]
    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
    pub enum Operation {
        Record(Entry),
    }
}

pub mod v030 {
    use crate::Magnitude;
    use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

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

        pub fn into_current(self) -> crate::Topic {
            crate::Topic::new(self.0)
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

        pub fn into_current(self) -> crate::Topics {
            crate::Topics::new(
                self.0
                    .into_iter()
                    .map(Topic::into_current)
                    .collect::<Vec<_>>(),
            )
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

        pub fn into_current(self) -> crate::Description {
            crate::Description::new(self.0)
        }
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

    impl From<Kind> for crate::Kind {
        fn from(value: Kind) -> Self {
            match value {
                Kind::Decision => Self::Decision,
                Kind::Principle => Self::Principle,
                Kind::Correction => Self::Correction,
                Kind::Clarification => Self::Clarification,
                Kind::Constraint => Self::Constraint,
            }
        }
    }

    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Entry {
        pub topics: Topics,
        pub kind: Kind,
        pub description: Description,
        pub certainty: Magnitude,
    }

    #[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
    pub enum Operation {
        Record(Entry),
    }
}

pub struct V010ToV011;
pub struct V020ToV030;
pub struct V030ToV040;

impl From<v010::Certainty> for Magnitude {
    fn from(value: v010::Certainty) -> Self {
        match value {
            v010::Certainty::Maximum => Self::Maximum,
            v010::Certainty::Medium => Self::Medium,
            v010::Certainty::Minimum => Self::Minimum,
        }
    }
}

impl VersionProjection<v010::Entry, Entry> for V010ToV011 {
    type Error = ProjectionError;

    fn project(source: v010::Entry) -> Result<Entry, Self::Error> {
        Ok(Entry {
            topics: Topics::single(source.topic.into_current()),
            kind: source.kind.into(),
            description: source.summary.into_description(),
            certainty: source.certainty.into(),
            privacy: Magnitude::Zero,
        })
    }
}

impl VersionProjection<v010::Operation, Operation> for V010ToV011 {
    type Error = ProjectionError;

    fn project(source: v010::Operation) -> Result<Operation, Self::Error> {
        match source {
            v010::Operation::Record(entry) => Ok(Operation::Record(<Self as VersionProjection<
                v010::Entry,
                Entry,
            >>::project(entry)?)),
        }
    }
}

impl VersionProjection<v020::Entry, Entry> for V020ToV030 {
    type Error = ProjectionError;

    fn project(source: v020::Entry) -> Result<Entry, Self::Error> {
        Ok(Entry {
            topics: Topics::single(source.topic.into_current()),
            kind: source.kind.into(),
            description: source.description.into_current(),
            certainty: source.certainty,
            privacy: Magnitude::Zero,
        })
    }
}

impl VersionProjection<v020::Operation, Operation> for V020ToV030 {
    type Error = ProjectionError;

    fn project(source: v020::Operation) -> Result<Operation, Self::Error> {
        match source {
            v020::Operation::Record(entry) => Ok(Operation::Record(<Self as VersionProjection<
                v020::Entry,
                Entry,
            >>::project(entry)?)),
        }
    }
}

impl VersionProjection<v030::Entry, Entry> for V030ToV040 {
    type Error = ProjectionError;

    fn project(source: v030::Entry) -> Result<Entry, Self::Error> {
        Ok(Entry {
            topics: source.topics.into_current(),
            kind: source.kind.into(),
            description: source.description.into_current(),
            certainty: source.certainty,
            privacy: Magnitude::Zero,
        })
    }
}

impl VersionProjection<v030::Operation, Operation> for V030ToV040 {
    type Error = ProjectionError;

    fn project(source: v030::Operation) -> Result<Operation, Self::Error> {
        match source {
            v030::Operation::Record(entry) => Ok(Operation::Record(<Self as VersionProjection<
                v030::Entry,
                Entry,
            >>::project(entry)?)),
        }
    }
}

impl VersionProjection<Statement, Statement> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Statement) -> Result<Statement, Self::Error> {
        Ok(source)
    }
}

impl VersionProjection<Topic, Topic> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Topic) -> Result<Topic, Self::Error> {
        Ok(source)
    }
}

impl VersionProjection<Topic, Topics> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Topic) -> Result<Topics, Self::Error> {
        Ok(Topics::single(source))
    }
}

impl VersionProjection<Kind, Kind> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: Kind) -> Result<Kind, Self::Error> {
        Ok(source)
    }
}

impl VersionProjection<v010::Summary, Description> for V010ToV011 {
    type Error = std::convert::Infallible;

    fn project(source: v010::Summary) -> Result<Description, Self::Error> {
        Ok(source.into_description())
    }
}
