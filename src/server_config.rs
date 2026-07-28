
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub user: String,
    pub host: String,
    pub port: u16,
    pub encode: String,
    pub password: String,
}