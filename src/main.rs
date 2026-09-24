mod cli;
mod ratatui;
mod server_config;
mod ssh;

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cli::Command;
use ratatui::MenuAction;
use server_config::{ServerConfig, load_servers, resolve_server_json_path};
use ssh::connect;

const DEFAULT_BIN: &str = "cbs-ssh";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let bin = args.first().map(String::as_str).unwrap_or(DEFAULT_BIN);

    let command = match cli::parse(&args) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            cli::usage(bin);
            return ExitCode::FAILURE;
        }
    };

    match run(command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Interactive => run_interactive(),
        Command::ShowServers => run_show(),
        Command::ConnectByName(server_name) => run_named(&server_name),
        Command::ConnectDirect(target) => connect(&target),
    }
}

/// โหลด path และรายการเซิร์ฟเวอร์จาก `server.json` พร้อมกัน
fn load_server_configs() -> Result<(PathBuf, Vec<ServerConfig>), String> {
    let path = resolve_server_json_path()?;
    let servers = load_servers(&path)?;
    Ok((path, servers))
}

/// เปิดเมนูเลือกเซิร์ฟเวอร์แบบ interactive
fn run_interactive() -> Result<(), String> {
    let (server_json, servers) = load_server_configs()?;
    let labels: Vec<String> = servers.iter().map(ServerConfig::label).collect();

    loop {
        let action = ratatui::select_server(&labels)
            .map_err(|e| format!("ไม่สามารถเปิดตัวเลือกเซิร์ฟเวอร์ได้: {e}"))?;

        match action {
            MenuAction::Connect(index) => {
                let server = servers
                    .get(index)
                    .ok_or_else(|| format!("เลือกเซิร์ฟเวอร์ไม่ถูกต้อง: {index}"))?;
                return connect(&server.to_target());
            }
            MenuAction::ShowServer => {
                show_server_file(&server_json)?;
                wait_for_enter();
            }
            // ผู้ใช้กดยกเลิก ไม่ถือว่าเป็น error จึงคืนค่าสำเร็จ
            MenuAction::Cancel => {
                eprintln!("ออกจากเมนูแล้ว");
                return Ok(());
            }
        }
    }
}

/// แสดงเนื้อหาไฟล์ `server.json` แล้วจบการทำงาน
fn run_show() -> Result<(), String> {
    let server_json = resolve_server_json_path()?;
    show_server_file(&server_json)
}

/// เชื่อมต่อโดยค้นหาเซิร์ฟเวอร์จากชื่อใน `server.json`
fn run_named(server_name: &str) -> Result<(), String> {
    let (server_json, servers) = load_server_configs()?;

    let server = servers
        .iter()
        .find(|server| server.name == server_name)
        .ok_or_else(|| format!("ไม่พบเซิร์ฟเวอร์ '{server_name}' ใน {}", server_json.display()))?;

    connect(&server.to_target())
}

fn show_server_file(path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("อ่านไฟล์ {} ไม่สำเร็จ: {err}", path.display()))?;

    println!("\n===== {} =====", path.display());
    println!("{content}");
    println!("========================\n");

    Ok(())
}

fn wait_for_enter() {
    print!("กด Enter เพื่อกลับไปหน้าเลือก... ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}
