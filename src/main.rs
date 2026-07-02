use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display, Pointer},
    fs::{DirEntry, FileType, ReadDir, read_dir},
    io,
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout},
    prelude::*,
    style::Style,
    text::Line,
    widgets::{ListState, Widget},
};

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(run)?;

    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app_state = AppState::default();

    loop {
        terminal.draw(|frame| render(frame, &mut app_state))?;

        match event::read()?.as_key_press_event() {
            Some(key_press) if key_press.modifiers.is_empty() => match key_press.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Up | KeyCode::Char('k') => app_state.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => app_state.select_next(),
                KeyCode::Left | KeyCode::Char('h') => {}
                KeyCode::Right | KeyCode::Char('l') => {}
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let layout = Layout::vertical([Constraint::Fill(1)]).spacing(1);
    let [area] = frame.area().layout(&layout);
    let pwd_files = read_dir(".").expect("Failed to read $PWD");
    let (pwd, pwd_length) = OpenDir::new(pwd_files);
    let app = App::new(pwd);

    app_state.set_length(pwd_length);
    frame.render_stateful_widget(app, area, app_state);
}

struct App {
    root: OpenDir,
}

impl App {
    fn new(root: OpenDir) -> Self {
        Self { root }
    }
}

impl StatefulWidget for App {
    type State = AppState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        self.root.render(area, buffer, state);
    }
}

#[derive(Default)]
struct AppState {
    length: usize,
    selected: Option<usize>,
}

impl AppState {
    fn set_length(&mut self, length: usize) {
        self.length = length;
    }

    fn select_next(&mut self) {
        match self.selected {
            None => self.selected = Some(0),
            Some(index) if index == self.length - 1 => self.selected = Some(0),
            Some(index) => {
                let next = (index + 1).min(self.length - 1);

                self.selected = Some(next);
            }
        }
    }

    fn select_previous(&mut self) {
        match self.selected {
            None | Some(0) => self.selected = Some(self.length),
            Some(index) => self.selected = Some(index - 1),
        }
    }
}

struct OpenDir {
    files: Vec<String>,
}

impl OpenDir {
    fn new(dir: ReadDir) -> (Self, usize) {
        let mut files = Vec::new();
        let mut count = 0;

        for entry in dir {
            let listing = match entry {
                Ok(entry) => entry.file_name().to_string_lossy().to_string(),
                Err(_) => "!tf_error!".to_string(),
            };

            files.push(listing);
            count += 1;
        }

        (Self { files }, count)
    }
}

impl StatefulWidget for OpenDir {
    type State = AppState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let constraints = vec![Constraint::Length(1); self.files.len()];
        let areas = area.layout_vec(&Layout::vertical(constraints));

        for (index, (file, area)) in self.files.into_iter().zip(areas).enumerate() {
            let mut line = Line::raw(file);

            if let AppState {
                selected: Some(selected),
                ..
            } = state
                && *selected == index
            {
                line = line.style(Style::default().bold());
            }

            line.render(area, buffer);
        }
    }
}
