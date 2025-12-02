# Pulse Architecture

This document describes the internal architecture of Pulse for developers who want to contribute or understand how the codebase is structured.

## Overview

Pulse is a terminal-based infrastructure monitoring tool built with Rust. It follows a modular architecture with clear separation between data fetching, application state, and UI rendering.

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                              │
│                   (Event Loop & Init)                       │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│    config.rs    │  │     app.rs      │  │     ui.rs       │
│ (Configuration) │  │ (App State)     │  │ (Rendering)     │
└─────────────────┘  └─────────────────┘  └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   providers/    │
                    │ (Data Sources)  │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   models.rs     │
                    │ (Data Types)    │
                    └─────────────────┘
```

## Core Modules

### `main.rs` - Application Entry Point

The main module handles:
- CLI argument parsing via `clap`
- Configuration loading
- Provider initialization
- Terminal setup/teardown with `ratatui`
- The main event loop (keyboard input + auto-refresh)

```rust
// Simplified event loop structure
while app.running {
    terminal.draw(|frame| ui::draw(frame, &app))?;

    if event::poll(Duration::from_millis(100))? {
        // Handle keyboard input
    }

    if last_refresh.elapsed() >= refresh_interval {
        app.refresh(&providers);
    }
}
```

### `app.rs` - Application State

The `App` struct holds all application state:

| Field | Purpose |
|-------|---------|
| `running` | Controls the main loop |
| `active_panel` | Which panel has focus (Nodes/Containers) |
| `nodes` / `containers` | Current data from providers |
| `node_index` / `container_index` | Selection state |
| `sort_field` / `sort_ascending` | Sorting configuration |
| `search_query` | Current filter text |
| `input_mode` | Normal vs Search mode |
| `error_message` | Last error to display |
| `last_refresh` | Timestamp for "X ago" display |

Key methods:
- `refresh()` - Fetches data from all providers
- `filtered_nodes()` / `filtered_containers()` - Apply search filter
- `select_next()` / `select_previous()` - Navigation
- `cycle_sort()` / `toggle_sort_order()` - Sorting

### `ui.rs` - Terminal UI Rendering

Uses `ratatui` for TUI rendering. The UI is composed of several draw functions:

```
┌─────────────────────────────────────────────────────────────┐
│ draw_header()     - Title, stats, refresh time              │
├─────────────────────────────────────────────────────────────┤
│ draw_nodes()      │ draw_containers()                       │
│ (35%)             │ (65%)                                   │
├─────────────────────────────────────────────────────────────┤
│ draw_detail_panel() - Selected item details with gauges    │
├─────────────────────────────────────────────────────────────┤
│ draw_status_bar() - Keybindings or search input            │
└─────────────────────────────────────────────────────────────┘
│ draw_help_popup() - Overlay when ? is pressed              │
```

### `models.rs` - Data Structures

Defines the core data types:

```rust
pub struct Node {
    pub name: String,
    pub status: NodeStatus,      // Online | Offline
    pub cpu_usage: f64,          // Percentage (0-100)
    pub memory_used: u64,        // Bytes
    pub memory_total: u64,       // Bytes
    pub uptime: u64,             // Seconds
}

pub struct Container {
    pub vmid: u32,
    pub name: String,
    pub node: String,
    pub container_type: ContainerType,  // VM | LXC
    pub status: ContainerStatus,        // Running | Stopped
    pub cpu_usage: f64,
    pub memory_used: u64,
    pub memory_max: u64,
    pub uptime: u64,
}
```

Also includes helper functions:
- `format_uptime()` - Converts seconds to "Xd Xh Xm"
- `format_bytes()` - Converts bytes to "X.X GB"

### `config.rs` - Configuration

Handles TOML configuration parsing:

```rust
pub struct Config {
    pub general: GeneralConfig,
    pub providers: ProvidersConfig,
}

pub struct ProxmoxConfig {
    pub name: String,
    pub host: String,
    pub user: String,
    pub token_id: String,
    pub token_secret: String,
}
```

### `providers/` - Data Source Abstraction

The provider system allows multiple infrastructure backends:

```rust
// providers/base.rs
pub trait Provider {
    fn name(&self) -> &str;
    fn fetch_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>>;
    fn fetch_containers(&self) -> Result<Vec<Container>, Box<dyn std::error::Error>>;
}
```

Current providers:
- `ProxmoxProvider` - Proxmox VE API integration

See [PROVIDERS.md](./PROVIDERS.md) for details on implementing new providers.

## Data Flow

1. **Startup**: Config loaded → Providers initialized → Initial refresh
2. **Refresh cycle**:
   ```
   Provider.fetch_nodes() ──┐
                            ├──► App.nodes/containers ──► UI render
   Provider.fetch_containers()┘
   ```
3. **User input**: Keyboard event → App state mutation → UI re-render

## Error Handling

- Provider errors are caught and stored in `app.error_message`
- On transient errors, existing data is preserved (UI doesn't go blank)
- Errors are displayed in the status bar

## Testing

Tests are co-located with modules using `#[cfg(test)]`:

```
src/models.rs   - Tests for format_uptime, format_bytes, memory calculations
src/app.rs      - Tests for navigation, sorting, filtering, state management
src/config.rs   - Tests for TOML parsing
```

Run tests with:
```bash
cargo test
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal manipulation |
| `clap` | CLI argument parsing |
| `reqwest` | HTTP client for API calls |
| `serde` | Serialization/deserialization |
| `toml` | Config file parsing |
