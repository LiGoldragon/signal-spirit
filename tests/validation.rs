use signal_spirit::{
    DataLeaf, Description, Domain, Domains, Entry, Importance, Kind, Magnitude, Privacy, Software,
    Technology,
};

#[test]
fn active_entry_accepts_without_certainty_or_referents() {
    let entry = Entry {
        domains: Domains::new(vec![Domain::Technology(Technology::Software(
            Software::Data(DataLeaf::SchemaEvolution),
        ))]),
        kind: Kind::Decision,
        description: Description::new("active entries are accepted records"),
        importance: Importance::new(Magnitude::Minimum),
        privacy: Privacy::new(Magnitude::Zero),
    };

    assert_eq!(entry.validate(), Ok(()));
}
