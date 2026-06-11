use signal_spirit::{
    ConfigurationPath, SpiritDaemonConfiguration, SpiritGuardianAgentConfiguration,
    SpiritGuardianMaximumOutputTokens, SpiritGuardianModelName, SpiritGuardianProviderName,
    SpiritGuardianTimeoutMilliseconds,
};

#[test]
fn daemon_configuration_archives_guardian_agent_configuration() {
    let configuration = SpiritDaemonConfiguration::new(
        ConfigurationPath::new("/run/user/1000/spirit.sock"),
        ConfigurationPath::new("/home/li/.local/state/spirit/spirit.sema"),
    )
    .with_guardian_agent_configuration(SpiritGuardianAgentConfiguration::new(
        ConfigurationPath::new("/run/user/1000/agent.sock"),
        Some(SpiritGuardianProviderName::new("criomos-local")),
        Some(SpiritGuardianModelName::new("gemma-4-26b-a4b")),
        SpiritGuardianTimeoutMilliseconds::new(120_000),
        Some(SpiritGuardianMaximumOutputTokens::new(512)),
    ));

    let bytes = configuration.to_rkyv_bytes().expect("encode config");
    let recovered = SpiritDaemonConfiguration::from_rkyv_bytes(&bytes).expect("decode config");
    let guardian = recovered
        .guardian_agent_configuration()
        .expect("guardian config round-trips");

    assert_eq!(guardian.agent_socket_path(), "/run/user/1000/agent.sock");
    assert_eq!(guardian.provider_name(), Some("criomos-local"));
    assert_eq!(guardian.model_name(), Some("gemma-4-26b-a4b"));
    assert_eq!(guardian.timeout_milliseconds(), 120_000);
    assert_eq!(guardian.maximum_output_tokens(), Some(512));
}

#[test]
fn daemon_configuration_allows_absent_guardian_output_budget() {
    let configuration = SpiritDaemonConfiguration::new(
        ConfigurationPath::new("/run/user/1000/spirit.sock"),
        ConfigurationPath::new("/home/li/.local/state/spirit/spirit.sema"),
    )
    .with_guardian_agent_configuration(SpiritGuardianAgentConfiguration::new(
        ConfigurationPath::new("/run/user/1000/agent.sock"),
        Some(SpiritGuardianProviderName::new("deepseek")),
        Some(SpiritGuardianModelName::new("deepseek-v4-flash")),
        SpiritGuardianTimeoutMilliseconds::new(120_000),
        None,
    ));

    let bytes = configuration.to_rkyv_bytes().expect("encode config");
    let recovered = SpiritDaemonConfiguration::from_rkyv_bytes(&bytes).expect("decode config");
    let guardian = recovered
        .guardian_agent_configuration()
        .expect("guardian config round-trips");

    assert_eq!(guardian.agent_socket_path(), "/run/user/1000/agent.sock");
    assert_eq!(guardian.provider_name(), Some("deepseek"));
    assert_eq!(guardian.model_name(), Some("deepseek-v4-flash"));
    assert_eq!(guardian.timeout_milliseconds(), 120_000);
    assert_eq!(guardian.maximum_output_tokens(), None);
}
