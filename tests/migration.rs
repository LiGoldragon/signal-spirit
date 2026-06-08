#![cfg(feature = "nota-text")]

use nota_next::NotaSource;
use signal_spirit::{
    Entry, Kind, Magnitude, Operation,
    migration::{V010ToV011, V020ToV030, V030ToV040, v010, v020, v030},
};
use version_projection::VersionProjection;

#[test]
fn v010_certainty_projects_to_v011_magnitude() {
    assert_eq!(
        Magnitude::from(v010::Certainty::Maximum),
        Magnitude::Maximum
    );
    assert_eq!(Magnitude::from(v010::Certainty::Medium), Magnitude::Medium);
    assert_eq!(
        Magnitude::from(v010::Certainty::Minimum),
        Magnitude::Minimum
    );
}

#[test]
fn v010_record_entry_projects_to_current_entry_shape() {
    let source = v010::Entry {
        topic: v010::Topic::new("workspace"),
        kind: v010::Kind::Decision,
        summary: v010::Summary::new("description"),
        context: v010::Context::new("context"),
        certainty: v010::Certainty::Maximum,
        quote: v010::Quote::new("quote"),
    };

    let current = <V010ToV011 as VersionProjection<v010::Entry, Entry>>::project(source).unwrap();

    assert_eq!(current.topics.as_slice().len(), 1);
    assert_eq!(current.topics.as_slice()[0].as_str(), "workspace");
    assert_eq!(current.kind, Kind::Decision);
    assert_eq!(current.description.as_str(), "description");
    assert_eq!(current.certainty, Magnitude::Maximum);
    assert_eq!(current.privacy, Magnitude::Zero);
}

#[test]
fn v010_nota_record_projects_to_current_operation() {
    let source = NotaSource::new(
        "(Record (workspace Decision [description] [context dropped] Medium [quote dropped]))",
    );
    let source = source.parse::<v010::Operation>().unwrap();

    let current =
        <V010ToV011 as VersionProjection<v010::Operation, Operation>>::project(source).unwrap();

    let Operation::Record(entry) = current else {
        panic!("expected current Record operation");
    };
    assert_eq!(entry.certainty, Magnitude::Medium);
    assert_eq!(entry.privacy, Magnitude::Zero);
    assert_eq!(entry.description.as_str(), "description");
    assert_eq!(entry.topics.as_slice()[0].as_str(), "workspace");
}

#[test]
fn v020_record_entry_projects_to_multi_topic_current_entry_shape() {
    let source = v020::Entry {
        topic: v020::Topic::new("spirit"),
        kind: v020::Kind::Correction,
        description: v020::Description::new("single topic becomes topic vector"),
        certainty: Magnitude::High,
    };

    let current = <V020ToV030 as VersionProjection<v020::Entry, Entry>>::project(source).unwrap();

    assert_eq!(current.topics.as_slice().len(), 1);
    assert_eq!(current.topics.as_slice()[0].as_str(), "spirit");
    assert_eq!(current.kind, Kind::Correction);
    assert_eq!(
        current.description.as_str(),
        "single topic becomes topic vector"
    );
    assert_eq!(current.certainty, Magnitude::High);
    assert_eq!(current.privacy, Magnitude::Zero);
}

#[test]
fn v030_record_entry_projects_to_privacy_aware_current_entry_shape() {
    let source = v030::Entry {
        topics: v030::Topics::new(vec![
            v030::Topic::new("spirit"),
            v030::Topic::new("privacy"),
        ]),
        kind: v030::Kind::Constraint,
        description: v030::Description::new("existing intent stays public after migration"),
        certainty: Magnitude::Maximum,
    };

    let current = <V030ToV040 as VersionProjection<v030::Entry, Entry>>::project(source).unwrap();

    assert_eq!(current.topics.as_slice().len(), 2);
    assert_eq!(current.topics.as_slice()[0].as_str(), "spirit");
    assert_eq!(current.topics.as_slice()[1].as_str(), "privacy");
    assert_eq!(current.kind, Kind::Constraint);
    assert_eq!(
        current.description.as_str(),
        "existing intent stays public after migration"
    );
    assert_eq!(current.certainty, Magnitude::Maximum);
    assert_eq!(current.privacy, Magnitude::Zero);
}
