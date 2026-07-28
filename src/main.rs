mod ratatui;
mod server_config;

use std::{
    env, fs,
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use server_config::ServerConfig;

const SERVER_JSON_ENV: &str = "CBS_SSH_SERVER_JSON";

fn connect_ssh_with_password(
    user: &str,
    host: &str,
    port: u16,
    password: &str,
    encode: &str,
) -> Result<(), String> {
    let connection_string = format!("{}@{}", user, host);

    /*
    อยากใส่ตัวแปรหรือเงื่อนไขเพิ่มเติมสามารถทำได้เลย
     */
    let mut child = Command::new("sshpass")
        .env("LC_ALL", "C")
        .env("LANG", encode)
        .arg("-p")
        .arg(password)
        .arg("ssh")
        .arg("-t")
        .arg("-p")
        .arg(port.to_string())
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
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
            _ => format!("เริ่มคำสั่ง sshpass ไม่สำเร็จ: {}", e),
        })?;

    let status = child
        .wait()
        .map_err(|e| format!("รอผลการเชื่อมต่อ SSH ไม่สำเร็จ: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("SSH จบด้วยสถานะผิดปกติ: {}", status))
    }
}

fn load_servers(path: &Path) -> Result<Vec<ServerConfig>, String> {
    let data = fs::read_to_string(path)
        .map_err(|e| format!("ไม่สามารถอ่าน JSON ได้ {}: {}", path.display(), e))?;
    serde_json::from_str::<Vec<ServerConfig>>(&data)
        .map_err(|e| format!("ไม่สามารถอ่าน JSON ได้ {}: {}", path.display(), e))
}

fn resolve_server_json_path() -> Result<PathBuf, String> {
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

    let mut checked_paths: Vec<PathBuf> = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        let path = cwd.join("server.json");
        if path.is_file() {
            return Ok(path);
        }
        checked_paths.push(path);
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let path = exe_dir.join("server.json");
            if path.is_file() {
                return Ok(path);
            }
            checked_paths.push(path);
        }
    }

    let checked = checked_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Err(format!(
        "ไม่พบ server.json (ค้นหาใน: {}). ตั้งค่า {} เพื่อระบุ path เอง",
        checked, SERVER_JSON_ENV
    ))
}

fn usage(bin: &str) {
    eprintln!("การใช้งาน:");
    eprintln!("  {}                             # open interactive server picker", bin);
    eprintln!("  {} แสดงรายการ Server.json      # print server.json and exit", bin);
    eprintln!("  {} <server_name>              # connect using server.json by name", bin);
    eprintln!("  {} <user> <host> <password>", bin);
    eprintln!("  {} <user> <host> <port> <password>", bin);
    eprintln!();
    eprintln!("ลำดับการตั้งค่า:");
    eprintln!("  1) ${} (absolute/relative path to server.json)", SERVER_JSON_ENV);
    eprintln!("  2) ./server.json (current working directory)");
    eprintln!("  3) server.json next to executable");
}

fn show_server_file(path: &Path) {
    println!("\n===== {} =====", path.display());
    match fs::read_to_string(path) {
        Ok(content) => println!("{}", content),
        Err(err) => eprintln!("อ่านไฟล์ไม่สำเร็จ: {}", err),
    }
    println!("========================\n");
}

fn wait_for_enter() {
    print!("กด Enter เพื่อกลับไปหน้าเลือก... ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bin = &args[0];

    match args.len() {
        1 => {
            let server_json = match resolve_server_json_path() {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };

            let servers = match load_servers(&server_json) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };

            let labels: Vec<String> = servers
                .iter()
                .map(|s| format!("{}  ({}@{}:{})", s.name, s.user, s.host, s.port))
                .collect();

            loop {
                let selected = match ratatui::select_server(&labels) {
                    Ok(v) => v,
                    Err(err) => {
                        eprintln!("ไม่สามารถเปิดตัวเลือกเซิร์ฟเวอร์ได้: {}", err);
                        std::process::exit(1);
                    }
                };

                match selected {
                    ratatui::MenuAction::Connect(index) => {
                        if let Some(server) = servers.get(index) {
                            connect_ssh_with_password(
                                &server.user,
                                &server.host,
                                server.port,
                                &server.password,
                                &server.encode,
                            )
                            .unwrap_or_else(|err| {
                                eprintln!("{}", err);
                                std::process::exit(1);
                            });
                            break;
                        }

                        eprintln!("เลือกเซิร์ฟเวอร์ไม่ถูกต้อง: {}", index);
                        std::process::exit(1);
                    }
                    ratatui::MenuAction::ShowServer => {
                        show_server_file(&server_json);
                        wait_for_enter();
                    }
                    ratatui::MenuAction::Cancel => {
                        eprintln!("ออกจากเมนูแล้ว");
                        std::process::exit(1);
                    }
                }
            }
        }
        2 => {
            let server_name = &args[1];
            let server_json = match resolve_server_json_path() {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };

            if server_name.eq_ignore_ascii_case("show") {
                show_server_file(&server_json);
                return;
            }

            let servers = match load_servers(&server_json) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };

            if let Some(server) = servers.iter().find(|s| &s.name == server_name) {
                connect_ssh_with_password(
                    &server.user,
                    &server.host,
                    server.port,
                    &server.password,
                    &server.encode,
                )
                .unwrap_or_else(|err| {
                    eprintln!("{}", err);
                    std::process::exit(1);
                });
            } else {
                eprintln!("Server '{}' not found in server.json", server_name);
                std::process::exit(1);
            }
        }
        _ => {
            usage(bin);
            std::process::exit(1);
        }
    }
}
