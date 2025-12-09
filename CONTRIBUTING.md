# Contributing to Arti Onion Proxy

First off, thank you for considering contributing to **Arti Onion Proxy**! It's people like you that make the open-source community such an amazing place to learn, inspire, and create.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```
    git clone https://github.com/your-username/arti-onion-proxy.git
    cd arti-onion-proxy
    ```
3.  **Create a new branch** for your feature or bugfix:
    ```bash
    git checkout -b feature/amazing-feature
    # or
    git checkout -b fix/annoying-bug
    ```

## Development Workflow

### Prerequisites
You need to have **Rust** and **Cargo** installed. We recommend using the latest stable version.

### Running Tests
This project uses the standard Rust testing framework. We use `tor-rtmock` to simulate network interactions, so you generally do not need a running Tor instance for unit tests.

To run the full test suite (CLI tests, Mock Proxy logic, and i18n):
```
cargo test
```
If you want to run a specific test file (e.g., only the mock proxy logic):
```
cargo test --test mock_proxy_test
```
### Code Style and Linting

We enforce code style and safety using rustfmt and clippy. Our CI pipeline will fail if these checks are not passed.
- Format your code:
    ```
    cargo fmt
    ```
- Run the linter: (Note: We treat warnings as errors in CI)
    ```
    cargo clippy --all-targets --all-features -- -D warnings
    ```

### Internationalization (i18n)

This project supports English (`en`) and Russian (`ru`) using `rust-i18n`. If you are adding new user-facing strings:

- Add the keys and translations to `locales/app.yml`.
- Use the `t!` macro in code, e.g., `t!("main.new_message")`.
- Ensure the structure remains valid YAML.

## Submitting a Pull Request (PR)

1. Push your branch to your fork:
  ```
  git push origin feature/amazing-feature
  ```
2. Open a Pull Request on the original repository.
3. Fill out the description:
   - Clearly describe what you changed.
   - Explain why this change is necessary.
   - Link to any relevant issues (e.g., "Closes #123").
4. Verify CI Checks: Wait for the GitHub Actions to pass (Clippy, Build, Tests).

### License

By contributing, you agree that your contributions will be licensed under the dual MIT/Apache-2.0 license, as defined in the project's root directory.