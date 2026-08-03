use signal_spirit::{
    Description, Domain, DomainScope, DomainScopes, Domains, Entry, Importance, Input, Kind,
    Magnitude, ValidationError,
};

#[test]
fn intent_accepts_non_empty_domain_scopes() {
    let input = Input::intent(DomainScopes::new(vec![DomainScope::All]));

    assert_eq!(input.validate(), Ok(()));
}

#[test]
fn intent_rejects_empty_domain_scopes() {
    let input = Input::intent(DomainScopes::new(Vec::new()));

    assert_eq!(input.validate(), Err(ValidationError::EmptyQueryDomain));
}

#[test]
fn entry_rejects_empty_domains() {
    let entry = Entry {
        domains: Domains::new(Vec::new()),
        kind: Kind::Decision,
        description: Description::new("empty domains are still invalid"),
        importance: Importance::new(Magnitude::Minimum),
    };

    assert_eq!(entry.validate(), Err(ValidationError::EmptyDomain));
}

#[test]
fn entry_rejects_empty_description() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::All]),
        kind: Kind::Decision,
        description: Description::new("  "),
        importance: Importance::new(Magnitude::Minimum),
    };

    assert_eq!(entry.validate(), Err(ValidationError::EmptyDescription));
}

#[test]
fn four_field_entry_accepts_top_level_all_domain() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::All]),
        kind: Kind::Decision,
        description: Description::new("top-level all means every subject domain"),
        importance: Importance::new(Magnitude::Minimum),
    };

    assert_eq!(entry.validate(), Ok(()));
}
