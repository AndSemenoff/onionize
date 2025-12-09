use anyhow::Result;
use rust_i18n::t;
use x25519_dalek::{PublicKey, StaticSecret};

/// A container for generated Tor authorization keys.
///
/// Holds the keys in various formats required for server and client configuration.
pub struct TorKeys {
    /// The formatted string for the server-side configuration (public key).
    /// Format: `descriptor:x25519:<PUBLIC_KEY_BASE32>`
    pub server_string: String,

    /// The formatted string for the client-side configuration (private key).
    /// Format: `<PUBLIC_KEY_BASE32>:descriptor:x25519:<PRIVATE_KEY_BASE32>`
    pub client_string: String,

    /// The raw public key encoded in Base32 (RFC 4648 no padding).
    pub public_b32: String,
}

/// Generates a new ephemeral x25519 keypair for Client Authorization.
///
/// This creates a random static secret and derives the public key, returning
/// them in the format expected by Tor configuration files.
pub fn generate_keys() -> TorKeys {
    // Generate a new x25519 static secret
    let secret = StaticSecret::random();

    // Derive the public key from the secret
    let public = PublicKey::from(&secret);

    // (RFC 4648, no padding)
    let secret_b32 = base32::encode(
        base32::Alphabet::Rfc4648 { padding: false },
        secret.to_bytes().as_slice(),
    )
    .to_lowercase();

    let public_b32 = base32::encode(
        base32::Alphabet::Rfc4648 { padding: false },
        public.as_bytes(),
    )
    .to_lowercase();

    TorKeys {
        server_string: format!("descriptor:x25519:{}", public_b32),
        client_string: format!("{}:descriptor:x25519:{}", public_b32, secret_b32),
        public_b32,
    }
}

/// Generates a keypair and prints the formatted strings to stdout.
///
/// Used by the CLI command `--keygen`.
pub fn print_new_keypair() -> Result<()> {
    let keys = generate_keys();

    println!("{}", t!("keygen.beginning"));

    println!("{}", t!("keygen.public_text"));
    println!(
        "{}",
        t!(
            "keygen.public_format_text",
            server_string = keys.server_string
        )
    );

    println!("{}", t!("keygen.private_text"));
    println!(
        "{}",
        t!("keygen.secret_string", secret_str = keys.client_string)
    );
    println!(
        "{}",
        t!(
            "keygen.secret_key",
            secret_b32 = keys.client_string.split(':').next_back().unwrap()
        )
    );

    Ok(())
}
