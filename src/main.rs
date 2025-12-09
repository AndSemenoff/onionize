// src/main.rs
mod proxy;
mod tor;
use anyhow::Context;
use anyhow::Result;
use clap::{Arg, ArgAction, CommandFactory, FromArgMatches};
use onionize::args::Args;
use onionize::keygen;
use qrcode::QrCode;
use qrcode::render::unicode;
use safelog::DisplayRedacted;
use tokio::signal;
use tor_rtcompat::PreferredRuntime;
use tracing::{debug, error, info};

use rust_i18n::t;
rust_i18n::i18n!("./locales");

#[tokio::main]
async fn main() -> Result<()> {
    onionize::utils::setup_locale();

    let mut command: clap::Command = Args::command();

    command = command.about(t!("cli.about"));

    // Customize help template
    // Important: {{ double braces }} are needed to escape clap variables,
    // while { single braces } are for our translations via format!
    let help_template = format!(
        "{{before-help}}{{name}} {{version}}\n{{about-with-newline}}\n{usage_title}: {{usage}}\n\n{options_title}:\n{{options}}\n\n{{after-help}}",
        usage_title = t!("cli.usage"),
        options_title = t!("cli.options")
    );

    command = command
        //.mut_arg("version", |arg| arg.help(t!("cli.version")))
        .help_template(help_template)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .long("help")
                .short('h')
                .action(ArgAction::Help)
                .help(t!("cli.help_info"))
                .global(true),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .action(ArgAction::Version)
                .help(t!("cli.version"))
                .global(true),
        )
        //.mut_arg("help", |arg| arg.help(t!("cli.help_info")))
        .mut_arg("port", |arg| arg.help(t!("cli.port_help")))
        .mut_arg("qr", |arg| arg.help(t!("cli.qr_help")))
        .mut_arg("auth", |arg| arg.help(t!("cli.auth_help")))
        .mut_arg("verbose", |arg| arg.help(t!("cli.verbose_help")))
        .mut_arg("host", |arg| arg.help(t!("cli.host_help")))
        .mut_arg("restricted", |arg| arg.help(t!("cli.restricted_help")))
        .mut_arg("nickname", |arg| arg.help(t!("cli.nickname_help")));

    let mut matches: clap::ArgMatches = command.get_matches();

    let args = Args::from_arg_matches_mut(&mut matches)?;

    // Set up logging
    let filter = if args.verbose {
        "debug"
    } else {
        "warn,arti_onion_proxy=info"
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    debug!("{:?}", rust_i18n::available_locales!());

    info!("{}", t!("main.starting"));

    if args.keygen {
        keygen::print_new_keypair()?;
        return Ok(());
    }

    let (auth_config, generated_client_key) = if args.restricted {
        info!("🔐 Generating ephemeral keys for restricted mode...");
        let keys = keygen::generate_keys();
        // Returns (Server_String, Client_String)
        (Some(keys.server_string), Some(keys.client_string))
    } else {
        // Use what the user provided in --auth (or None)
        (args.auth.clone(), None)
    };

    let host = args.get_normalized_host();
    let nickname = args.get_effective_nickname();
    let target_address = format!("{}:{}", host, args.port);

    info!("{}", t!("main.target_address", addr = target_address));
    if args.host != host {
        info!("{}", t!("main.localhost_conversion"));
    }
    info!("{}", t!("main.nickname_server", nickname = nickname));

    // Initialize the preferred runtime
    let runtime = PreferredRuntime::current()?;

    // Run the Tor client
    let tor_client = tor::start_tor_client(runtime.clone(), None).await?;

    // Launch the Onion Service
    let (service, requests) =
        tor::launch_onion_service(&tor_client, &nickname, auth_config).await?;

    let o_addr = service
        .onion_address()
        .ok_or(anyhow::anyhow!(t!("main.notgen")))?;

    if args.qr {
        let code = QrCode::new(format!("http://{}", o_addr.display_unredacted()))
            .context(t!("main.long"))?;
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();

        println!("{}", t!("main.qrcode", image = image));
    }

    info!(
        "{}",
        t!("main.o_created", o_addr = o_addr.display_unredacted())
    );

    if let Some(client_key) = generated_client_key {
        info!("{}", t!("main.restricted_info"));
        info!("{}", t!("main.restricted_client", client_key = client_key));
    }

    info!("{}", t!("main.redirecting_to", addr = target_address));

    // Run the proxy loop and handle Ctrl+C
    tokio::select! {
        _ = proxy::run_proxy_loop(runtime, requests, &target_address) => {
            error!("{}", t!("main.errors.loop_crashed"));
        }
        _ = signal::ctrl_c() => {
            info!("{}", t!("main.quit"));
            // Graceful shutdown can be handled here if needed
        }
    }

    Ok(())
}
