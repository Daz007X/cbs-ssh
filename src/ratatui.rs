use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use std::io;

pub enum MenuAction {
    Connect(usize),
    ShowServer,
    Cancel,
}

pub fn select_server(items: &[String]) -> Result<MenuAction, io::Error> {
    let mut menu_items: Vec<String> = vec!["SHOW SERVER (cat server.json)".to_string()];
    menu_items.extend(items.iter().cloned());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = ListState::default();
    state.select(Some(0));

    let result = loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let list_items: Vec<ListItem> = menu_items
                .iter()
                .map(|item| ListItem::new(item.as_str()))
                .collect();

            let list = List::new(list_items)
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
                .direction(ratatui::widgets::ListDirection::TopToBottom);

            frame.render_stateful_widget(list, area, &mut state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if let Some(i) = state.selected() {
                        let new_i = if i == 0 { menu_items.len() - 1 } else { i - 1 };
                        state.select(Some(new_i));
                    }
                }
                KeyCode::Down => {
                    if let Some(i) = state.selected() {
                        let new_i = if i + 1 >= menu_items.len() { 0 } else { i + 1 };
                        state.select(Some(new_i));
                    }
                }
                KeyCode::Enter => {
                    let selected = state.selected().unwrap_or(0);
                    if selected == 0 {
                        break Ok(MenuAction::ShowServer);
                    }
                    break Ok(MenuAction::Connect(selected - 1));
                }
                KeyCode::Esc | KeyCode::Char('q') => break Ok(MenuAction::Cancel),
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
