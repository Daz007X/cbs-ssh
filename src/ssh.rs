use std::io::ErrorKind;
use std::process::{Command, Stdio};

const LUIT_BIN: &str = "luit";
const SSHPASS_BIN: &str = "sshpass";
const SSH_BIN: &str = "ssh";
const ACCEPT_NEW_HOST_KEY: &str = "StrictHostKeyChecking=accept-new";

/// ปลายทางที่ใช้เชื่อมต่อ SSH (รวบทุกข้อมูลที่ต้องใช้ไว้ที่เดียว)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    pub user: String,
    pub host: String,
    pub port: u16,
    pub password: String,
    pub encode: String,
}

impl SshTarget {
    pub fn new(
        user: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        password: impl Into<String>,
        encode: impl Into<String>,
    ) -> Self {
        Self {
            user: user.into(),
            host: host.into(),
            port,
            password: password.into(),
            encode: encode.into(),
        }
    }
}

/// เชื่อมต่อ SSH ด้วยรหัสผ่านผ่านคำสั่ง `sshpass`
///
/// เทียบเท่าคำสั่ง shell:
/// `luit -encoding <encode> sshpass -p <password> ssh <user>@<host>`
pub fn connect(target: &SshTarget) -> Result<(), String> {
    let connection_string = format!("{}@{}", target.user, target.host);

    let status = Command::new(LUIT_BIN)
        .arg("-encoding")
        .arg(&target.encode)
        .arg(SSHPASS_BIN)
        .arg("-p")
        .arg(&target.password)
        .arg(SSH_BIN)
        .arg("-t")
        .arg("-p")
        .arg(target.port.to_string())
        .arg("-o")
        .arg(ACCEPT_NEW_HOST_KEY)
        .arg(connection_string)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(spawn_error)?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("SSH จบด้วยสถานะผิดปกติ: {status}"))
    }
}

fn spawn_error(err: std::io::Error) -> String {
    match err.kind() {
        ErrorKind::NotFound => format!(
            "ไม่พบคำสั่ง {LUIT_BIN} ใน PATH. ติดตั้งก่อนใช้งาน เช่น Ubuntu/WSL: sudo apt install luit sshpass"
        ),
        _ => format!("เริ่มคำสั่ง {LUIT_BIN} ไม่สำเร็จ: {err}"),
    }
}
