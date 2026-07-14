use std::{
    io::{self, Write, stdout},
    path::PathBuf,
    time::Instant,
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print},
    terminal::{Clear, ClearType, size},
};

use crate::{
    icons::Icons,
    models::Files,
    terminal::Terminal,
    views::{Area, FileTree, View},
};

#[derive(Debug)]
pub struct App {
    files: Files,
    icons: Icons,
}

impl App {
    pub fn new(root: PathBuf, icons: Icons) -> Result<Self, io::Error> {
        Ok(Self {
            files: Files::new(root)?,
            icons,
        })
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        let mut terminal = Terminal::new(stdout().lock())?;

        self.icons.load_icons(terminal.stdout())?;

        let mut last_render = Instant::now();

        loop {
            let (width, height) = size()?;
            let render_time = last_render.elapsed();
            last_render = Instant::now();
            let file_tree = FileTree::new(&self.files);
            let file_tree_area = Area {
                x: 0,
                y: 1,
                width,
                height,
            };

            execute!(terminal.stdout(), Clear(ClearType::All))?;
            execute!(
                terminal.stdout(),
                MoveTo(width / 2, 0),
                Print(render_time.as_millis()),
                Print("ms")
            )?;
            file_tree.render(
                file_tree_area,
                (
                    Color::Rgb {
                        r: 166,
                        g: 166,
                        b: 166,
                    },
                    Color::Rgb {
                        r: 44,
                        g: 44,
                        b: 44,
                    },
                ),
                terminal.stdout(),
            )?;
            terminal.stdout().flush()?;

            let input_event = event::read()?;
            let quit = self.handle_input(input_event)?;

            if quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, event: Event) -> Result<bool, io::Error> {
        let Event::Key(key_press) = event else {
            return Ok(false);
        };

        if !key_press.modifiers.is_empty() {
            return Ok(false);
        }

        match key_press.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Down => self.files.move_cursor_down(),
            KeyCode::Up => self.files.move_cursor_up(),
            KeyCode::Right => self.files.expand_directory_under_cursor()?,
            KeyCode::Char(' ') => self.files.toggle_file_under_cursor_marked(),
            _ => {}
        }

        Ok(false)
    }
}
