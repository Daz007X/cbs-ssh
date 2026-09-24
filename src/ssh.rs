use std::io::ErrorKind;
use std::process::{Command, Stdio};

/// เชื่อมต่อ SSH ด้วยรหัสผ่านผ่านคำสั่ง `sshpass`
pub fn connect_ssh_with_password(
    user: &str,
    host: &str,
    port: u16,
    password: &str,
    encode: &str,
) -> Result<(), String> {
    let connection_string = format!("{user}@{host}");

    let mut child = Command::new("sshpass")
        // ส่งรหัสผ่านผ่าน env SSHPASS (ใช้ sshpass -e) เพื่อไม่ให้รหัสผ่าน
        // ปรากฏใน process list (ps)
        .env("SSHPASS", password)
        .env("LC_ALL", "C")
        .env("LANG", encode)
        .arg("-e")
        .arg("ssh")
        .arg("-t")
        .arg("-p")
        .arg(port.to_string())
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg(connection_string)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => {
                "ไม่พบคำสั่ง sshpass ใน PATH. ติดตั้งก่อนใช้งาน เช่น Ubuntu/WSL: sudo apt install sshpass"
                    .to_string()
            }
            _ => format!("เริ่มคำสั่ง sshpass ไม่สำเร็จ: {e}"),
        })?;

    let status = child
        .wait()
        .map_err(|e| format!("รอผลการเชื่อมต่อ SSH ไม่สำเร็จ: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("SSH จบด้วยสถานะผิดปกติ: {status}"))
    }
}
