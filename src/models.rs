#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub status: NodeStatus,
    pub cpu_usage: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub uptime: u64,
}

impl Node {
    pub fn memory_percent(&self) -> f64 {
        if self.memory_total > 0 {
            (self.memory_used as f64 / self.memory_total as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Online,
    Offline,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerType {
    VM,
    LXC,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub vmid: u32,
    pub name: String,
    pub node: String,
    pub container_type: ContainerType,
    pub status: ContainerStatus,
    pub cpu_usage: f64,
    pub memory_used: u64,
    pub memory_max: u64,
    pub uptime: u64,
}

impl Container {
    pub fn memory_percent(&self) -> f64 {
        if self.memory_max > 0 {
            (self.memory_used as f64 / self.memory_max as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn type_label(&self) -> &'static str {
        match self.container_type {
            ContainerType::VM => "VM",
            ContainerType::LXC => "LXC",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStatus {
    Running,
    Stopped,
}

pub fn format_uptime(seconds: u64) -> String {
    if seconds == 0 {
        return "-".to_string();
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // format_uptime tests
    #[test]
    fn test_format_uptime_zero() {
        assert_eq!(format_uptime(0), "-");
    }

    #[test]
    fn test_format_uptime_minutes_only() {
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(300), "5m");
        assert_eq!(format_uptime(3540), "59m");
    }

    #[test]
    fn test_format_uptime_hours_and_minutes() {
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(3660), "1h 1m");
        assert_eq!(format_uptime(7200), "2h 0m");
        assert_eq!(format_uptime(86399), "23h 59m");
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(86400), "1d 0h 0m");
        assert_eq!(format_uptime(90000), "1d 1h 0m");
        assert_eq!(format_uptime(172800), "2d 0h 0m");
        assert_eq!(format_uptime(192600), "2d 5h 30m");
    }

    // format_bytes tests
    #[test]
    fn test_format_bytes_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1 KB");
        assert_eq!(format_bytes(2048), "2 KB");
        assert_eq!(format_bytes(1048575), "1024 KB");
    }

    #[test]
    fn test_format_bytes_megabytes() {
        assert_eq!(format_bytes(1048576), "1 MB");
        assert_eq!(format_bytes(536870912), "512 MB");
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        assert_eq!(format_bytes(1073741824), "1.0 GB");
        assert_eq!(format_bytes(8589934592), "8.0 GB");
    }

    #[test]
    fn test_format_bytes_terabytes() {
        assert_eq!(format_bytes(1099511627776), "1.0 TB");
        assert_eq!(format_bytes(2199023255552), "2.0 TB");
    }

    // Node tests
    #[test]
    fn test_node_memory_percent() {
        let node = Node {
            name: "test".to_string(),
            status: NodeStatus::Online,
            cpu_usage: 0.0,
            memory_used: 512,
            memory_total: 1024,
            uptime: 0,
        };
        assert_eq!(node.memory_percent(), 50.0);
    }

    #[test]
    fn test_node_memory_percent_zero_total() {
        let node = Node {
            name: "test".to_string(),
            status: NodeStatus::Online,
            cpu_usage: 0.0,
            memory_used: 512,
            memory_total: 0,
            uptime: 0,
        };
        assert_eq!(node.memory_percent(), 0.0);
    }

    // Container tests
    #[test]
    fn test_container_memory_percent() {
        let container = Container {
            vmid: 100,
            name: "test".to_string(),
            node: "node1".to_string(),
            container_type: ContainerType::LXC,
            status: ContainerStatus::Running,
            cpu_usage: 0.0,
            memory_used: 256,
            memory_max: 1024,
            uptime: 0,
        };
        assert_eq!(container.memory_percent(), 25.0);
    }

    #[test]
    fn test_container_memory_percent_zero_max() {
        let container = Container {
            vmid: 100,
            name: "test".to_string(),
            node: "node1".to_string(),
            container_type: ContainerType::VM,
            status: ContainerStatus::Stopped,
            cpu_usage: 0.0,
            memory_used: 256,
            memory_max: 0,
            uptime: 0,
        };
        assert_eq!(container.memory_percent(), 0.0);
    }

    #[test]
    fn test_container_type_label() {
        let vm = Container {
            vmid: 100,
            name: "test".to_string(),
            node: "node1".to_string(),
            container_type: ContainerType::VM,
            status: ContainerStatus::Running,
            cpu_usage: 0.0,
            memory_used: 0,
            memory_max: 0,
            uptime: 0,
        };
        assert_eq!(vm.type_label(), "VM");

        let lxc = Container {
            vmid: 101,
            name: "test".to_string(),
            node: "node1".to_string(),
            container_type: ContainerType::LXC,
            status: ContainerStatus::Running,
            cpu_usage: 0.0,
            memory_used: 0,
            memory_max: 0,
            uptime: 0,
        };
        assert_eq!(lxc.type_label(), "LXC");
    }
}
