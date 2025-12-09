// src/tor.rs
use anyhow::{Context, Result};
use arti_client::config::CfgPath;
use arti_client::{TorClient, TorClientConfig};
use directories::ProjectDirs;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use rust_i18n::t;
use tor_hscrypto::pk::HsClientDescEncKey;
use tor_hsservice::config::restricted_discovery::HsClientNickname; // Type for the nickname
//use tor_hsservice::config::restricted_discovery::RestrictedDiscoveryConfigBuilder; // Config builder
use tor_hsservice::{HsNickname, RunningOnionService, config::OnionServiceConfigBuilder}; // Public key type

use tor_rtcompat::Runtime;
use tracing::info;

pub async fn start_tor_client<R: Runtime>(
    runtime: R,
    _config: Option<TorClientConfig>,
) -> Result<TorClient<R>> {
    info!("{}", t!("tor.starting_tor_client"));

    let dirs = ProjectDirs::from("", "", "arti-onion-proxy").context(t!("tor.errors.dirs"))?;
    // Path -> String -> CfgPath
    let cache_dir = CfgPath::new(dirs.cache_dir().to_string_lossy().into());
    let data_dir = CfgPath::new(dirs.data_dir().to_string_lossy().into());

    let mut config_builder = TorClientConfig::builder();
    config_builder
        .storage()
        .cache_dir(cache_dir)
        .state_dir(data_dir);

    let config: TorClientConfig = config_builder.build().unwrap_or_default();

    // Create unbootstrapped Tor client
    let tor_client = TorClient::with_runtime(runtime)
        .config(config)
        .create_unbootstrapped()
        .with_context(|| t!("tor.errors.bootstrap"))?;

    // Set up progress bar for bootstrap
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}\n") // {pos}%
            .expect(&t!("tor.errors.progress_bar"))
            .progress_chars("#>-"),
    );

    pb.set_message(t!("tor.initializing"));

    // Clone for event listener
    let events_client = tor_client.clone();
    let pb_clone = pb.clone();

    // Spawn a task to listen for bootstrap events
    let bootstrap_task = tokio::spawn(async move {
        let mut events = events_client.bootstrap_events();

        while let Some(status) = events.next().await {
            let percent = (status.as_frac() * 100.0) as u64;
            pb_clone.set_position(percent);
            pb_clone.set_message(status.to_string());

            if status.ready_for_traffic() {
                break;
            }
        }
    });

    // Start bootstrap process
    // This will block until bootstrap is complete.
    // We rely on the event listener to update the progress bar.
    if let Err(e) = tor_client.bootstrap().await {
        pb.abandon_with_message(t!("tor.bootstrap_failed"));
        return Err(anyhow::anyhow!(t!("tor.errors.bootstrap")).context(e));
    }

    bootstrap_task.abort();

    // Force set to 100% as bootstrap completed successfully.
    // This is needed in case events lagged or the stream closed.
    pb.set_position(100);
    pb.finish_with_message(t!("tor.tor_client_started"));

    Ok(tor_client)
}

pub async fn launch_onion_service<R: Runtime>(
    client: &TorClient<R>,
    nickname_str: &str,
    client_auth_str: Option<String>,
) -> Result<(
    std::sync::Arc<RunningOnionService>,
    impl futures::Stream<Item = tor_hsservice::RendRequest>,
)> {
    use std::str::FromStr;

    let nickname = HsNickname::new(nickname_str.to_string())
        .with_context(|| t!("tor.errors.invalid_nickname"))?;

    let mut service_builder = OnionServiceConfigBuilder::default();
    service_builder.nickname(nickname);

    // --- Setup Restricted Discovery ---
    if let Some(auth_str) = client_auth_str {
        // parse the client authorization string into a HsClientDescEncKey
        let key: HsClientDescEncKey = auth_str
            .parse()
            .map_err(|_| anyhow::anyhow!(t!("cli.auth_error")))?;

        let random_bytes = rand::random::<[u8; 3]>();

        let client_nickname = &format!("client-{}", hex::encode(random_bytes));

        let client_nick = HsClientNickname::from_str(client_nickname)
            .map_err(|_| anyhow::anyhow!(t!("tor.error.invalid_nickname")))?;

        // Get access to the restricted_discovery we are take it in service_builder
        let rd = service_builder.restricted_discovery();

        // Enable Restricted Discovery
        rd.enabled(true);

        // Note: push takes ownership, so we pass client_nick and key directly
        rd.static_keys().access().push((client_nick, key));

        tracing::info!("{}", t!("tor.restricted_enabled", nick = client_nickname));
    }

    let service_config = service_builder
        .build()
        .with_context(|| t!("tor.errors.service_config"))?;

    let Some((service, requests)) = client
        .launch_onion_service(service_config)
        .with_context(|| t!("tor.errors.launch_service"))?
    else {
        return Err(anyhow::anyhow!(t!("tor.errors.launch_service")));
    };

    Ok((service, requests))
}
