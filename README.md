# cbs-ssh

cbs-ssh เป็นเครื่องมือ CLI สำหรับเชื่อมต่อ SSH ไปยังเซิร์ฟเวอร์ต่าง ๆ จากไฟล์ `server.json` โดยรองรับการเลือกเซิร์ฟเวอร์ผ่าน TUI (Ratatui) และคำสั่งตามชื่อเซิร์ฟเวอร์ได้ทันที

## คุณสมบัติ

- อ่านเซิร์ฟเวอร์จากไฟล์ `server.json`
- เลือกเซิร์ฟเวอร์ผ่านหน้าจอ Ratatui
- รองรับคำสั่งเชื่อมต่อแบบพารามิเตอร์
- รองรับคำสั่ง `show` เพื่อแสดงเนื้อหาไฟล์ `server.json`

## ข้อกำหนดเบื้องต้น

- Rust toolchain
- SSH client ที่ถูกติดตั้งอยู่ในระบบ
- `sshpass` สำหรับการส่งรหัสผ่านแบบไม่โต้ตอบ

## ติดตั้ง `sshpass`

### Ubuntu / Debian / WSL Ubuntu

```bash
sudo apt update
sudo apt install -y sshpass
```

### Fedora / RHEL

```bash
sudo dnf install -y sshpass
```

### Arch Linux

```bash
sudo pacman -S sshpass
```

> หมายเหตุ: macOS อาจใช้ได้แต่ไม่แน่ใจว่าเหมาะสมหรือจะทำงานครบถ้วนกับโปรเจกต์นี้ เพราะพฤติกรรมของ SSH และ `sshpass` บน macOS อาจต่างจาก Linux/WSL เล็กน้อย

## Build โปรเจกต์

```bash
cargo build --release
```

## วิธีใช้งาน

### 1) เปิดเมนูเลือกเซิร์ฟเวอร์

```bash
cargo run
```

### 2) แสดงข้อมูลใน `server.json`

```bash
cargo run -- show
```

### 3) เชื่อมต่อด้วยชื่อเซิร์ฟเวอร์จากไฟล์

```bash
cargo run -- SERVER_1
```

### 4) เชื่อมต่อด้วยค่าตรง ๆ

```bash
cargo run -- ubuntu 127.0.0.1 password111
```

หรือกำหนดพอร์ตเอง:

```bash
cargo run -- ubuntu 127.0.0.1 2221 password111
```

## โครงสร้างไฟล์ `server.json`

ตัวอย่าง:

```json
[
  {
    "name": "SERVER_1",
    "user": "ubuntu",
    "host": "127.0.0.1",
    "port": 2221,
    "password": "password111",
    "encode": "UTF-8"
  }
]
```

## ตั้งค่า Environment Variable

หากต้องการระบุ path ของไฟล์ `server.json` โดยไม่ต้องวางไฟล์ไว้ในโฟลเดอร์ปัจจุบัน ให้ตั้งตัวแปร `CBS_SSH_SERVER_JSON`

### PowerShell

```powershell
$env:CBS_SSH_SERVER_JSON = "D:\saas-project\cbs-ssh\server.json"
```

### WSL / Linux / zsh

```bash
export CBS_SSH_SERVER_JSON="$HOME/.config/cbs-ssh/server.json"
```

## ติดตั้งบน WSL / Linux

1. Build executable file

```bash
cargo build --release
```

2. คัดลอกไฟล์ executable และ `server.json` ไปไว้ในโฟลเดอร์เดียวกัน

3. เพิ่มข้อมูลใน `~/.zshrc` หรือ `~/.bashrc`

```bash
export PATH="/path/to/your/bin:$PATH"
export CBS_SSH_SERVER_JSON="/path/to/your/bin/server.json"
```

4. โหลดไฟล์ config ใหม่

```bash
source ~/.zshrc
```

หรือ

```bash
source ~/.bashrc
```



