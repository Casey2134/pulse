# Pulse

A real-time TUI (Terminal User Interface) monitor for your homelab infrastructure. Currently supports Proxmox VE.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

<img width="1470" alt="Screenshot 2025-12-02 at 3 17 08 AM" src="https://github.com/user-attachments/assets/99390bb7-3767-419a-b472-c21ee4ce95d6" />
<img width="1470" alt="Screenshot 2025-12-02 at 3 17 19 AM" src="https://github.com/user-attachments/assets/57d3ef36-a0c8-45ff-b026-6096784f3fe6" />
<img width="733" height="546" alt="Screenshot 2025-12-02 at 3 19 53 AM" src="https://github.com/user-attachments/assets/31d2a2c8-b83d-4062-8de2-c91e8eecb8e7" />
<img width="355" height="116" alt="Screenshot 2025-12-02 at 3 23 49 AM" src="https://github.com/user-attachments/assets/0aacfb4f-a235-4e7e-854f-325a9bc7544c" />


## Features

- Real-time monitoring of Proxmox nodes and containers (VMs & LXC)
- CPU and memory usage with visual gauges
- Uptime tracking for nodes and containers
- Search/filter functionality
- Sortable by name, status, CPU, or memory
- Auto-refresh every 5 seconds
- Keyboard-driven interface

## Installation

### Download Binary (Recommended)

Download the latest release for your platform from the [Releases page](https://github.com/casey2134/pulse/releases).

| Platform | Download |
|----------|----------|
| Linux (x86_64) | `pulse-linux-amd64` |
| macOS (Intel) | `pulse-macos-amd64` |
| macOS (Apple Silicon) | `pulse-macos-arm64` |
| Windows | `pulse-windows-amd64.exe` |

#### Linux / macOS
```bash
# Download (replace URL with latest release)
curl -LO https://github.com/YOUR_USERNAME/pulse/releases/latest/download/pulse-linux-amd64

# Make executable
chmod +x pulse-linux-amd64

# Move to PATH (optional)
sudo mv pulse-linux-amd64 /usr/local/bin/pulse

# Run
pulse --config config.toml
```

#### Windows

1. Download `pulse-windows-amd64.exe` from the [Releases page](https://github.com/casey2134/pulse/releases)
2. Open PowerShell or Command Prompt
3. Run:
```powershell
   .\pulse-windows-amd64.exe --config config.toml
```

### Build from Source
```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/pulse.git
cd pulse

# Build the project
cargo build --release

# The binary will be at ./target/release/pulse
```

#### Requirements

- Rust 1.75+ (uses 2024 edition)
- A Proxmox VE server with API access

## Configuration

1. Create a config file:
```bash
   curl -LO https://raw.githubusercontent.com/YOUR_USERNAME/pulse/main/config.example.toml
   mv config.example.toml config.toml
```

2. Edit `config.toml` with your Proxmox credentials:
```toml
   [general]
   refresh_rate = "5s"

   [[providers.proxmox]]
   name = "My Proxmox Server"
   host = "https://your-proxmox-host:8006"
   user = "root@pam"
   token_id = "your-token-name"
   token_secret = "your-token-secret-here"
```

### Creating a Proxmox API Token

1. Log into your Proxmox web UI
2. Go to **Datacenter → Permissions → API Tokens**
3. Click **Add** and create a token for your user
4. **Important**: Uncheck "Privilege Separation" for full access, or assign appropriate permissions
5. Copy the token ID and secret to your config

## Usage
```bash
# Run with default config (./config.toml)
pulse

# Or specify a config file
pulse --config /path/to/config.toml

# Show help
pulse --help
```

## Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Tab` | Switch between Nodes/Containers panels |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `r` | Manual refresh |
| `s` | Cycle sort field (Name → Status → CPU → Memory) |
| `S` | Toggle sort order (ascending/descending) |
| `/` | Enter search mode |
| `Esc` | Clear search / exit search mode |
| `?` | Show help |

## Roadmap

- [ ] Docker provider
- [ ] Portainer provider
- [ ] TrueNAS provider
- [ ] Alerts and notifications
- [ ] Container actions (start/stop)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details.
