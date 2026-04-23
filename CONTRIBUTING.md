# Contributing to Nexus Protocol

Thank you for your interest in contributing!

## Quick Start

```bash
# Clone the repo
git clone https://github.com/KaioH3/nexus
cd nexus

# Build
cargo build

# Run tests
cargo test

# Run example
cargo run --example ollama_test -p nexus-protocol-core
```

## Project Structure

```
nexus/
├── crates/
│   ├── nexus-protocol-core/  # Core types, messages, errors
│   ├── nexus-sandbox/        # WASM sandbox runtime
│   └── nexus-ollama/         # Ollama client integration
├── tests/
│   └── e2e/                  # End-to-end security tests
└── README.md
```

## Code Standards

- All code must compile with `cargo build`
- All tests must pass with `cargo test`
- Follow Rust idioms and style
- Add tests for new features
- Document public APIs

## Security

- Never commit API keys or secrets
- Use `.env` files (gitignored)
- Report security issues to: [security email]

## Pull Requests

1. Fork the repo
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit PR

## Questions?

Open an issue on GitHub.