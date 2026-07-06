use std::{env::current_dir, io};

use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, KeyCode, KeyEvent},
};

use crate::{config::Config, models::Files, views::FileTree};

#[derive(Debug)]
pub struct App {
    config: Config,
    files: Files,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            files: Files::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), io::Error> {
        let root_file_id = self.files.open(current_dir()?, 0)?;

        self.files.expand(root_file_id, 0)?;

        loop {
            terminal.draw(|frame| {
                frame.render_stateful_widget(FileTree, frame.area(), &mut self.files);
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
            KeyCode::Right => self.files.expand_under_cursor()?,
            _ => {}
        }

        Ok(false)
    }
}
