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

ปุ่มที่ใช้ในเมนู:

| ปุ่ม | การทำงาน |
| ---- | -------- |
| `↑` / `↓` | เลื่อนรายการ (วนรอบ) |
| `/` | เปิดช่องค้นหา (กรองจากชื่อ / user / host / port) |
| `Enter` | เชื่อมต่อเซิร์ฟเวอร์ที่เลือก |
| `Backspace` | ลบตัวอักษรในช่องค้นหา |
| `Esc` | ขณะค้นหา = ล้างคำค้นหาและออกจากช่อง, ปกติ = ออกจากเมนู |
| `q` | ออกจากเมนู (ใช้ได้เมื่อไม่ได้อยู่ในช่องค้นหา) |

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

> หมายเหตุ: หากไม่ระบุพอร์ต จะใช้พอร์ตเริ่มต้น `22` และใช้ encoding เริ่มต้น `UTF-8`

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

> `encode` เป็นฟิลด์ไม่บังคับ หากไม่ระบุจะใช้ค่าเริ่มต้น `UTF-8`

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

## แก้ปัญหา SSH host key changed

ถ้าเจอข้อความแบบนี้:

```
WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!
Host key for [127.0.0.1]:2221 has changed and you have requested strict checking.
Host key verification failed.
```

สาเหตุคือ host key ของเซิร์ฟเวอร์เปลี่ยนไป (พบบ่อยกับ container ที่ถูกสร้างใหม่) โปรแกรมใช้ `StrictHostKeyChecking=accept-new` จึงยอมรับเฉพาะ host **ใหม่** แต่จะปฏิเสธเมื่อ key เดิมเปลี่ยน

แก้โดยลบ key เก่าออกจาก `known_hosts`:

```bash
ssh-keygen -f "$HOME/.ssh/known_hosts" -R '[127.0.0.1]:2221'
```

ทำซ้ำกับทุกพอร์ตที่ใช้งาน (2222, 2223)

วิธีป้องกันไม่ให้เกิดซ้ำ: ให้ container เก็บ host key ถาวรด้วย volume ที่ path `/config` ซึ่งกำหนดไว้แล้วใน `docker-compose.yml` (`./data/server_*:/config`) เมื่อสร้าง container ใหม่ host key จะคงเดิม

## ความปลอดภัย

- รหัสผ่านถูกส่งให้ `sshpass` ผ่านตัวแปรสภาพแวดล้อม `SSHPASS` (ใช้ `sshpass -e`) จึงไม่ปรากฏใน process list
- โปรแกรมตั้งค่า `StrictHostKeyChecking=accept-new` คือยอมรับ host key ใหม่ในครั้งแรก แต่จะปฏิเสธหาก key เดิมเปลี่ยนภายหลัง
- ไฟล์ `server.json` เก็บรหัสผ่านแบบ plain text ควรระวังการแชร์หรือ commit ไฟล์นี้ขึ้น git

## การพัฒนา

```bash
cargo fmt --all -- --check   # ตรวจรูปแบบโค้ด
cargo clippy --all-targets   # ตรวจ lint
cargo test                   # รันชุดทดสอบ
```



