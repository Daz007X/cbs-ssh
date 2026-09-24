use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::ssh::SshTarget;

/// Environment variable ที่ใช้ระบุ path ของไฟล์ `server.json`
pub const SERVER_JSON_ENV: &str = "CBS_SSH_SERVER_JSON";

/// encoding เริ่มต้น ใช้เมื่อไม่ได้ระบุ `encode` ในไฟล์ config
pub const DEFAULT_ENCODE: &str = "UTF-8";

const SERVER_JSON_FILE: &str = "server.json";

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub user: String,
    pub host: String,
    pub port: u16,
    #[serde(default = "default_encode")]
    pub encode: String,
    pub password: String,
}

fn default_encode() -> String {
    DEFAULT_ENCODE.to_string()
}

impl ServerConfig {
    /// ข้อความที่ใช้แสดงในเมนูเลือกเซิร์ฟเวอร์
    pub fn label(&self) -> String {
        format!("{}  ({}@{}:{})", self.name, self.user, self.host, self.port)
    }

    /// แปลงข้อมูลเซิร์ฟเวอร์เป็นปลายทางสำหรับเชื่อมต่อ SSH
    pub fn to_target(&self) -> SshTarget {
        SshTarget {
            user: self.user.clone(),
            host: self.host.clone(),
            port: self.port,
            password: self.password.clone(),
            encode: self.encode.clone(),
        }
    }
}

pub fn load_servers(path: &Path) -> Result<Vec<ServerConfig>, String> {
    let data = fs::read_to_string(path).map_err(|e| read_error(path, &e))?;

    serde_json::from_str::<Vec<ServerConfig>>(&data).map_err(|e| read_error(path, &e))
}

fn read_error(path: &Path, err: &dyn std::fmt::Display) -> String {
    format!("ไม่สามารถอ่าน JSON ได้ {}: {}", path.display(), err)
}

/// หา path ของ `server.json` ตามลำดับความสำคัญ:
/// 1) environment variable `CBS_SSH_SERVER_JSON`
/// 2) current working directory
/// 3) โฟลเดอร์เดียวกับ executable
pub fn resolve_server_json_path() -> Result<PathBuf, String> {
    if let Ok(value) = env::var(SERVER_JSON_ENV) {
        let configured = PathBuf::from(value);
        if configured.is_file() {
            return Ok(configured);
        }

        return Err(format!(
            "ตั้งค่า {} แล้วแต่ไม่พบไฟล์: {}",
            SERVER_JSON_ENV,
            configured.display()
        ));
    }

    let candidates = candidate_paths();
    if let Some(found) = candidates.iter().find(|path| path.is_file()) {
        return Ok(found.clone());
    }

    let checked = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Err(format!(
        "ไม่พบ {SERVER_JSON_FILE} (ค้นหาใน: {checked}). ตั้งค่า {SERVER_JSON_ENV} เพื่อระบุ path เอง"
    ))
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join(SERVER_JSON_FILE));
    }

    if let Ok(exe) = env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        paths.push(exe_dir.join(SERVER_JSON_FILE));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_defaults_to_utf8_when_missing() {
        let json = r#"[{"name":"S1","user":"u","host":"h","port":22,"password":"p"}]"#;
        let servers: Vec<ServerConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(servers[0].encode, DEFAULT_ENCODE);
    }

    #[test]
    fn explicit_encode_is_kept() {
        let json =
            r#"[{"name":"S1","user":"u","host":"h","port":22,"password":"p","encode":"TIS-620"}]"#;
        let servers: Vec<ServerConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(servers[0].encode, "TIS-620");
    }

    #[test]
    fn label_shows_connection_target() {
        let server = ServerConfig {
            name: "SERVER_1".to_string(),
            user: "ubuntu".to_string(),
            host: "127.0.0.1".to_string(),
            port: 2221,
            encode: DEFAULT_ENCODE.to_string(),
            password: "secret".to_string(),
        };
        assert_eq!(server.label(), "SERVER_1  (ubuntu@127.0.0.1:2221)");
    }

    #[test]
    fn to_target_maps_all_connection_fields() {
        let server = ServerConfig {
            name: "SERVER_1".to_string(),
            user: "ubuntu".to_string(),
            host: "127.0.0.1".to_string(),
            port: 2221,
            encode: "TIS-620".to_string(),
            password: "secret".to_string(),
        };

        let target = server.to_target();
        assert_eq!(target.user, "ubuntu");
        assert_eq!(target.host, "127.0.0.1");
        assert_eq!(target.port, 2221);
        assert_eq!(target.password, "secret");
        assert_eq!(target.encode, "TIS-620");
    }

    #[test]
    fn load_servers_reports_unreadable_file() {
        let err = load_servers(Path::new("definitely-not-a-real-file.json")).unwrap_err();
        assert!(err.contains("ไม่สามารถอ่าน JSON ได้"));
    }

    #[test]
    fn load_servers_rejects_invalid_json() {
        let dir = env::temp_dir().join("cbs-ssh-test-invalid");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("server.json");
        fs::write(&file, "{ not json }").unwrap();

        let err = load_servers(&file).unwrap_err();
        assert!(err.contains("ไม่สามารถอ่าน JSON ได้"));

        let _ = fs::remove_file(&file);
    }
}
