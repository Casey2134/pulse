use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub providers: ProvidersConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub refresh_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct ProvidersConfig {
    pub proxmox: Option<Vec<ProxmoxConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct ProxmoxConfig {
    pub name: String,
    pub host: String,
    pub user: String,
    pub token_id: String,
    pub token_secret: String,
}

pub fn load(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
[general]
refresh_rate = "5s"

[[providers.proxmox]]
name = "Test Server"
host = "https://192.168.1.100:8006"
user = "root@pam"
token_id = "root@pam!test-token"
token_secret = "12345678-1234-1234-1234-123456789012"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.general.refresh_rate, "5s");
        assert!(config.providers.proxmox.is_some());

        let proxmox_configs = config.providers.proxmox.unwrap();
        assert_eq!(proxmox_configs.len(), 1);

        let proxmox = &proxmox_configs[0];
        assert_eq!(proxmox.name, "Test Server");
        assert_eq!(proxmox.host, "https://192.168.1.100:8006");
        assert_eq!(proxmox.user, "root@pam");
        assert_eq!(proxmox.token_id, "root@pam!test-token");
        assert_eq!(proxmox.token_secret, "12345678-1234-1234-1234-123456789012");
    }

    #[test]
    fn test_parse_multiple_proxmox_providers() {
        let toml_str = r#"
[general]
refresh_rate = "10s"

[[providers.proxmox]]
name = "Server 1"
host = "https://server1:8006"
user = "admin@pam"
token_id = "admin@pam!token1"
token_secret = "secret1"

[[providers.proxmox]]
name = "Server 2"
host = "https://server2:8006"
user = "admin@pam"
token_id = "admin@pam!token2"
token_secret = "secret2"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        let proxmox_configs = config.providers.proxmox.unwrap();

        assert_eq!(proxmox_configs.len(), 2);
        assert_eq!(proxmox_configs[0].name, "Server 1");
        assert_eq!(proxmox_configs[1].name, "Server 2");
    }

    #[test]
    fn test_parse_no_proxmox_providers() {
        let toml_str = r#"
[general]
refresh_rate = "5s"

[providers]
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.providers.proxmox.is_none());
    }

    #[test]
    fn test_parse_missing_field_fails() {
        let toml_str = r#"
[general]
refresh_rate = "5s"

[[providers.proxmox]]
name = "Test Server"
host = "https://192.168.1.100:8006"
# Missing user, token_id, token_secret
"#;

        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_toml_fails() {
        let toml_str = "this is not valid toml [[[";

        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }
}