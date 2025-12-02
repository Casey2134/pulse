# Contributing to Pulse

Thank you for your interest in contributing to Pulse! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/pulse.git
   cd pulse
   ```
3. Create a branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Before You Start

**Please open an issue first** before working on any significant changes. This helps us:
- Discuss the approach before you invest time
- Avoid duplicate work
- Ensure the change aligns with the project's direction

For small fixes (typos, documentation improvements), you can submit a PR directly.

## Development Setup

### Requirements

- Rust 1.75+ (uses 2024 edition)
- A Proxmox VE server for testing (or another supported provider)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run
cargo run -- --config config.toml
```

### Code Style

We use standard Rust tooling:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

Please ensure your code passes both before submitting a PR.

## Testing

**New features must include tests.** Run tests with:

```bash
cargo test
```

## Areas Where We Need Help

### New Providers

We'd love to support more infrastructure platforms! Some ideas:
- Docker / Docker Swarm
- Kubernetes
- Portainer
- Unraid
- TrueNAS

To add a new provider:
1. Create a new file in `src/providers/`
2. Implement the `Provider` trait from `src/providers/base.rs`
3. Add configuration struct to `src/config.rs`
4. Register the provider in `src/main.rs`

### UI Improvements

Ideas for UI enhancements:
- Graphs/sparklines for CPU/memory history
- Network I/O statistics
- Disk usage display
- Color themes
- Mouse support
- Container actions (start/stop/restart)

## Pull Request Process

1. Ensure your code compiles without warnings
2. Run `cargo fmt` and `cargo clippy`
3. Add tests for new functionality
4. Update documentation if needed
5. Submit the PR with a clear description of the changes

## Code of Conduct

Be respectful and constructive. We're all here to build something useful together.

## Questions?

Open an issue with your question and we'll do our best to help!

<!-- Discord coming soon -->
