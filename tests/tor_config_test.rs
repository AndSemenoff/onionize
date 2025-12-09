// tests/tor_config_test.rs
use arti_client::{TorClient, TorClientConfig};
use tor_rtmock::MockRuntime;

#[test]
fn test_client_config_builder() {
    // Test ensures we can create a client config with valid paths
    let runtime = MockRuntime::new();

    // Use a temporary directory to avoid cluttering the system
    let temp_dir = std::env::temp_dir().join("arti-proxy-test");
    let cache_dir = temp_dir.join("cache");
    let state_dir = temp_dir.join("state");

    let mut config_builder = TorClientConfig::builder();
    config_builder
        .storage()
        .cache_dir(arti_client::config::CfgPath::new(
            cache_dir.to_string_lossy().into(),
        ))
        .state_dir(arti_client::config::CfgPath::new(
            state_dir.to_string_lossy().into(),
        ));

    let config = config_builder.build();

    assert!(config.is_ok(), "Config should be built successfully");

    let config = config.unwrap();

    // Attempt to create a client in "unbootstrapped" mode
    // This checks config validity and runtime compatibility
    let client = TorClient::with_runtime(runtime)
        .config(config)
        .create_unbootstrapped();

    assert!(
        client.is_ok(),
        "Client should be created successfully with the given config"
    );
}
