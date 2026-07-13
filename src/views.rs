use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use log::error;

use crate::models::{File, Files};

pub trait View {
    fn render(&self, area: Area, stdout: &mut impl Write) -> Result<(), io::Error>;
}

pub struct Area {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct FileTree<'a> {
    files: &'a Files,
}

impl<'a> FileTree<'a> {
    pub fn new(files: &'a Files) -> Self {
        Self { files }
    }
}

impl View for FileTree<'_> {
    fn render(&self, area: Area, stdout: &mut impl Write) -> Result<(), io::Error> {
        let height = area.height as usize;
        let scrollbar_x = area.width - 1;
        let scroll_offset = self.files.cursor().saturating_sub(height.saturating_sub(5));

        for (file_id, y_offset) in self
            .files
            .visible()
            .iter()
            .skip(scroll_offset)
            .take(height)
            .zip(0..)
        {
            let file = self.files.get_file(file_id);
            let file_area = Area {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: 1,
            };
            let cursor = self.files.cursor() as u16 == y_offset + scroll_offset as u16;

            FileLine { file, cursor }.render(file_area, stdout)?;
        }

        if self.files.visible().len() > height {
            for y in 0..area.height {
                queue!(stdout, MoveTo(scrollbar_x, y), Print("│"))?;
            }

            let scroll_position =
                (self.files.cursor() * height / self.files.visible().len()) as u16;

            queue!(stdout, MoveTo(scrollbar_x, scroll_position), Print("█"))?;
        }

        Ok(())
    }
}

struct FileLine<'a> {
    file: &'a File,
    cursor: bool,
}

impl View for FileLine<'_> {
    fn render(&self, area: Area, stdout: &mut impl Write) -> Result<(), io::Error> {
        let cursor_width = 2;
        let indent_width = (self.file.depth as u16 * 3) + cursor_width;
        let icon_width = 2;
        let name_x = icon_width + indent_width + 1;
        let file_name = &self
            .file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| {
                error!(
                    "Tried to render a nameless file: {}",
                    self.file.path.display()
                );

                "error: see logs for info"
            });
        let (icon_id, linked_path) = self.file.icon_id_and_linked_path();

        if self.cursor {
            queue!(stdout, MoveTo(area.x, area.y), Print("▷"))?;
        }

        queue!(
            stdout,
            MoveTo(indent_width, area.y),
            SetForegroundColor(Color::Rgb {
                r: 0,
                g: 0,
                b: icon_id.inner()
            }),
            Print("\u{10EEEE}\u{10EEEE}"),
            ResetColor,
        )?;
        queue!(stdout, MoveTo(name_x, area.y))?;

        if self.file.marked {
            queue!(
                stdout,
                SetAttribute(Attribute::Bold),
                Print(file_name),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(stdout, Print(file_name))?;
        }

        if let Some(linked_path) = linked_path {
            let file_name_end = name_x + file_name.chars().count() as u16;

            queue!(stdout, MoveTo(file_name_end + 1, area.y), Print("->"),)?;
            queue!(
                stdout,
                MoveTo(file_name_end + 4, area.y),
                Print(linked_path.display()),
            )?;
        }

        Ok(())
    }
}
