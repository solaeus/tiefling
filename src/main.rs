use std::{error::Error, fs::read_dir, io};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{List, ListState},
};

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(run)?;

    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut list_state = ListState::default().with_selected(Some(0));

    loop {
        terminal.draw(|frame| render(frame, &mut list_state))?;

        match event::read()?.as_key_press_event() {
            Some(key_press) if key_press.modifiers.is_empty() => match key_press.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Up | KeyCode::Char('k') => list_state.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => list_state.select_next(),
                KeyCode::Left | KeyCode::Char('h') => {}
                KeyCode::Right | KeyCode::Char('l') => {}
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, list_state: &mut ListState) {
    let layout = Layout::vertical([Constraint::Fill(1)]).spacing(1);
    let [area] = frame.area().layout(&layout);

    let files = read_dir(".").map_or_else(
        |_| Vec::new(),
        |entries| {
            entries
                .map(|entry| {
                    entry.map_or_else(
                        |_| String::new(),
                        |entry| entry.file_name().to_string_lossy().to_string(),
                    )
                })
                .collect()
        },
    );

    let list = List::new(files).highlight_style(Style::default().bold());

    frame.render_stateful_widget(list, area, list_state);
}
