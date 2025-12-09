use anyhow::Result;
use rust_i18n::t;
use x25519_dalek::{PublicKey, StaticSecret};

/// Struct to hold generated Tor keys
pub struct TorKeys {
    pub server_string: String, // String for server-side configuration
    pub client_string: String, // String for client-side configuration
    pub public_b32: String,    // Public key in Base32 format
}

/// Generates a new x25519 keypair and returns the formatted strings
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

/// Prints the generated keypair to the console
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
