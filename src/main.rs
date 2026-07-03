use std::{
    env,
    error::Error,
    fs::read_dir,
    hash::{Hash, Hasher},
    io,
    path::PathBuf,
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout},
    prelude::*,
};

fn main() -> Result<(), Box<dyn Error>> {
    let pwd = env::current_dir().expect("I/O Error: Failed to get $PWD variable");
    let mut app_state = AppState::new(pwd);

    ratatui::run(|terminal| run(terminal, &mut app_state))?;

    println!("{app_state:#?}");

    Ok(())
}

fn run(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        match event::read()?.as_key_press_event() {
            Some(key_press) if key_press.modifiers.is_empty() => match key_press.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Up | KeyCode::Char('k') => app_state.root.select_previous_child(),
                KeyCode::Down | KeyCode::Char('j') => app_state.root.select_next_child(),
                KeyCode::Left | KeyCode::Char('h') => {
                    if let Some(selected_file) = app_state.root.get_selected_mut() {
                        selected_file.collapse();
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app_state.root.expand_selected();
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    todo!("Handle marking files")
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let [area] = frame
        .area()
        .layout(&Layout::vertical([Constraint::Fill(1)]));

    frame.render_stateful_widget(AppWidget, area, app_state);
}

#[derive(Debug)]
struct AppState {
    root: FileState,
}

impl AppState {
    fn new(pwd_path: PathBuf) -> Self {
        let mut root = FileState::new(pwd_path, 0);

        root.selected = true;
        root.expand_selected();

        Self { root }
    }
}

#[derive(Debug)]
struct FileState {
    path: PathBuf,
    dir: bool,
    selected: bool,
    marked: bool,
    expanded: bool,
    depth: u16,
    children: Vec<FileState>,
    visible_children: u16,
}

impl FileState {
    fn new(path: PathBuf, depth: u16) -> Self {
        Self {
            dir: path.is_dir(),
            path,
            selected: false,
            marked: false,
            expanded: false,
            depth,
            children: Vec::new(),
            visible_children: 0,
        }
    }

    fn expand_selected(&mut self) -> bool {
        if self.selected {
            self.expand();
            self.visible_children += self.children.len() as u16;

            return true;
        }

        let child_count = self.children.len() as u16;

        for child in &mut self.children {
            if child.expand_selected() {
                self.visible_children = child_count + child.visible_children;

                return true;
            }
        }

        false
    }

    fn expand(&mut self) -> u16 {
        if self.expanded || !self.path.is_dir() {
            return 0;
        }

        let lines_added = if self.children.is_empty() {
            let read_path = read_dir(&self.path).expect(&self.error_message());

            for result in read_path {
                let entry = result.expect(&self.error_message());
                let listing = FileState::new(entry.path(), self.depth + 1);

                self.children.push(listing);
            }

            self.children.len() as u16
        } else {
            let mut total_children = 0;

            for file_state in &self.children {
                total_children += file_state.visible_children;
            }

            total_children
        };

        self.expanded = true;

        lines_added
    }

    fn collapse(&mut self) {
        self.expanded = false;
        self.visible_children = 0;
    }

    fn select_next_child(&mut self) {
        fn advance(state: &mut FileState, select_next: &mut bool) -> bool {
            for child in &mut state.children {
                if *select_next {
                    child.selected = true;
                    *select_next = false;

                    return true;
                }

                if child.selected {
                    child.selected = false;
                    *select_next = true;
                }

                if child.expanded && advance(child, select_next) {
                    return true;
                }
            }

            false
        }

        if self.selected && self.expanded && !self.children.is_empty() {
            self.selected = false;
            self.select_first_child();

            return;
        }

        self.selected = false;
        let mut select_next = false;

        if !advance(self, &mut select_next) {
            if select_next {
                self.select_first_child();
            } else {
                self.select_last_child();
            }
        }
    }

    fn select_previous_child(&mut self) {
        fn retreat(state: &mut FileState, select_next: &mut bool) -> bool {
            for child in state.children.iter_mut().rev() {
                if child.expanded && retreat(child, select_next) {
                    return true;
                }

                if *select_next {
                    child.selected = true;
                    *select_next = false;

                    return true;
                }

                if child.selected {
                    child.selected = false;
                    *select_next = true;
                }
            }

            false
        }

        if self.selected && self.expanded && !self.children.is_empty() {
            self.selected = false;
            self.select_last_child();

            return;
        }

        let mut select_next = false;

        if !retreat(self, &mut select_next) {
            self.selected = false;

            if select_next {
                self.select_last_child();
            } else {
                self.select_first_child();
            }
        }
    }

    fn select_first_child(&mut self) {
        if let Some(first) = self.children.first_mut() {
            first.selected = true;
        }
    }

    fn select_last_child(&mut self) {
        if let Some(last) = self.children.last_mut() {
            if last.expanded && !last.children.is_empty() {
                last.select_last_child();
            } else {
                last.selected = true;
            }
        }
    }

    fn get_selected_mut(&mut self) -> Option<&mut FileState> {
        if self.selected {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(selected) = child.get_selected_mut() {
                return Some(selected);
            }
        }

        None
    }

    fn error_message(&self) -> String {
        format!("I/O Error: Failed to read {}", self.path.display())
    }
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = AppState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let [title_area, child_area] = area.layout(&Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ]));

        FileWidget.render(child_area, buffer, &mut state.root);
    }
}

struct FileWidget;

impl StatefulWidget for FileWidget {
    type State = FileState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let [name_area, child_area] = area.layout(&Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(state.visible_children as u16),
        ]));
        let [_indent_area, icon_area, _spacing, name_text_area] =
            name_area.layout(&Layout::horizontal([
                Constraint::Length(state.depth * 2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]));

        if state.dir {
            let expansion_symbol = if state.expanded { "🗁" } else { "🖿" };

            Span::raw(expansion_symbol).render(icon_area, buffer);
        }

        let mut name_text = Span::raw(state.path.file_name().unwrap().to_string_lossy());

        if state.selected {
            name_text = name_text.style(Style::default().bold());
        }

        name_text.render(name_text_area, buffer);

        if !state.expanded {
            return;
        }

        let mut next_y = child_area.y;

        for file_state in &mut state.children {
            if next_y >= child_area.bottom() {
                break;
            }

            let height = file_state.visible_children + 1;
            let row = Rect {
                x: area.x,
                y: next_y,
                width: area.width,
                height,
            };
            next_y += height;

            FileWidget.render(row, buffer, file_state);
        }
    }
}
