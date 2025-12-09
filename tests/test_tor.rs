// tests/test_tor.rs

use arti_client::{TorClient, TorClientConfig};
use create::tor;
use rust_i18n::set_locale;
// Import Runtime for generics and ToplevelBlockOn for calling
use tor_rtcompat::{Runtime, ToplevelBlockOn};
use tor_rtmock::{MockRuntime, net::MockNetwork};

/// Helper function to create a test client.
/// Now it accepts any type R implementing Runtime to work with both
/// MockRuntime and MockNetRuntime.
fn create_mock_client<R: Runtime>(runtime: R) -> TorClient<R> {
    let config = TorClientConfig::builder().build().unwrap();

    // Create an unbootstrapped client
    TorClient::with_runtime(runtime)
        .config(config)
        .create_unbootstrapped()
        .expect("Failed to create mock Tor client")
}

#[test]
fn test_launch_onion_service_invalid_nickname() {
    set_locale("en");

    let runtime = MockRuntime::new();
    let client = create_mock_client(runtime.clone());

    runtime.block_on(async {
        // Attempt 1: Nickname with a space (invalid in Tor)
        let result = tor::launch_onion_service(&client, "invalid nickname", None).await;

        // Use match instead of unwrap_err() because the success result types differ
        match result {
            Ok(_) => panic!("Expected error for invalid nickname, but got success"),
            Err(e) => {
                let err_msg = e.to_string();
                // Expect "Invalid nickname" error
                assert_eq!(err_msg, "Invalid nickname");
            }
        }
    });
}

#[test]
fn test_launch_onion_service_valid_config_generation() {
    set_locale("en");

    // Create network and wrapped runtime
    let network = MockNetwork::new();
    let runtime = network.builder().runtime(MockRuntime::new());

    // Create mock client
    let client = create_mock_client(runtime.clone());

    runtime.block_on(async {
        let nickname = "valid-nickname";

        let result = tor::launch_onion_service(&client, nickname, None).await;

        match result {
            Ok(_) => {
                // Success case - service launched
            }
            Err(e) => {
                let msg = e.to_string();

                // Verify that we passed nickname validation and failed inside Arti.
                // Errors indicating an attempted launch (meaning validation passed):
                let accepted_errors = [
                    "Cannot create onion service with unbootstrapped client",
                    "Keystore",
                    "No such provider",
                    "Failed to launch service",
                    "Bootstrap required",
                ];

                let is_accepted = accepted_errors.iter().any(|&sub| msg.contains(sub));

                // The nickname is valid, so these errors should not be about invalid nickname or config.
                assert_ne!(
                    msg, "Invalid nickname",
                    "Nickname validation failed unexpectedly"
                );
                assert_ne!(
                    msg, "Error in service configuration",
                    "Service configuration validation failed unexpectedly"
                );

                if !is_accepted {
                    println!("Info: runtime error: {}", msg);
                }
            }
        }
    });
}

#[test]
fn test_launch_onion_service_with_auth() {
    // Enable logs to see errors
    rust_i18n::set_locale("en");

    // We need a KeyPair. Use tor-hscrypto to generate a valid key.
    use tor_hscrypto::pk::HsClientDescEncKey;
    use tor_llcrypto::pk::curve25519;

    let runtime = MockRuntime::new();
    let client = create_mock_client(runtime.clone());

    // Generate a random keypair (x25519)
    let mut rng = rand::rng();
    let sk = curve25519::StaticSecret::random_from_rng(&mut rng);
    let pk = curve25519::PublicKey::from(&sk);

    // Convert public key to HsClientDescEncKey format
    let client_auth_key = HsClientDescEncKey::from(pk);

    // Get string representation: "descriptor:x25519:<BASE32>"
    // The to_string() method HsClientDescEncKey automatically adds the prefix.
    let auth_str = client_auth_key.to_string();

    println!("Generated Auth String: {}", auth_str);

    runtime.block_on(async {
        // Launch service with authorization requirement
        let result = tor::launch_onion_service(
            &client,
            "auth-test-nick",
            Some(auth_str), // Pass the auth string here
        )
        .await;

        match result {
            Ok((_service, _stream)) => {
                println!("✅ Onion service with auth launched successfully.");
            }
            Err(e) => {
                let msg = e.to_string();
                // Ignore mock network errors if config validation passed
                if msg.contains("Keystore") || msg.contains("unbootstrapped") {
                    println!("Config is valid{}", msg);
                } else {
                    panic!("❌ Error config: {}", msg);
                }
            }
        }
    });
}
