// tests/cli_test.rs
use clap::Parser;
use create::args::Args;

#[test]
fn test_default_args() {
    let args = Args::parse_from(["binary_name"]);

    // Check default values
    assert_eq!(args.port, 3000);
    assert_eq!(args.host, "127.0.0.1"); // Default host
}

#[test]
fn test_custom_host_and_port() {
    let args = Args::parse_from(["binary_name", "--port", "8080", "--host", "192.168.0.105"]);

    assert_eq!(args.port, 8080);
    assert_eq!(args.host, "192.168.0.105");
}

#[test]
fn test_localhost_parsing() {
    // Verify that CLI accepts "localhost" string correctly.
    // Conversion to IP happens in main.rs
    let args = Args::parse_from(["binary_name", "-H", "localhost"]);

    assert_eq!(args.host, "localhost");
}

#[test]
fn test_short_flags_combination() {
    let args = Args::parse_from([
        "binary_name",
        "-p",
        "5000",
        "-H",
        "0.0.0.0",
        "-n",
        "short-name",
        "-v",
    ]);

    assert_eq!(args.port, 5000);
    assert_eq!(args.host, "0.0.0.0");
    assert_eq!(args.nickname, "short-name");
    assert!(args.verbose);
}
