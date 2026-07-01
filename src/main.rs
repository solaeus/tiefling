use std::{error::Error, fs::read_dir, io};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event,
    layout::{Constraint, Layout},
    widgets::{List, ListState},
};

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(run)?;

    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut list_state = ListState::default();

    loop {
        terminal.draw(|frame| render(frame, &mut list_state))?;

        if event::read()?.is_key_press() {
            break;
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

    let list = List::new(files);

    frame.render_stateful_widget(list, area, list_state);
}
