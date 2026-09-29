use std::io;

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph},
};

const SHOW_SERVER_LABEL: &str = "SHOW SERVER (cat server.json)";

pub enum MenuAction {
    Connect(usize),
    ShowServer,
    Cancel,
}

/// รายการที่แสดงในเมนู หลังผ่านการกรองแล้ว
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    /// รายการพิเศษสำหรับแสดงเนื้อหา server.json
    ShowServer,
    /// เซิร์ฟเวอร์ลำดับที่ `index` ในรายการต้นทาง (ก่อนกรอง)
    Server(usize),
}

/// สถานะของหน้าจอเลือกเซิร์ฟเวอร์
struct Picker {
    query: String,
    search_focused: bool,
    entries: Vec<Entry>,
    state: ListState,
}

impl Picker {
    fn new(items: &[String]) -> Self {
        let entries = filter_entries(items, "");
        let mut state = ListState::default();
        if !entries.is_empty() {
            state.select(Some(0));
        }

        Self {
            query: String::new(),
            search_focused: false,
            entries,
            state,
        }
    }

    /// กรองรายการใหม่ตามคำค้นหา และปรับ selection ให้ยังอยู่ในช่วง
    fn apply_filter(&mut self, items: &[String]) {
        self.entries = filter_entries(items, &self.query);

        if self.entries.is_empty() {
            self.state.select(None);
            return;
        }

        let last = self.entries.len() - 1;
        let current = self.state.selected().unwrap_or(0).min(last);
        self.state.select(Some(current));
    }

    /// เลื่อน selection แบบวนรอบ
    fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            self.state.select(None);
            return;
        }

        let len = self.entries.len() as isize;
        let current = self.state.selected().unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(len);
        self.state.select(Some(next as usize));
    }

    /// แปลงรายการที่เลือกเป็น `MenuAction`
    fn activate(&self) -> Option<MenuAction> {
        match self.entries.get(self.state.selected()?)? {
            Entry::ShowServer => Some(MenuAction::ShowServer),
            Entry::Server(index) => Some(MenuAction::Connect(*index)),
        }
    }

    /// จัดการปุ่มกด คืนค่า `Some(action)` เมื่อผู้ใช้ยืนยัน/ยกเลิก
    fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        items: &[String],
    ) -> Option<MenuAction> {
        if self.search_focused {
            return match code {
                // รับเฉพาะตัวอักษรปกติ ไม่รวม Ctrl/Alt (เช่น Ctrl+C)
                KeyCode::Char(c)
                    if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    self.query.push(c);
                    self.apply_filter(items);
                    None
                }
                KeyCode::Backspace => {
                    self.query.pop();
                    self.apply_filter(items);
                    None
                }
                KeyCode::Esc => {
                    self.query.clear();
                    self.search_focused = false;
                    self.apply_filter(items);
                    None
                }
                KeyCode::Up => {
                    self.move_selection(-1);
                    None
                }
                KeyCode::Down => {
                    self.move_selection(1);
                    None
                }
                KeyCode::Enter => self.activate(),
                _ => None,
            };
        }

        match code {
            KeyCode::Char('/') => {
                self.search_focused = true;
                None
            }
            KeyCode::Up => {
                self.move_selection(-1);
                None
            }
            KeyCode::Down => {
                self.move_selection(1);
                None
            }
            KeyCode::Enter => self.activate(),
            KeyCode::Esc | KeyCode::Char('q') => Some(MenuAction::Cancel),
            _ => None,
        }
    }
}

/// กรองรายการเซิร์ฟเวอร์ด้วยคำค้นหา (case-insensitive)
///
/// - คำค้นหาว่าง → แสดง `ShowServer` ตามด้วยเซิร์ฟเวอร์ทั้งหมด
/// - มีคำค้นหา → แสดงเฉพาะเซิร์ฟเวอร์ที่ตรง (ไม่รวม `ShowServer`)
fn filter_entries(items: &[String], query: &str) -> Vec<Entry> {
    let needle = query.trim().to_lowercase();

    if needle.is_empty() {
        let mut entries = Vec::with_capacity(items.len() + 1);
        entries.push(Entry::ShowServer);
        entries.extend((0..items.len()).map(Entry::Server));
        return entries;
    }

    items
        .iter()
        .enumerate()
        .filter(|(_, label)| label.to_lowercase().contains(&needle))
        .map(|(index, _)| Entry::Server(index))
        .collect()
}

/// เปิดเมนูเลือกเซิร์ฟเวอร์แบบ interactive พร้อมช่องค้นหา
pub fn select_server(items: &[String]) -> io::Result<MenuAction> {
    // `TerminalSession` จะคืนค่า terminal อัตโนมัติเมื่อออกจาก scope (รวมกรณี error)
    let mut session = TerminalSession::start()?;
    let mut picker = Picker::new(items);

    loop {
        session
            .terminal
            .draw(|frame| draw(frame, &mut picker, items))?;

        if let Event::Key(key) = event::read()? {
            // บนบางแพลตฟอร์ม (เช่น Windows) มีทั้ง event ตอนกดและตอนปล่อยปุ่ม
            // จึงจัดการเฉพาะตอนกด เพื่อไม่ให้คำสั่งถูกทำซ้ำ
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if let Some(action) = picker.handle_key(key.code, key.modifiers, items) {
                return Ok(action);
            }
        }
    }
}

fn draw(frame: &mut Frame, picker: &mut Picker, items: &[String]) {
    let [search_area, list_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(frame.area());

    draw_search(frame, search_area, picker);
    draw_list(frame, list_area, picker, items);
}

fn draw_search(frame: &mut Frame, area: Rect, picker: &Picker) {
    let border_style = if picker.search_focused {
        Style::default().fg(Color::LightGreen)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if picker.search_focused {
        " ค้นหา (Esc ล้าง/ออก, Enter เลือก) "
    } else {
        " ค้นหา (กด / เพื่อค้นหา) "
    };

    let content = if picker.query.is_empty() {
        Text::styled(
            "พิมพ์ชื่อ / user / host / port ที่ต้องการค้นหา",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Text::raw(picker.query.as_str())
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    frame.render_widget(Paragraph::new(content).block(block), area);

    if picker.search_focused {
        let width = picker.query.chars().count() as u16;
        let max_x = area.right().saturating_sub(2);
        let x = (area.x + 1).saturating_add(width).min(max_x);
        frame.set_cursor_position(Position::new(x, area.y + 1));
    }
}

fn draw_list(frame: &mut Frame, area: Rect, picker: &mut Picker, items: &[String]) {
    let count = picker.entries.len();
    let title = if picker.query.is_empty() {
        format!(" เลือกเซิร์ฟเวอร์ ({count} รายการ) — ↑/↓ Enter, / ค้นหา, q/Esc ออก ")
    } else {
        format!(" ผลลัพธ์ค้นหา ({count} รายการ) — Esc ล้าง, Enter เลือก ")
    };

    let list_items: Vec<ListItem> = picker
        .entries
        .iter()
        .map(|entry| match entry {
            Entry::ShowServer => ListItem::new(SHOW_SERVER_LABEL),
            Entry::Server(index) => ListItem::new(items[*index].as_str()),
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    frame.render_stateful_widget(list, area, &mut picker.state);

    if count == 0 {
        let inner = area.inner(Margin::new(2, 1));
        if inner.width > 0 && inner.height > 0 {
            let hint = Paragraph::new(format!("ไม่พบเซิร์ฟเวอร์ที่ตรงกับ \"{}\"", picker.query.trim()))
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(hint, inner);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Vec<String> {
        vec![
            "SERVER_1  (ubuntu@127.0.0.1:2221)".to_string(),
            "SERVER_2  (ubuntu@127.0.0.1:2222)".to_string(),
            "PROD_DB  (root@10.0.0.9:22)".to_string(),
        ]
    }

    #[test]
    fn empty_query_shows_show_server_then_all_servers() {
        assert_eq!(
            filter_entries(&labels(), ""),
            vec![
                Entry::ShowServer,
                Entry::Server(0),
                Entry::Server(1),
                Entry::Server(2)
            ]
        );
    }

    #[test]
    fn query_matches_case_insensitively() {
        assert_eq!(filter_entries(&labels(), "prod"), vec![Entry::Server(2)]);
        assert_eq!(filter_entries(&labels(), "PrOd"), vec![Entry::Server(2)]);
    }

    #[test]
    fn query_matches_host_user_and_port() {
        assert_eq!(
            filter_entries(&labels(), "10.0.0.9"),
            vec![Entry::Server(2)]
        );
        assert_eq!(
            filter_entries(&labels(), "ubuntu"),
            vec![Entry::Server(0), Entry::Server(1)]
        );
        assert_eq!(filter_entries(&labels(), "2222"), vec![Entry::Server(1)]);
    }

    #[test]
    fn query_trims_spaces_and_ignores_show_server_entry() {
        assert_eq!(
            filter_entries(&labels(), "  server_1  "),
            vec![Entry::Server(0)]
        );
        assert!(filter_entries(&labels(), "show").is_empty());
    }

    #[test]
    fn no_match_returns_empty_list() {
        assert!(filter_entries(&labels(), "zzz-not-found").is_empty());
    }

    #[test]
    fn picker_clamps_selection_after_filtering() {
        let items = labels();
        let mut picker = Picker::new(&items);
        picker.move_selection(2);
        assert_eq!(picker.state.selected(), Some(2));

        picker.query = "prod".to_string();
        picker.apply_filter(&items);
        assert_eq!(picker.entries, vec![Entry::Server(2)]);
        assert_eq!(picker.state.selected(), Some(0));
    }

    #[test]
    fn picker_selection_is_none_when_no_match() {
        let items = labels();
        let mut picker = Picker::new(&items);
        picker.query = "zzz".to_string();
        picker.apply_filter(&items);

        assert!(picker.entries.is_empty());
        assert_eq!(picker.state.selected(), None);
        assert!(picker.activate().is_none());
    }

    #[test]
    fn picker_maps_filtered_selection_back_to_original_index() {
        let items = labels();
        let mut picker = Picker::new(&items);
        picker.query = "prod".to_string();
        picker.apply_filter(&items);

        assert!(matches!(picker.activate(), Some(MenuAction::Connect(2))));
    }

    #[test]
    fn picker_activates_show_server_when_query_is_empty() {
        let items = labels();
        let picker = Picker::new(&items);

        assert!(matches!(picker.activate(), Some(MenuAction::ShowServer)));
    }

    #[test]
    fn slash_focuses_search_and_typing_filters() {
        let items = labels();
        let mut picker = Picker::new(&items);

        assert!(
            picker
                .handle_key(KeyCode::Char('/'), KeyModifiers::NONE, &items)
                .is_none()
        );
        assert!(picker.search_focused);

        for ch in ['p', 'r', 'o', 'd'] {
            assert!(
                picker
                    .handle_key(KeyCode::Char(ch), KeyModifiers::NONE, &items)
                    .is_none()
            );
        }

        assert_eq!(picker.query, "prod");
        assert_eq!(picker.entries, vec![Entry::Server(2)]);
    }

    #[test]
    fn escape_in_search_clears_query_and_unfocuses() {
        let items = labels();
        let mut picker = Picker::new(&items);
        picker.handle_key(KeyCode::Char('/'), KeyModifiers::NONE, &items);
        picker.handle_key(KeyCode::Char('p'), KeyModifiers::NONE, &items);

        assert!(
            picker
                .handle_key(KeyCode::Esc, KeyModifiers::NONE, &items)
                .is_none()
        );
        assert!(!picker.search_focused);
        assert!(picker.query.is_empty());
        assert_eq!(picker.entries.len(), items.len() + 1);
    }

    #[test]
    fn q_cancels_only_when_search_is_not_focused() {
        let items = labels();
        let mut picker = Picker::new(&items);

        assert!(matches!(
            picker.handle_key(KeyCode::Char('q'), KeyModifiers::NONE, &items),
            Some(MenuAction::Cancel)
        ));

        picker.handle_key(KeyCode::Char('/'), KeyModifiers::NONE, &items);
        assert!(
            picker
                .handle_key(KeyCode::Char('q'), KeyModifiers::NONE, &items)
                .is_none()
        );
        assert_eq!(picker.query, "q");
    }

    #[test]
    fn control_modified_characters_are_ignored() {
        let items = labels();
        let mut picker = Picker::new(&items);
        picker.handle_key(KeyCode::Char('/'), KeyModifiers::NONE, &items);

        assert!(
            picker
                .handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL, &items)
                .is_none()
        );
        assert!(picker.query.is_empty());
    }
}
