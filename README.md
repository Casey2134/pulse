# Pulse

A real-time TUI (Terminal User Interface) monitor for your homelab infrastructure. Currently supports Proxmox VE.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)

## Features

- Real-time monitoring of Proxmox nodes and containers (VMs & LXC)
- CPU and memory usage with visual gauges
- Uptime tracking for nodes and containers
- Search/filter functionality
- Sortable by name, status, CPU, or memory
- Auto-refresh every 5 seconds
- Keyboard-driven interface

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/pulse.git
cd pulse

# Build the project
cargo build --release

# The binary will be at ./target/release/pulse
```

### Requirements

- Rust 1.75+ (uses 2024 edition)
- A Proxmox VE server with API access

## Configuration

1. Copy the example config:
   ```bash
   cp config.example.toml config.toml
   ```

2. Edit `config.toml` with your Proxmox credentials:
   ```toml
   [general]
   refresh_rate = "5s"

   [[providers.proxmox]]
   name = "My Proxmox Server"
   host = "https://your-proxmox-host:8006"
   user = "root@pam"
   token_id = "root@pam!your-token-name"
   token_secret = "your-token-secret-here"
   ```

### Creating a Proxmox API Token

1. Log into your Proxmox web UI
2. Go to Datacenter ’ Permissions ’ API Tokens
3. Click "Add" and create a token for your user
4. **Important**: Uncheck "Privilege Separation" for full access, or assign appropriate permissions
5. Copy the token ID and secret to your config

## Usage

```bash
# Run with default config (./config.toml)
./pulse

# Or specify a config file
./pulse --config /path/to/config.toml
```

## Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Tab` | Switch between Nodes/Containers panels |
| `j` / `“` | Move selection down |
| `k` / `‘` | Move selection up |
| `r` | Manual refresh |
| `s` | Cycle sort field (Name ’ Status ’ CPU ’ Memory) |
| `S` | Toggle sort order (ascending/descending) |
| `/` | Enter search mode |
| `Esc` | Clear search / exit search mode |
| `?` | Show help |

## License

MIT License - see LICENSE file for details.
