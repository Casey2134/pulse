use crate::models::{Node, NodeStatus, Container, ContainerStatus};

pub fn fake_nodes() -> Vec<Node> {
    vec![
        Node {
            name: "pve1".to_string(),
            status: NodeStatus::Online,
            cpu_usage: 67.0,
            memory_usage: 48.0,
        },
        Node {
            name: "pve2".to_string(),
            status: NodeStatus::Online,
            cpu_usage: 34.0,
            memory_usage: 71.0,
        },
        Node {
            name: "pve3".to_string(),
            status: NodeStatus::Offline,
            cpu_usage: 0.0,
            memory_usage: 0.0,
        },
    ]
}

pub fn fake_containers() -> Vec<Container> {
    vec![
        Container {
            name: "jellyfin".to_string(),
            node: "pve1".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 12.0,
            memory_mb: 2100,
        },
        Container {
            name: "frigate".to_string(),
            node: "pve1".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 45.0,
            memory_mb: 4300,
        },
        Container {
            name: "radarr".to_string(),
            node: "pve1".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 2.0,
            memory_mb: 800,
        },
        Container {
            name: "sonarr".to_string(),
            node: "pve2".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 1.0,
            memory_mb: 700,
        },
        Container {
            name: "postgres".to_string(),
            node: "pve2".to_string(),
            status: ContainerStatus::Running,
            cpu_usage: 8.0,
            memory_mb: 1200,
        },
        Container {
            name: "redis".to_string(),
            node: "pve2".to_string(),
            status: ContainerStatus::Stopped,
            cpu_usage: 0.0,
            memory_mb: 0,
        },
    ]
}