use crate::server_config::{DEFAULT_ENCODE, SERVER_JSON_ENV};
use crate::ssh::SshTarget;

/// พอร์ต SSH เริ่มต้นเมื่อผู้ใช้ไม่ระบุ
pub const DEFAULT_SSH_PORT: u16 = 22;

const SHOW_COMMAND: &str = "show";

/// คำสั่งที่ผู้ใช้เรียกผ่าน command line
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// เปิดเมนูเลือกเซิร์ฟเวอร์แบบ interactive
    Interactive,
    /// แสดงเนื้อหา server.json แล้วจบการทำงาน
    ShowServers,
    /// เชื่อมต่อโดยอ้างชื่อเซิร์ฟเวอร์ใน server.json
    ConnectByName(String),
    /// เชื่อมต่อด้วยค่าที่ระบุตรง ๆ
    ConnectDirect(SshTarget),
}

/// แปลง argument (รวมชื่อ executable ที่ index 0) เป็น `Command`
///
/// คืน `Err` พร้อมข้อความอธิบายเมื่อรูปแบบคำสั่งไม่ถูกต้อง
pub fn parse(args: &[String]) -> Result<Command, String> {
    let Some((_bin, rest)) = args.split_first() else {
        return Err("จำนวน argument ไม่ถูกต้อง".to_string());
    };

    match rest {
        [] => Ok(Command::Interactive),
        [name] if name.eq_ignore_ascii_case(SHOW_COMMAND) => Ok(Command::ShowServers),
        [name] => Ok(Command::ConnectByName(name.clone())),
        [user, host, password] => Ok(Command::ConnectDirect(direct_target(
            user,
            host,
            DEFAULT_SSH_PORT,
            password,
        ))),
        [user, host, port, password] => Ok(Command::ConnectDirect(direct_target(
            user,
            host,
            parse_port(port)?,
            password,
        ))),
        _ => Err("จำนวน argument ไม่ถูกต้อง".to_string()),
    }
}

fn direct_target(user: &str, host: &str, port: u16, password: &str) -> SshTarget {
    SshTarget::new(user, host, port, password, DEFAULT_ENCODE)
}

fn parse_port(value: &str) -> Result<u16, String> {
    match value.parse::<u16>() {
        Ok(port) if port > 0 => Ok(port),
        _ => Err(format!("พอร์ตไม่ถูกต้อง: {value} (ต้องเป็นตัวเลข 1-65535)")),
    }
}

/// แสดงวิธีใช้งาน
pub fn usage(bin: &str) {
    eprintln!("การใช้งาน:");
    eprintln!("  {bin}                              # เปิดเมนูเลือกเซิร์ฟเวอร์");
    eprintln!("  {bin} show                         # แสดงเนื้อหา server.json และจบการทำงาน");
    eprintln!("  {bin} <server_name>                # เชื่อมต่อตามชื่อใน server.json");
    eprintln!("  {bin} <user> <host> <password>     # เชื่อมต่อตรง (พอร์ตเริ่มต้น {DEFAULT_SSH_PORT})");
    eprintln!("  {bin} <user> <host> <port> <password>");
    eprintln!();
    eprintln!("ลำดับการค้นหา server.json:");
    eprintln!("  1) ${SERVER_JSON_ENV} (path ที่กำหนด)");
    eprintln!("  2) ./server.json (โฟลเดอร์ปัจจุบัน)");
    eprintln!("  3) server.json ข้างไฟล์ executable");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn no_args_starts_interactive() {
        assert_eq!(parse(&args(&["cbs-ssh"])).unwrap(), Command::Interactive);
    }

    #[test]
    fn show_command_is_case_insensitive() {
        assert_eq!(
            parse(&args(&["cbs-ssh", "SHOW"])).unwrap(),
            Command::ShowServers
        );
        assert_eq!(
            parse(&args(&["cbs-ssh", "show"])).unwrap(),
            Command::ShowServers
        );
    }

    #[test]
    fn single_arg_connects_by_name() {
        assert_eq!(
            parse(&args(&["cbs-ssh", "SERVER_1"])).unwrap(),
            Command::ConnectByName("SERVER_1".to_string())
        );
    }

    #[test]
    fn three_args_use_default_port_and_utf8() {
        assert_eq!(
            parse(&args(&["cbs-ssh", "ubuntu", "127.0.0.1", "pw"])).unwrap(),
            Command::ConnectDirect(SshTarget::new(
                "ubuntu",
                "127.0.0.1",
                DEFAULT_SSH_PORT,
                "pw",
                DEFAULT_ENCODE
            ))
        );
    }

    #[test]
    fn four_args_use_given_port() {
        let command = parse(&args(&["cbs-ssh", "ubuntu", "127.0.0.1", "2221", "pw"])).unwrap();
        let expected = Command::ConnectDirect(SshTarget::new(
            "ubuntu",
            "127.0.0.1",
            2221,
            "pw",
            DEFAULT_ENCODE,
        ));
        assert_eq!(command, expected);
    }

    #[test]
    fn invalid_port_is_rejected() {
        assert!(parse(&args(&["cbs-ssh", "ubuntu", "127.0.0.1", "abc", "pw"])).is_err());
        assert!(parse(&args(&["cbs-ssh", "ubuntu", "127.0.0.1", "0", "pw"])).is_err());
        assert!(parse(&args(&["cbs-ssh", "ubuntu", "127.0.0.1", "65536", "pw"])).is_err());
    }

    #[test]
    fn wrong_argument_count_is_rejected() {
        assert!(parse(&args(&["cbs-ssh", "a", "b"])).is_err());
        assert!(parse(&args(&["cbs-ssh", "a", "b", "c", "d", "e"])).is_err());
    }
}
