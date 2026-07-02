use std::{
    borrow::Cow,
    error::Error,
    fmt::{Display, Pointer},
    fs::{DirEntry, FileType, ReadDir, read_dir},
    io,
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, HorizontalAlignment, Layout},
    macros::constraints,
    prelude::*,
    style::Style,
    text::{Line, Span, Text, ToSpan},
    widgets::{List, ListItem, ListState, Widget},
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
    let root_files =
        read_dir(".").map_or_else(|_| OpenDir::empty(), |entries| OpenDir::new(entries));
    let app = App::new(root_files);
    let mut app_state = AppState::default();

    frame.render_stateful_widget(app, area, &mut app_state);
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
        self.root.render(area, buffer);
    }
}

#[derive(Default)]
struct AppState {
    selected: Option<usize>,
}

struct OpenDir {
    files: Vec<FileListing>,
}

impl OpenDir {
    fn new(dir: ReadDir) -> Self {
        let mut files = Vec::new();

        for entry in dir {
            let listing = match entry {
                Ok(entry) => FileListing::new(entry, false),
                Err(_) => FileListing::Error,
            };

            files.push(listing);
        }

        Self { files }
    }

    fn empty() -> Self {
        Self { files: Vec::new() }
    }
}

impl Widget for OpenDir {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let constraints = vec![Constraint::Length(1); self.files.len()];
        let areas = area.layout_vec(&Layout::vertical(constraints));

        for (file, area) in self.files.into_iter().zip(areas) {
            file.render(area, buffer);
        }
    }
}

enum FileListing {
    Selected(String),
    Unselected(String),
    OpenDir(OpenDir),
    Error,
}

impl FileListing {
    fn new(entry: DirEntry, selected: bool) -> Self {
        match entry.file_type() {
            Ok(_) => {
                let name = entry.file_name().to_string_lossy().to_string();

                if selected {
                    Self::Selected(name)
                } else {
                    Self::Unselected(name)
                }
            }
            _ => Self::Error,
        }
    }
}

impl Widget for FileListing {
    fn render(self, area: Rect, buffer: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            FileListing::Selected(name) => Line::raw(&name).bold().render(area, buffer),
            FileListing::Unselected(name) => Line::raw(&name).render(area, buffer),
            FileListing::OpenDir(open_dir) => open_dir.render(area, buffer),
            FileListing::Error => Line::raw("!tf_error!").render(area, buffer),
        }
    }
}
