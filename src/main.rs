use std::{
    env,
    error::Error,
    fs::read_dir,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use devicons::FileIcon;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout},
    prelude::*,
    text::ToSpan,
};

const TARGET_FPS: u64 = 30;
const TARGET_FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);
const INPUT_INTERVAL: Duration = Duration::from_millis(100);
const REFRESH_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn Error>> {
    let pwd = env::current_dir().expect("I/O Error: Failed to get $PWD variable");
    let mut app_state = AppState::new(pwd);

    ratatui::run(|terminal| run(terminal, &mut app_state))?;

    Ok(())
}

fn run(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> io::Result<()> {
    let mut redraw = true;

    loop {
        if redraw {
            terminal.draw(|frame| {
                frame.render_stateful_widget(AppWidget, frame.area(), app_state);
            })?;
            redraw = false;
        }

        if event::poll(INPUT_INTERVAL)? {
            match event::read()?.as_key_press_event() {
                Some(key_press) if key_press.modifiers.is_empty() => match key_press.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('k') => app_state.root.select_previous_child(),
                    KeyCode::Down | KeyCode::Char('j') => app_state.root.select_next_child(),
                    KeyCode::Left | KeyCode::Char('h') => {
                        app_state.root.collapse_selected();
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

            redraw = true;
        }

        if app_state.last_refresh.elapsed() >= REFRESH_INTERVAL {
            if app_state.refresh() {
                redraw = true;
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct AppState {
    root: FileState,
    last_render: Instant,
    last_refresh: Instant,
}

impl AppState {
    fn new(pwd_path: PathBuf) -> Self {
        let now = Instant::now();
        let mut root = FileState::new(pwd_path, 0);

        root.selected = true;
        root.expand_selected();

        Self {
            root,
            last_render: now,
            last_refresh: now,
        }
    }

    fn refresh(&mut self) -> bool {
        self.last_refresh = Instant::now();
        self.root.refresh()
    }
}

#[derive(Debug)]
struct FileState {
    path: PathBuf,
    icon: Icon,
    dir: bool,
    selected: bool,
    expanded: bool,
    marked: bool,
    depth: u16,
    modified: SystemTime,
    children: Vec<FileState>,
    visible_children: u16,
}

impl FileState {
    fn new(path: PathBuf, depth: u16) -> Self {
        Self {
            dir: path.is_dir(),
            modified: path.metadata().unwrap().modified().unwrap(),
            icon: Icon::new(&path),
            path,
            selected: false,
            marked: false,
            expanded: false,
            depth,
            children: Vec::new(),
            visible_children: 0,
        }
    }

    fn read(&mut self) {
        self.modified = self.path.metadata().unwrap().modified().unwrap();

        let read_path = read_dir(&self.path).expect(&self.error_message());

        for result in read_path {
            let entry = result.expect(&self.error_message());
            let listing = FileState::new(entry.path(), self.depth + 1);

            self.children.push(listing);
        }
    }

    fn refresh(&mut self) -> bool {
        if self.modified == self.path.metadata().unwrap().modified().unwrap() {
            return false;
        }

        self.children.clear();
        self.read();

        let mut changed = false;

        for child_state in &mut self.children {
            if child_state.refresh() {
                changed = true;
            }
        }

        changed
    }

    fn expand_selected(&mut self) {
        if !self.dir {
            return;
        }

        if self.selected {
            if self.children.is_empty() {
                self.read();
            }

            self.visible_children = self.children.len() as u16;

            for file_state in &self.children {
                self.visible_children += file_state.visible_children;
            }

            self.expanded = true;

            return;
        }

        if self.expanded {
            self.visible_children = self.children.len() as u16;

            for file_state in &mut self.children {
                file_state.expand_selected();

                self.visible_children += file_state.visible_children;
            }
        }
    }

    fn collapse_selected(&mut self) {
        if !self.dir {
            return;
        }

        if self.selected {
            self.expanded = false;
            self.visible_children = 0;

            return;
        }

        if self.expanded {
            self.visible_children = self.children.len() as u16;

            for child_state in &mut self.children {
                child_state.collapse_selected();
                self.visible_children += child_state.visible_children;
            }
        }
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

        let mut frame_time = state.last_render.elapsed();

        while frame_time < TARGET_FRAME_TIME {
            frame_time = state.last_render.elapsed();
        }

        Span::raw(frame_time.as_millis().to_string()).render(title_area, buffer);
        FileWidget.render(child_area, buffer, &mut state.root);

        state.last_render = Instant::now();
    }
}

struct FileWidget;

impl StatefulWidget for FileWidget {
    type State = FileState;

    fn render(self, area: Rect, buffer: &mut Buffer, file_state: &mut Self::State)
    where
        Self: Sized,
    {
        let [name_area, child_area] = area.layout(&Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(file_state.visible_children as u16),
        ]));
        let depth = if file_state.depth == 0 {
            0
        } else {
            file_state.depth * 2 + 2
        };
        let [_indent_area, icon_area, _spacing, name_text_area] =
            name_area.layout(&Layout::horizontal([
                Constraint::Length(depth),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]));

        file_state
            .icon
            .glyph
            .to_span()
            .style(Style::default().fg(file_state.icon.color))
            .render(icon_area, buffer);

        let mut name_text = Span::raw(file_state.path.file_name().unwrap().to_string_lossy());

        if file_state.selected {
            name_text = name_text.style(Style::default().bold());
        }

        name_text.render(name_text_area, buffer);

        if !file_state.expanded {
            return;
        }

        let mut next_y = child_area.y;

        for file_state in &mut file_state.children {
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

#[derive(Debug)]
struct Icon {
    glyph: char,
    color: Color,
}

impl Icon {
    fn new(path: &Path) -> Self {
        let devicon = FileIcon::from(path);
        let red = devicon.color[1..3].parse::<u8>().unwrap_or(255);
        let green = devicon.color[3..5].parse::<u8>().unwrap_or(255);
        let blue = devicon.color[5..7].parse::<u8>().unwrap_or(255);

        Self {
            glyph: devicon.icon,
            color: Color::Rgb(red, green, blue),
        }
    }
}
