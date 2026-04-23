# Contributing to Nexus Protocol

Thank you for your interest in contributing!

## Quick Start

```bash
git clone git@github.com:KaioH3/nexus.git
cd nexus

cargo build
cargo test
```

## Project Structure

```
nexus/
├── crates/
│   ├── nexus-protocol-core/  # Core types, messages, errors
│   ├── nexus-sandbox/        # WASM sandbox runtime
│   ├── nexus-ollama/        # Ollama client integration
│   └── nexus-sdk/           # Multi-language SDKs
└── tests/e2e/              # End-to-end tests
```

## Code Standards

- Run `cargo test` before submitting PRs
- Follow Rust idioms and style
- Add tests for new features
- Document public APIs

## Security

Never commit API keys or secrets. Use `.env` files (gitignored).

## License

Apache 2.0 - all contributions are Apache 2.0 licensed.