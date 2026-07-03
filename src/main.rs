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
        let mut root = FileState::new(pwd_path);

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
    children: Vec<FileState>,
    visible_children: usize,
}

impl FileState {
    fn new(path: PathBuf) -> Self {
        Self {
            dir: path.is_dir(),
            path,
            selected: false,
            marked: false,
            expanded: false,
            children: Vec::new(),
            visible_children: 0,
        }
    }

    fn expand_selected(&mut self) -> bool {
        if self.selected {
            self.expand();
            self.visible_children += self.children.len();

            return true;
        }

        let child_count = self.children.len();

        for child in &mut self.children {
            if child.expand_selected() {
                self.visible_children = child_count + child.visible_children;

                return true;
            }
        }

        false
    }

    fn expand(&mut self) {
        if self.expanded || !self.path.is_dir() {
            return;
        }

        if !self.children.is_empty() {
            self.expanded = true;

            return;
        }

        let read_path = read_dir(&self.path).expect(&self.error_message());

        for result in read_path {
            let entry = result.expect(&self.error_message());
            let listing = FileState::new(entry.path());

            self.children.push(listing);
        }

        self.expanded = true;
    }

    fn collapse(&mut self) {
        self.expanded = false;
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

        let mut select_next = false;

        if !advance(self, &mut select_next) {
            self.selected = false;

            if select_next {
                self.select_last_child();
            } else {
                self.select_first_child();
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

        let mut select_next = false;

        if !retreat(self, &mut select_next) {
            self.selected = false;
            self.select_last_child();
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
        let [pwd_area, child_area] = area.layout(&Layout::vertical([
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
            Constraint::Min(0),
        ]));

        let [expansion_symbol_area, name_text_area] = name_area.layout(&Layout::horizontal([
            Constraint::Length(2),
            Constraint::Fill(1),
        ]));

        if state.dir {
            let expansion_symbol = if state.expanded { "🗁 " } else { "🖿 " };

            Span::raw(expansion_symbol).render(expansion_symbol_area, buffer);
        }

        let mut name_text = Span::raw(state.path.file_name().unwrap().to_string_lossy());

        if state.selected {
            name_text = name_text.style(Style::default().bold());
        }

        name_text.render(name_text_area, buffer);

        if state.expanded {
            let child_areas = child_area.layout_vec(&Layout::vertical(vec![
                Constraint::Length(1);
                state.visible_children
            ]));

            for (child_state, child_area) in state.children.iter_mut().zip(child_areas) {
                FileWidget.render(child_area, buffer, child_state);
            }
        }
    }
}
