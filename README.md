# cbs-ssh

cbs-ssh เป็นเครื่องมือ CLI สำหรับเชื่อมต่อ SSH ไปยังเซิร์ฟเวอร์ต่างๆ จากไฟล์ server.json โดยรองรับการเลือกเซิร์ฟเวอร์ผ่าน TUI (Ratatui) และคำสั่งตามชื่อเซิร์ฟเวอร์ได้ทันที

## คุณสมบัติ

- อ่านเซิร์ฟเวอร์จากไฟล์ server.json
- เลือกเซิร์ฟเวอร์ผ่านหน้าจอ Ratatui
- รองรับคำสั่งเชื่อมต่อแบบพารามิเตอร์
- รองรับคำสั่ง `show` เพื่อแสดงเนื้อหาไฟล์ server.json
- รองรับการตั้งค่า `CBS_SSH_SERVER_JSON` เพื่อระบุ path ของไฟล์ server.json

## ข้อกำหนดเบื้องต้น

- Rust toolchain
- SSH client ที่ถูกติดตั้งอยู่ในระบบ
- `sshpass` สำหรับการส่งรหัสผ่านแบบไม่โต้ตอบ

## ติดตั้ง Rust

ถ้ายังไม่มี Rust ติดตั้ง ให้ติดตั้งจาก:

https://www.rust-lang.org/tools/install

## ติดตั้ง sshpass

โปรเจกต์นี้เหมาะกับ Linux/WSL มากที่สุด เพราะใช้ `sshpass` เพื่อส่งรหัสผ่านให้ ssh โดยอัตโนมัติ

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

### macOS

macOS อาจใช้ได้ แต่ไม่แน่ใจว่าเหมาะสมหรือจะทำงานครบถ้วนกับโปรเจกต์นี้ เพราะ `sshpass` และพฤติกรรมของ SSH บน macOS อาจต่างจาก Linux/WSL เล็กน้อย

ถ้าต้องการลองบน macOS ให้ลอง:

```bash
brew install hudochenkov/sshpass/sshpass
```

แต่ถ้าไม่สำเร็จหรือมีปัญหาเกี่ยวกับ PATH หรือ SSH authentication ควรใช้ Linux/WSL เป็นตัวเลือกหลัก

## ติดตั้งโปรเจกต์

```bash
cargo build
```

## การใช้งาน

### เปิดเมนูเลือกเซิร์ฟเวอร์

```bash
cargo run
```

### แสดงข้อมูลใน server.json

```bash
cargo run -- show
```

### เชื่อมต่อด้วยชื่อเซิร์ฟเวอร์จากไฟล์

```bash
cargo run -- SERVER_1
```

### เชื่อมต่อด้วยค่าตรง ๆ

```bash
cargo run -- ubuntu 127.0.0.1 password111
```

หรือกำหนดพอร์ตเอง

```bash
cargo run -- ubuntu 127.0.0.1 2221 password111
```

## โครงสร้างไฟล์ server.json

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

## ตั้งค่า environment variable

หากต้องการระบุ path ของไฟล์ server.json โดยไม่ต้องวางไฟล์ไว้ในโฟลเดอร์ปัจจุบัน ให้ตั้งตัวแปร `CBS_SSH_SERVER_JSON`

### PowerShell

```powershell
$env:CBS_SSH_SERVER_JSON = "D:\saas-project\cbs-ssh\server.json"
```

### WSL / Linux / zsh

```bash
export CBS_SSH_SERVER_JSON="$HOME/.config/cbs-ssh/server.json"
```

ถ้าต้องการให้ตัวแปรอยู่ตลอด session หรือเปิด terminal ใหม่ ให้เพิ่มลงใน profile ของ shell ที่ใช้

## หมายเหตุ

- ถ้าโปรแกรมแจ้งว่าไม่พบ `sshpass` ให้ติดตั้งก่อนใช้งาน
- ถ้าใช้ WSL ให้ตรวจว่า `sshpass` ติดตั้งใน WSL environment ที่คุณรันโปรแกรมอยู่ด้วย
- ถ้าใช้ PowerShell บน Windows และเรียกโปรแกรมจาก WSL อาจต้องดูว่า environment ที่รันอยู่เป็น Windows หรือ WSL ก่อน