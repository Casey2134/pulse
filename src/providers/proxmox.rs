use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;

use super::Provider;
use crate::config::ProxmoxConfig;
use crate::models::{Container, ContainerStatus, ContainerType, Node, NodeStatus};

pub struct ProxmoxProvider {
    name: String,
    client: Client,
    base_url: String,
    auth_header: String,
}

impl ProxmoxProvider {
    pub fn new(config: &ProxmoxConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()?;

        let auth_header = format!("PVEAPIToken={}={}", config.token_id, config.token_secret);

        Ok(Self {
            name: config.name.clone(),
            client,
            base_url: config.host.clone(),
            auth_header,
        })
    }

    fn fetch_node_status(&self, node: &str) -> NodeStatusData {
        let url = format!("{}/api2/json/nodes/{}/status", self.base_url, node);

        let result = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .and_then(|r| r.json::<ProxmoxResponse<ProxmoxNodeStatus>>());

        match result {
            Ok(response) => {
                let cpu = response.data.cpu.unwrap_or(0.0) * 100.0;
                let memory_used = response.data.memory.as_ref().map(|m| m.used).unwrap_or(0);
                let memory_total = response.data.memory.as_ref().map(|m| m.total).unwrap_or(0);
                let uptime = response.data.uptime.unwrap_or(0);
                NodeStatusData {
                    cpu,
                    memory_used,
                    memory_total,
                    uptime,
                }
            }
            Err(_) => NodeStatusData::default(),
        }
    }

    fn fetch_node_vms(&self, node: &str) -> Vec<Container> {
        let url = format!("{}/api2/json/nodes/{}/qemu", self.base_url, node);

        let result = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .and_then(|r| r.json::<ProxmoxResponse<Vec<ProxmoxVm>>>());

        match result {
            Ok(response) => response
                .data
                .into_iter()
                .map(|vm| Container {
                    vmid: vm.vmid,
                    name: vm.name.unwrap_or_else(|| format!("VM {}", vm.vmid)),
                    node: node.to_string(),
                    container_type: ContainerType::VM,
                    status: if vm.status == "running" {
                        ContainerStatus::Running
                    } else {
                        ContainerStatus::Stopped
                    },
                    cpu_usage: vm.cpu.unwrap_or(0.0) * 100.0,
                    memory_used: vm.mem.unwrap_or(0),
                    memory_max: vm.maxmem.unwrap_or(0),
                    uptime: vm.uptime.unwrap_or(0),
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    fn fetch_node_lxc(&self, node: &str) -> Vec<Container> {
        let url = format!("{}/api2/json/nodes/{}/lxc", self.base_url, node);

        let result = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .and_then(|r| r.json::<ProxmoxResponse<Vec<ProxmoxLxc>>>());

        match result {
            Ok(response) => response
                .data
                .into_iter()
                .map(|lxc| Container {
                    vmid: lxc.vmid,
                    name: lxc.name.unwrap_or_else(|| format!("CT {}", lxc.vmid)),
                    node: node.to_string(),
                    container_type: ContainerType::LXC,
                    status: if lxc.status == "running" {
                        ContainerStatus::Running
                    } else {
                        ContainerStatus::Stopped
                    },
                    cpu_usage: lxc.cpu.unwrap_or(0.0) * 100.0,
                    memory_used: lxc.mem.unwrap_or(0),
                    memory_max: lxc.maxmem.unwrap_or(0),
                    uptime: lxc.uptime.unwrap_or(0),
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl Provider for ProxmoxProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn fetch_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let url = format!("{}/api2/json/nodes", self.base_url);

        let response: ProxmoxResponse<Vec<ProxmoxNodeBasic>> = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()?
            .json()?;

        let mut nodes = Vec::new();

        for n in response.data {
            let status_data = if n.status == "online" {
                self.fetch_node_status(&n.node)
            } else {
                NodeStatusData::default()
            };

            nodes.push(Node {
                name: n.node,
                status: if n.status == "online" {
                    NodeStatus::Online
                } else {
                    NodeStatus::Offline
                },
                cpu_usage: status_data.cpu,
                memory_used: status_data.memory_used,
                memory_total: status_data.memory_total,
                uptime: status_data.uptime,
            });
        }

        Ok(nodes)
    }

    fn fetch_containers(&self) -> Result<Vec<Container>, Box<dyn std::error::Error>> {
        let url = format!("{}/api2/json/nodes", self.base_url);

        let response: ProxmoxResponse<Vec<ProxmoxNodeBasic>> = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()?
            .json()?;

        let mut all_containers = Vec::new();

        for n in response.data {
            if n.status == "online" {
                all_containers.extend(self.fetch_node_vms(&n.node));
                all_containers.extend(self.fetch_node_lxc(&n.node));
            }
        }

        Ok(all_containers)
    }
}

// --- Helper Structs ---

#[derive(Default)]
struct NodeStatusData {
    cpu: f64,
    memory_used: u64,
    memory_total: u64,
    uptime: u64,
}

// --- API Response Structs ---

#[derive(Debug, Deserialize)]
struct ProxmoxResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ProxmoxNodeBasic {
    node: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct ProxmoxNodeStatus {
    cpu: Option<f64>,
    memory: Option<ProxmoxMemory>,
    uptime: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ProxmoxMemory {
    total: u64,
    used: u64,
}

#[derive(Debug, Deserialize)]
struct ProxmoxVm {
    vmid: u32,
    name: Option<String>,
    status: String,
    cpu: Option<f64>,
    mem: Option<u64>,
    maxmem: Option<u64>,
    uptime: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ProxmoxLxc {
    vmid: u32,
    name: Option<String>,
    status: String,
    cpu: Option<f64>,
    mem: Option<u64>,
    maxmem: Option<u64>,
    uptime: Option<u64>,
}
