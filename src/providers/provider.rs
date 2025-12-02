use reqwest::blocking::Client;
use serde::Deserialize;

use crate::config::ProxmoxConfig;
use crate::models::{Container, ContainerStatus, Node, NodeStatus};
use super::Provider;

pub trait Provider {
    fn name(&self) -> &str;
    fn fetch_nodes(&self) -> Result<Vec<Node>, Box<dyn std::error::Error>>;
    fn fetch_containers(&self) -> Result<Vec<Container>, Box<dyn std::error::Error>>;
}