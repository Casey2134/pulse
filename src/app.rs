use std::time::Instant;

use crate::models::{Container, ContainerStatus, Node, NodeStatus};
use crate::providers::Provider;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Nodes,
    Containers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Status,
    Cpu,
    Memory,
}

impl SortField {
    pub fn next(self) -> Self {
        match self {
            SortField::Name => SortField::Status,
            SortField::Status => SortField::Cpu,
            SortField::Cpu => SortField::Memory,
            SortField::Memory => SortField::Name,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortField::Name => "Name",
            SortField::Status => "Status",
            SortField::Cpu => "CPU",
            SortField::Memory => "Memory",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    pub running: bool,
    pub active_panel: Panel,
    pub nodes: Vec<Node>,
    pub containers: Vec<Container>,
    pub node_index: usize,
    pub container_index: usize,
    pub error_message: Option<String>,
    pub last_refresh: Option<Instant>,
    pub sort_field: SortField,
    pub sort_ascending: bool,
    pub input_mode: InputMode,
    pub search_query: String,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            active_panel: Panel::Nodes,
            nodes: Vec::new(),
            containers: Vec::new(),
            node_index: 0,
            container_index: 0,
            error_message: None,
            last_refresh: None,
            sort_field: SortField::Name,
            sort_ascending: true,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            show_help: false,
        }
    }

    pub fn refresh(&mut self, providers: &[Box<dyn Provider>]) {
        self.error_message = None;
        let mut all_nodes = Vec::new();
        let mut all_containers = Vec::new();
        let mut had_error = false;

        for provider in providers {
            match provider.fetch_nodes() {
                Ok(nodes) => {
                    all_nodes.extend(nodes);
                }
                Err(e) => {
                    self.error_message = Some(format!("Error fetching nodes: {}", e));
                    had_error = true;
                }
            }

            match provider.fetch_containers() {
                Ok(containers) => {
                    all_containers.extend(containers);
                }
                Err(e) => {
                    self.error_message = Some(format!("Error fetching containers: {}", e));
                    had_error = true;
                }
            }
        }

        // Only update data if we got new data, otherwise keep existing data
        // This prevents the UI from going blank on transient network errors
        if !all_nodes.is_empty() || !had_error {
            self.nodes = all_nodes;
        }
        if !all_containers.is_empty() || !had_error {
            self.containers = all_containers;
        }

        self.last_refresh = Some(Instant::now());

        self.apply_sort();

        if self.node_index >= self.filtered_nodes().len() {
            self.node_index = self.filtered_nodes().len().saturating_sub(1);
        }
        if self.container_index >= self.filtered_containers().len() {
            self.container_index = self.filtered_containers().len().saturating_sub(1);
        }
    }

    fn apply_sort(&mut self) {
        let ascending = self.sort_ascending;

        match self.sort_field {
            SortField::Name => {
                self.nodes.sort_by(|a, b| {
                    if ascending {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
                self.containers.sort_by(|a, b| {
                    if ascending {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
            }
            SortField::Status => {
                self.nodes.sort_by(|a, b| {
                    let a_val = matches!(a.status, NodeStatus::Online);
                    let b_val = matches!(b.status, NodeStatus::Online);
                    if ascending {
                        b_val.cmp(&a_val)
                    } else {
                        a_val.cmp(&b_val)
                    }
                });
                self.containers.sort_by(|a, b| {
                    let a_val = matches!(a.status, ContainerStatus::Running);
                    let b_val = matches!(b.status, ContainerStatus::Running);
                    if ascending {
                        b_val.cmp(&a_val)
                    } else {
                        a_val.cmp(&b_val)
                    }
                });
            }
            SortField::Cpu => {
                self.nodes.sort_by(|a, b| {
                    if ascending {
                        a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()
                    } else {
                        b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
                    }
                });
                self.containers.sort_by(|a, b| {
                    if ascending {
                        a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()
                    } else {
                        b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
                    }
                });
            }
            SortField::Memory => {
                self.nodes.sort_by(|a, b| {
                    let a_pct = a.memory_percent();
                    let b_pct = b.memory_percent();
                    if ascending {
                        a_pct.partial_cmp(&b_pct).unwrap()
                    } else {
                        b_pct.partial_cmp(&a_pct).unwrap()
                    }
                });
                self.containers.sort_by(|a, b| {
                    let a_pct = a.memory_percent();
                    let b_pct = b.memory_percent();
                    if ascending {
                        a_pct.partial_cmp(&b_pct).unwrap()
                    } else {
                        b_pct.partial_cmp(&a_pct).unwrap()
                    }
                });
            }
        }
    }

    pub fn filtered_nodes(&self) -> Vec<&Node> {
        if self.search_query.is_empty() {
            self.nodes.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.nodes
                .iter()
                .filter(|n| n.name.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn filtered_containers(&self) -> Vec<&Container> {
        if self.search_query.is_empty() {
            self.containers.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.containers
                .iter()
                .filter(|c| {
                    c.name.to_lowercase().contains(&query) || c.node.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.filtered_nodes().get(self.node_index).copied()
    }

    pub fn selected_container(&self) -> Option<&Container> {
        self.filtered_containers()
            .get(self.container_index)
            .copied()
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Nodes => Panel::Containers,
            Panel::Containers => Panel::Nodes,
        };
    }

    pub fn select_next(&mut self) {
        match self.active_panel {
            Panel::Nodes => {
                let max = self.filtered_nodes().len().saturating_sub(1);
                if self.node_index < max {
                    self.node_index += 1;
                }
            }
            Panel::Containers => {
                let max = self.filtered_containers().len().saturating_sub(1);
                if self.container_index < max {
                    self.container_index += 1;
                }
            }
        }
    }

    pub fn select_previous(&mut self) {
        match self.active_panel {
            Panel::Nodes => {
                self.node_index = self.node_index.saturating_sub(1);
            }
            Panel::Containers => {
                self.container_index = self.container_index.saturating_sub(1);
            }
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_field = self.sort_field.next();
        self.apply_sort();
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.apply_sort();
    }

    pub fn enter_search_mode(&mut self) {
        self.input_mode = InputMode::Search;
    }

    pub fn exit_search_mode(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.node_index = 0;
        self.container_index = 0;
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.node_index = 0;
        self.container_index = 0;
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
        self.node_index = 0;
        self.container_index = 0;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn time_since_refresh(&self) -> String {
        match self.last_refresh {
            Some(instant) => {
                let secs = instant.elapsed().as_secs();
                if secs < 60 {
                    format!("{}s ago", secs)
                } else {
                    format!("{}m ago", secs / 60)
                }
            }
            None => "never".to_string(),
        }
    }

    pub fn nodes_summary(&self) -> (usize, usize) {
        let online = self
            .nodes
            .iter()
            .filter(|n| n.status == NodeStatus::Online)
            .count();
        (online, self.nodes.len())
    }

    pub fn containers_summary(&self) -> (usize, usize) {
        let running = self
            .containers
            .iter()
            .filter(|c| c.status == ContainerStatus::Running)
            .count();
        (running, self.containers.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ContainerType;

    fn create_test_node(name: &str, status: NodeStatus, cpu: f64) -> Node {
        Node {
            name: name.to_string(),
            status,
            cpu_usage: cpu,
            memory_used: 512,
            memory_total: 1024,
            uptime: 3600,
        }
    }

    fn create_test_container(
        name: &str,
        node: &str,
        status: ContainerStatus,
        cpu: f64,
    ) -> Container {
        Container {
            vmid: 100,
            name: name.to_string(),
            node: node.to_string(),
            container_type: ContainerType::LXC,
            status,
            cpu_usage: cpu,
            memory_used: 256,
            memory_max: 1024,
            uptime: 3600,
        }
    }

    // App initialization tests
    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(app.running);
        assert_eq!(app.active_panel, Panel::Nodes);
        assert!(app.nodes.is_empty());
        assert!(app.containers.is_empty());
        assert_eq!(app.node_index, 0);
        assert_eq!(app.container_index, 0);
        assert!(app.error_message.is_none());
        assert_eq!(app.sort_field, SortField::Name);
        assert!(app.sort_ascending);
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.search_query.is_empty());
        assert!(!app.show_help);
    }

    // Navigation tests
    #[test]
    fn test_next_panel() {
        let mut app = App::new();
        assert_eq!(app.active_panel, Panel::Nodes);

        app.next_panel();
        assert_eq!(app.active_panel, Panel::Containers);

        app.next_panel();
        assert_eq!(app.active_panel, Panel::Nodes);
    }

    #[test]
    fn test_select_next_nodes() {
        let mut app = App::new();
        app.nodes = vec![
            create_test_node("node1", NodeStatus::Online, 10.0),
            create_test_node("node2", NodeStatus::Online, 20.0),
            create_test_node("node3", NodeStatus::Online, 30.0),
        ];
        app.active_panel = Panel::Nodes;

        assert_eq!(app.node_index, 0);
        app.select_next();
        assert_eq!(app.node_index, 1);
        app.select_next();
        assert_eq!(app.node_index, 2);
        app.select_next(); // Should not go past last item
        assert_eq!(app.node_index, 2);
    }

    #[test]
    fn test_select_previous_nodes() {
        let mut app = App::new();
        app.nodes = vec![
            create_test_node("node1", NodeStatus::Online, 10.0),
            create_test_node("node2", NodeStatus::Online, 20.0),
        ];
        app.active_panel = Panel::Nodes;
        app.node_index = 1;

        app.select_previous();
        assert_eq!(app.node_index, 0);
        app.select_previous(); // Should not go below 0
        assert_eq!(app.node_index, 0);
    }

    #[test]
    fn test_select_next_containers() {
        let mut app = App::new();
        app.containers = vec![
            create_test_container("ct1", "node1", ContainerStatus::Running, 10.0),
            create_test_container("ct2", "node1", ContainerStatus::Running, 20.0),
        ];
        app.active_panel = Panel::Containers;

        assert_eq!(app.container_index, 0);
        app.select_next();
        assert_eq!(app.container_index, 1);
    }

    // Sort tests
    #[test]
    fn test_sort_field_cycle() {
        assert_eq!(SortField::Name.next(), SortField::Status);
        assert_eq!(SortField::Status.next(), SortField::Cpu);
        assert_eq!(SortField::Cpu.next(), SortField::Memory);
        assert_eq!(SortField::Memory.next(), SortField::Name);
    }

    #[test]
    fn test_sort_field_label() {
        assert_eq!(SortField::Name.label(), "Name");
        assert_eq!(SortField::Status.label(), "Status");
        assert_eq!(SortField::Cpu.label(), "CPU");
        assert_eq!(SortField::Memory.label(), "Memory");
    }

    #[test]
    fn test_cycle_sort() {
        let mut app = App::new();
        assert_eq!(app.sort_field, SortField::Name);

        app.cycle_sort();
        assert_eq!(app.sort_field, SortField::Status);

        app.cycle_sort();
        assert_eq!(app.sort_field, SortField::Cpu);
    }

    #[test]
    fn test_toggle_sort_order() {
        let mut app = App::new();
        assert!(app.sort_ascending);

        app.toggle_sort_order();
        assert!(!app.sort_ascending);

        app.toggle_sort_order();
        assert!(app.sort_ascending);
    }

    // Search/filter tests
    #[test]
    fn test_search_mode() {
        let mut app = App::new();
        assert_eq!(app.input_mode, InputMode::Normal);

        app.enter_search_mode();
        assert_eq!(app.input_mode, InputMode::Search);

        app.exit_search_mode();
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_search_query() {
        let mut app = App::new();

        app.push_search_char('t');
        app.push_search_char('e');
        app.push_search_char('s');
        app.push_search_char('t');
        assert_eq!(app.search_query, "test");

        app.pop_search_char();
        assert_eq!(app.search_query, "tes");

        app.clear_search();
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn test_filtered_nodes() {
        let mut app = App::new();
        app.nodes = vec![
            create_test_node("alpha", NodeStatus::Online, 10.0),
            create_test_node("beta", NodeStatus::Online, 20.0),
            create_test_node("alphabeta", NodeStatus::Online, 30.0),
        ];

        // No filter - all nodes
        assert_eq!(app.filtered_nodes().len(), 3);

        // Filter by "alpha"
        app.search_query = "alpha".to_string();
        let filtered = app.filtered_nodes();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|n| n.name == "alpha"));
        assert!(filtered.iter().any(|n| n.name == "alphabeta"));
    }

    #[test]
    fn test_filtered_containers_by_name() {
        let mut app = App::new();
        app.containers = vec![
            create_test_container("web-server", "node1", ContainerStatus::Running, 10.0),
            create_test_container("database", "node1", ContainerStatus::Running, 20.0),
            create_test_container("web-cache", "node2", ContainerStatus::Running, 30.0),
        ];

        app.search_query = "web".to_string();
        let filtered = app.filtered_containers();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filtered_containers_by_node() {
        let mut app = App::new();
        app.containers = vec![
            create_test_container("ct1", "production", ContainerStatus::Running, 10.0),
            create_test_container("ct2", "staging", ContainerStatus::Running, 20.0),
            create_test_container("ct3", "production", ContainerStatus::Running, 30.0),
        ];

        app.search_query = "prod".to_string();
        let filtered = app.filtered_containers();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut app = App::new();
        app.nodes = vec![create_test_node("ProductionNode", NodeStatus::Online, 10.0)];

        app.search_query = "production".to_string();
        assert_eq!(app.filtered_nodes().len(), 1);

        app.search_query = "PRODUCTION".to_string();
        assert_eq!(app.filtered_nodes().len(), 1);
    }

    // Summary tests
    #[test]
    fn test_nodes_summary() {
        let mut app = App::new();
        app.nodes = vec![
            create_test_node("node1", NodeStatus::Online, 10.0),
            create_test_node("node2", NodeStatus::Offline, 0.0),
            create_test_node("node3", NodeStatus::Online, 30.0),
        ];

        let (online, total) = app.nodes_summary();
        assert_eq!(online, 2);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_containers_summary() {
        let mut app = App::new();
        app.containers = vec![
            create_test_container("ct1", "node1", ContainerStatus::Running, 10.0),
            create_test_container("ct2", "node1", ContainerStatus::Stopped, 0.0),
            create_test_container("ct3", "node1", ContainerStatus::Running, 30.0),
            create_test_container("ct4", "node1", ContainerStatus::Stopped, 0.0),
        ];

        let (running, total) = app.containers_summary();
        assert_eq!(running, 2);
        assert_eq!(total, 4);
    }

    // Selection tests
    #[test]
    fn test_selected_node() {
        let mut app = App::new();
        app.nodes = vec![
            create_test_node("node1", NodeStatus::Online, 10.0),
            create_test_node("node2", NodeStatus::Online, 20.0),
        ];

        app.node_index = 0;
        assert_eq!(app.selected_node().unwrap().name, "node1");

        app.node_index = 1;
        assert_eq!(app.selected_node().unwrap().name, "node2");
    }

    #[test]
    fn test_selected_node_empty() {
        let app = App::new();
        assert!(app.selected_node().is_none());
    }

    // Misc tests
    #[test]
    fn test_quit() {
        let mut app = App::new();
        assert!(app.running);

        app.quit();
        assert!(!app.running);
    }

    #[test]
    fn test_toggle_help() {
        let mut app = App::new();
        assert!(!app.show_help);

        app.toggle_help();
        assert!(app.show_help);

        app.toggle_help();
        assert!(!app.show_help);
    }

    #[test]
    fn test_time_since_refresh_never() {
        let app = App::new();
        assert_eq!(app.time_since_refresh(), "never");
    }
}
