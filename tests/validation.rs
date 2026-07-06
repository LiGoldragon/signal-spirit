use signal_spirit::{
    Certainty, DataLeaf, Description, Domain, DomainScope, DomainScopes, Domains, Entry,
    Importance, Input, Kind, Magnitude, Privacy, Referent, Referents, Software, Technology,
    ValidationError,
};

#[test]
fn public_intent_accepts_non_empty_domain_scopes() {
    let input = Input::public_intent(DomainScopes::new(vec![DomainScope::All]));

    assert_eq!(input.validate(), Ok(()));
}

#[test]
fn public_intent_rejects_empty_domain_scopes() {
    let input = Input::public_intent(DomainScopes::new(Vec::new()));

    assert_eq!(input.validate(), Err(ValidationError::EmptyQueryDomain));
}

#[test]
fn entry_rejects_empty_domains() {
    let entry = Entry {
        domains: Domains::new(Vec::new()),
        kind: Kind::Decision,
        description: Description::new("empty domains are still invalid"),
        certainty: Certainty::new(Magnitude::High),
        importance: Importance::new(Magnitude::Minimum),
        privacy: Privacy::new(Magnitude::Zero),
        referents: Referents::new(vec![Referent::new("spirit")]),
    };

    assert_eq!(entry.validate(), Err(ValidationError::EmptyDomain));
}

#[test]
fn entry_accepts_top_level_all_domain() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::All]),
        kind: Kind::Decision,
        description: Description::new("top-level all means every subject domain"),
        certainty: Certainty::new(Magnitude::High),
        importance: Importance::new(Magnitude::Minimum),
        privacy: Privacy::new(Magnitude::Zero),
        referents: Referents::new(vec![Referent::new("spirit")]),
    };

    assert_eq!(entry.validate(), Ok(()));
}

#[test]
fn active_entry_rejects_empty_referents() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::Technology(Technology::Software(
            Software::Data(DataLeaf::SchemaEvolution),
        ))]),
        kind: Kind::Decision,
        description: Description::new("active entries need retrieval keys"),
        certainty: Certainty::new(Magnitude::High),
        importance: Importance::new(Magnitude::Minimum),
        privacy: Privacy::new(Magnitude::Zero),
        referents: Referents::new(Vec::new()),
    };

    assert_eq!(entry.validate(), Err(ValidationError::EmptyReferents));
}

#[test]
fn zero_certainty_entry_allows_empty_referents() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::Technology(Technology::Software(
            Software::Data(DataLeaf::SchemaEvolution),
        ))]),
        kind: Kind::Decision,
        description: Description::new("zero entries may remain legacy removal candidates"),
        certainty: Certainty::new(Magnitude::Zero),
        importance: Importance::new(Magnitude::Minimum),
        privacy: Privacy::new(Magnitude::Zero),
        referents: Referents::new(Vec::new()),
    };

    assert_eq!(entry.validate(), Ok(()));
}

#[test]
fn active_entry_accepts_non_empty_referents() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::Technology(Technology::Software(
            Software::Data(DataLeaf::SchemaEvolution),
        ))]),
        kind: Kind::Decision,
        description: Description::new("active entries carry retrieval keys"),
        certainty: Certainty::new(Magnitude::High),
        importance: Importance::new(Magnitude::Minimum),
        privacy: Privacy::new(Magnitude::Zero),
        referents: Referents::new(vec![Referent::new("spirit")]),
    };

    assert_eq!(entry.validate(), Ok(()));
}
