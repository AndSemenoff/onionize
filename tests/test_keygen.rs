// tests/test_keygen.rs
use create::keygen;

#[test]
fn test_keygen_format_structure() {
    // Generate a new keypair
    let keys = keygen::generate_keys();

    // Test the format of the generated strings
    // Format: descriptor:x25519:<pubkey_base32>
    assert!(
        keys.server_string.starts_with("descriptor:x25519:"),
        "Server string format mismatch"
    );

    // Format:  <pubkey_base32>:descriptor:x25519:<privkey_base32>
    assert!(
        keys.client_string.contains(":descriptor:x25519:"),
        "Client string format mismatch"
    );

    // Check that the public key in base32 is correctly placed
    assert!(
        keys.server_string.contains(&keys.public_b32),
        "Server string does not contain the correct public key"
    );
    assert!(
        keys.client_string.starts_with(&keys.public_b32),
        "Client string does not start with the public key"
    );
}

#[test]
fn test_keygen_length() {
    let keys = keygen::generate_keys();

    // x25519 key is 32 bytes.
    // In Base32 (RFC4648 no padding) 32 bytes encode to 52 characters.
    // Calculation: (32 * 8) / 5 = 51.2 -> rounded to 52 characters.
    assert_eq!(
        keys.public_b32.len(),
        52,
        "Public key base32 length should be 52 characters"
    );

    // Check private key length in client string
    // Format: PUB_KEY (52) + ":descriptor:x25519:" (20) + PRIV_KEY (52)
    // Total client string length should be around 124 characters
    let parts: Vec<&str> = keys.client_string.split(":descriptor:x25519:").collect();
    assert_eq!(
        parts.len(),
        2,
        "Client string should have exactly two parts split by separator"
    );

    let priv_key_b32 = parts[1];
    assert_eq!(
        priv_key_b32.len(),
        52,
        "Private key base32 length should be 52 characters"
    );
}

#[test]
fn test_keygen_randomness() {
    // Generate two keypairs and ensure they are different
    let keys1 = keygen::generate_keys();
    let keys2 = keygen::generate_keys();

    assert_ne!(keys1.public_b32, keys2.public_b32, "Keys should be random");
    assert_ne!(keys1.client_string, keys2.client_string);
}
