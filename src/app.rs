use std::{env::current_dir, io};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyCode},
};

use crate::{models::Files, views::FileTree};

pub struct App {
    files: Files,
}

impl App {
    pub fn new() -> Self {
        Self {
            files: Files::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
        let root_file_id = self.files.open(current_dir()?, 0)?;

        self.files.expand(root_file_id)?;

        loop {
            terminal.draw(|frame| {
                frame.render_stateful_widget(FileTree, frame.area(), &mut self.files);
            })?;

            match event::read()?.as_key_press_event() {
                Some(key_press) if key_press.modifiers.is_empty() => match key_press.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(())
    }
}
