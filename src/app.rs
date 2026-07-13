use std::{
    io::{self, Write, stdout},
    path::PathBuf,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseEvent},
    terminal::size,
};

use crate::{
    models::{Files, Icons},
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
        let mut terminal = Terminal::new(stdout())?;

        self.icons.load_icons(terminal.stdout())?;

        loop {
            let file_tree = FileTree::new(&self.files);
            let (width, height) = size()?;
            let file_tree_area = Area {
                x: 0,
                y: 0,
                width,
                height,
            };

            file_tree.render(file_tree_area, terminal.stdout())?;
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
        if let Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers,
        }) = event
        {
            self.files.set_cursor(column as usize);
        }

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
            KeyCode::Right => self.files.toggle_file_under_cursor()?,
            _ => {}
        }

        Ok(false)
    }
}
