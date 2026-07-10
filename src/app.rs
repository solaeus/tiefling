use std::{io, path::PathBuf};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyCode, KeyEvent},
};

use crate::{
    models::{Files, Icons},
    views::FileTree,
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
        self.icons.load_icons()?;

        loop {
            terminal.draw(|frame| {
                frame.render_widget(FileTree::new(&mut self.files), frame.area());
            })?;

            if let Some(key_press) = event::read()?.as_key_press_event() {
                let quit = self.handle_input(key_press)?;

                if quit {
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, event: KeyEvent) -> Result<bool, io::Error> {
        if !event.modifiers.is_empty() {
            return Ok(false);
        }

        match event.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Down => self.files.move_cursor_down(),
            KeyCode::Up => self.files.move_cursor_up(),
            KeyCode::Right => self.files.toggle_file_under_cursor()?,
            _ => {}
        }

        Ok(false)
    }
}
