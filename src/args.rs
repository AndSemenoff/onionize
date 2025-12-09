// src/args.rs
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "arti-onion-proxy")]
#[command(version, about = "Expose local ports via Tor Onion Services", long_about = None)]
pub struct Args {
    /// Local port to proxy
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Host or IP address
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,

    /// Nickname for the Onion Service
    #[arg(short, long, default_value = "my-ephemeral-service")]
    pub nickname: String,

    /// Enable verbose logging
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Display QR code for the Onion address
    #[arg(long, default_value_t = false)]
    pub qr: bool,

    /// Generate x25519 keypair for Client Authorization
    #[arg(long, default_value_t = false)]
    pub keygen: bool,

    /// Add authorized client (format: descriptor:x25519:<pubkey>)
    /// Enables restricted access (Client Auth).
    #[arg(long)]
    pub auth: Option<String>,

    /// Auto-generate keys and enable restricted access
    /// (Generates ephemeral keys for this session)
    #[arg(long, default_value_t = false)]
    pub restricted: bool,
}

impl Args {
    /// Returns normalized host (converts "localhost" to "127.0.0.1")
    pub fn get_normalized_host(&self) -> String {
        if self.host.eq_ignore_ascii_case("localhost") {
            "127.0.0.1".to_string()
        } else {
            self.host.clone()
        }
    }

    /// Returns effective nickname, generating a random one if default is used
    pub fn get_effective_nickname(&self) -> String {
        if self.nickname == "my-ephemeral-service" {
            // Generate a random nickname
            let random_bytes = rand::random::<[u8; 3]>();
            format!("proxy-{}", hex::encode(random_bytes))
        } else {
            self.nickname.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_normalization() {
        let args = Args::parse_from(["bin", "-H", "localhost"]);
        assert_eq!(args.get_normalized_host(), "127.0.0.1");

        let args_ip = Args::parse_from(["bin", "-H", "192.168.1.1"]);
        assert_eq!(args_ip.get_normalized_host(), "192.168.1.1");
    }

    #[test]
    fn test_nickname_generation() {
        let args = Args::parse_from(["bin"]); // Using default
        let nick = args.get_effective_nickname();
        assert!(nick.starts_with("proxy-"));
        assert_ne!(nick, "my-ephemeral-service");

        let args_custom = Args::parse_from(["bin", "-n", "custom-name"]);
        assert_eq!(args_custom.get_effective_nickname(), "custom-name");
    }

    #[test]
    fn test_custom_nickname_is_static() {
        let args = Args::parse_from(["bin", "-n", "static-name"]);
        assert_eq!(args.get_effective_nickname(), "static-name");
        assert_eq!(args.get_effective_nickname(), "static-name");
    }

    #[test]
    fn test_nickname_generation_randomness() {
        // Generate two nicknames with default settings and ensure they differ
        let args1 = Args::parse_from(["bin"]);
        let args2 = Args::parse_from(["bin"]);

        let nick1 = args1.get_effective_nickname();
        let nick2 = args2.get_effective_nickname();

        assert!(nick1.starts_with("proxy-"));
        assert!(nick2.starts_with("proxy-"));

        // They should not be the same
        assert_ne!(nick1, nick2, "Generated nicknames should be different");
    }
}
