use std::io;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState},
};

/// ดัชนีของรายการ "SHOW SERVER" ซึ่งอยู่บนสุดเสมอ
const SHOW_SERVER_INDEX: usize = 0;
const SHOW_SERVER_LABEL: &str = "SHOW SERVER (cat server.json)";

pub enum MenuAction {
    Connect(usize),
    ShowServer,
    Cancel,
}

/// เปิดเมนูเลือกเซิร์ฟเวอร์แบบ interactive
pub fn select_server(items: &[String]) -> io::Result<MenuAction> {
    let menu_items = build_menu_items(items);

    // `TerminalSession` จะคืนค่า terminal อัตโนมัติเมื่อออกจาก scope (รวมกรณี error)
    let mut session = TerminalSession::start()?;

    let mut state = ListState::default();
    state.select(Some(SHOW_SERVER_INDEX));

    loop {
        session.terminal.draw(|frame| {
            frame.render_stateful_widget(build_list(&menu_items), frame.area(), &mut state);
        })?;

        if let Event::Key(key) = event::read()? {
            // บนบางแพลตฟอร์ม (เช่น Windows) มีทั้ง event ตอนกดและตอนปล่อยปุ่ม
            // จึงจัดการเฉพาะตอนกด เพื่อไม่ให้คำสั่งถูกทำซ้ำ
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if let Some(action) = handle_key(key.code, &mut state, menu_items.len()) {
                return Ok(action);
            }
        }
    }
}

fn build_menu_items(items: &[String]) -> Vec<String> {
    let mut menu_items = Vec::with_capacity(items.len() + 1);
    menu_items.push(SHOW_SERVER_LABEL.to_string());
    menu_items.extend_from_slice(items);
    menu_items
}

fn build_list(items: &[String]) -> List<'_> {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(item.as_str()))
        .collect();

    List::new(list_items)
        .block(
            Block::default()
                .title(" เลือกเมนู (ลูกศร ขึ้น/ลง + Enter, Esc เพื่อออก) ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom)
}

/// จัดการปุ่มกด คืนค่า `Some(action)` เมื่อผู้ใช้ยืนยัน/ยกเลิก
fn handle_key(code: KeyCode, state: &mut ListState, item_count: usize) -> Option<MenuAction> {
    match code {
        KeyCode::Up => {
            if let Some(current) = state.selected() {
                let next = if current == 0 {
                    item_count - 1
                } else {
                    current - 1
                };
                state.select(Some(next));
            }
            None
        }
        KeyCode::Down => {
            if let Some(current) = state.selected() {
                let next = if current + 1 >= item_count {
                    0
                } else {
                    current + 1
                };
                state.select(Some(next));
            }
            None
        }
        KeyCode::Enter => {
            let selected = state.selected().unwrap_or(SHOW_SERVER_INDEX);
            if selected == SHOW_SERVER_INDEX {
                Some(MenuAction::ShowServer)
            } else {
                Some(MenuAction::Connect(selected - 1))
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => Some(MenuAction::Cancel),
        _ => None,
    }
}

/// เป็นเจ้าของ terminal ระหว่างใช้งาน TUI และคืนค่าให้อัตโนมัติเมื่อถูก drop
struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn start() -> io::Result<Self> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        if let Err(err) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err);
        }

        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(err) => {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                Err(err)
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
