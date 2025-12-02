# Implementing Providers

This guide explains how to add support for new infrastructure providers in Pulse.

## Provider Architecture

Providers are responsible for fetching node and container data from infrastructure platforms. Each provider implements the `Provider` trait defined in `src/providers/base.rs`.

## The Provider Trait

```rust
pub trait Provider {
    /// Returns the provider name (for display/logging)
    fn name(&self) -> &str;

    /// Fetch all nodes from this provider
    fn fetch_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>>;

    /// Fetch all containers from this provider
    fn fetch_containers(&self) -> Result<Vec<Container>, Box<dyn std::error::Error>>;
}
```

## Data Models

Your provider must return data in these formats:

### Node

```rust
use crate::models::{Node, NodeStatus};

Node {
    name: String,              // Display name (e.g., "pve-node-1")
    status: NodeStatus,        // NodeStatus::Online or NodeStatus::Offline
    cpu_usage: f64,            // CPU percentage (0.0 - 100.0)
    memory_used: u64,          // Memory used in bytes
    memory_total: u64,         // Total memory in bytes
    uptime: u64,               // Uptime in seconds
}
```

### Container

```rust
use crate::models::{Container, ContainerStatus, ContainerType};

Container {
    vmid: u32,                      // Unique ID
    name: String,                   // Display name
    node: String,                   // Which node this runs on
    container_type: ContainerType,  // ContainerType::VM or ContainerType::LXC
    status: ContainerStatus,        // ContainerStatus::Running or ContainerStatus::Stopped
    cpu_usage: f64,                 // CPU percentage (0.0 - 100.0)
    memory_used: u64,               // Memory used in bytes
    memory_max: u64,                // Max memory in bytes
    uptime: u64,                    // Uptime in seconds (0 if stopped)
}
```

## Step-by-Step: Adding a New Provider

### 1. Create the Provider File

Create `src/providers/yourprovider.rs`:

```rust
use reqwest::blocking::Client;
use std::time::Duration;

use super::Provider;
use crate::config::YourProviderConfig;
use crate::models::{Container, ContainerStatus, ContainerType, Node, NodeStatus};

pub struct YourProvider {
    name: String,
    client: Client,
    // Add your connection details
}

impl YourProvider {
    pub fn new(config: &YourProviderConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()?;

        Ok(Self {
            name: config.name.clone(),
            client,
        })
    }
}

impl Provider for YourProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn fetch_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        // Implement API calls to fetch nodes
        let nodes = vec![
            Node {
                name: "example-node".to_string(),
                status: NodeStatus::Online,
                cpu_usage: 25.0,
                memory_used: 8 * 1024 * 1024 * 1024,  // 8 GB
                memory_total: 32 * 1024 * 1024 * 1024, // 32 GB
                uptime: 86400 * 30,  // 30 days
            }
        ];
        Ok(nodes)
    }

    fn fetch_containers(&self) -> Result<Vec<Container>, Box<dyn std::error::Error>> {
        // Implement API calls to fetch containers
        let containers = vec![
            Container {
                vmid: 100,
                name: "web-server".to_string(),
                node: "example-node".to_string(),
                container_type: ContainerType::LXC,
                status: ContainerStatus::Running,
                cpu_usage: 5.0,
                memory_used: 512 * 1024 * 1024,   // 512 MB
                memory_max: 2 * 1024 * 1024 * 1024, // 2 GB
                uptime: 86400 * 7,  // 7 days
            }
        ];
        Ok(containers)
    }
}
```

### 2. Add Configuration

Update `src/config.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct ProvidersConfig {
    pub proxmox: Option<Vec<ProxmoxConfig>>,
    pub yourprovider: Option<Vec<YourProviderConfig>>,  // Add this
}

#[derive(Debug, Deserialize)]
pub struct YourProviderConfig {
    pub name: String,
    pub host: String,
    pub api_key: String,
    // Add your config fields
}
```

### 3. Register the Provider

Update `src/providers/mod.rs`:

```rust
mod base;
mod proxmox;
mod yourprovider;  // Add this

pub use base::Provider;
pub use proxmox::ProxmoxProvider;
pub use yourprovider::YourProvider;  // Add this
```

Update `src/main.rs`:

```rust
use crate::providers::{Provider, ProxmoxProvider, YourProvider};

// In main(), after proxmox initialization:
if let Some(configs) = &config.providers.yourprovider {
    for provider_config in configs {
        match YourProvider::new(provider_config) {
            Ok(provider) => {
                providers.push(Box::new(provider));
            }
            Err(e) => {
                eprintln!("Failed to create provider '{}': {}", provider_config.name, e);
            }
        }
    }
}
```

### 4. Update Example Config

Add to `config.example.toml`:

```toml
# [[providers.yourprovider]]
# name = "My Provider"
# host = "https://your-host:port"
# api_key = "your-api-key"
```

## Best Practices

### Error Handling

- Use `Result` return types for all API operations
- Include meaningful error messages
- Don't panic on API failures - return errors gracefully

```rust
fn fetch_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let response = self.client
        .get(&format!("{}/api/nodes", self.host))
        .send()
        .map_err(|e| format!("Failed to connect: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    // Parse response...
    Ok(nodes)
}
```

### HTTP Client Configuration

Always set timeouts to prevent hanging:

```rust
let client = Client::builder()
    .timeout(Duration::from_secs(10))
    .connect_timeout(Duration::from_secs(5))
    .build()?;
```

### SSL/TLS

For self-signed certificates (common in homelabs):

```rust
let client = Client::builder()
    .danger_accept_invalid_certs(true)  // Use with caution
    .build()?;
```

### API Response Parsing

Use serde for JSON deserialization:

```rust
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ApiNode {
    name: String,
    status: String,
    cpu: Option<f64>,
    // ...
}

let response: ApiResponse<Vec<ApiNode>> = self.client
    .get(&url)
    .send()?
    .json()?;
```

## Testing Your Provider

Add tests in your provider file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node_response() {
        // Test your response parsing
    }

    #[test]
    fn test_status_conversion() {
        // Test status string to enum conversion
    }
}
```

## Provider Ideas

Here are some infrastructure platforms that would be great to support:

| Platform | API Docs |
|----------|----------|
| Docker | [Docker Engine API](https://docs.docker.com/engine/api/) |
| Kubernetes | [Kubernetes API](https://kubernetes.io/docs/reference/kubernetes-api/) |
| Portainer | [Portainer API](https://docs.portainer.io/api/docs) |
| Unraid | [Unraid API](https://forums.unraid.net/topic/85995-api/) |
| TrueNAS | [TrueNAS API](https://www.truenas.com/docs/api/) |
| Proxmox Backup Server | [PBS API](https://pbs.proxmox.com/docs/api-viewer/) |

## Questions?

Open an issue on GitHub if you need help implementing a provider!
