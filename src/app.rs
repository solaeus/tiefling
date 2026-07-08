use std::{env::current_dir, io};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyCode, KeyEvent},
};

use crate::{
    models::{Files, IconTheme},
    views::FileTree,
};

#[derive(Debug)]
pub struct App {
    files: Files,
}

impl App {
    pub fn new<I: IconTheme>() -> Self {
        Self {
            files: Files::new::<I>(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
        let root_file_id = self.files.open(current_dir()?, 0)?;

        self.files.select_file(root_file_id, 0)?;

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
            KeyCode::Right => self.files.select_file_under_cursor()?,
            _ => {}
        }

        Ok(false)
    }
}
