use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use log::error;

use crate::models::{File, Files};

pub trait View {
    type Placement;
    type Style;

    fn render(
        &self,
        placement: Self::Placement,
        style: Self::Style,
        stdout: &mut impl Write,
    ) -> Result<(), io::Error>;
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
    type Placement = Area;
    type Style = (Color, Color);

    fn render(
        &self,
        placement: Area,
        (foreground_color, background_color): Self::Style,
        stdout: &mut impl Write,
    ) -> Result<(), io::Error> {
        let height = placement.height as usize;
        let scrollbar_x = placement.width - 1;
        let scroll_offset = self.files.cursor().saturating_sub(height / 2);
        let show_scrollbar = self.files.visible().len() > height;

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
                x: placement.x,
                y: placement.y + y_offset,
                width: if show_scrollbar {
                    placement.width - 1
                } else {
                    placement.width
                },
                height: 1,
            };
            let cursor = self.files.cursor() as u16 == y_offset + scroll_offset as u16;
            let (foreground_color, background_color) = if cursor {
                (background_color, foreground_color)
            } else {
                (foreground_color, background_color)
            };

            FileLine { file }.render(file_area, (foreground_color, background_color), stdout)?;
        }

        if show_scrollbar {
            let scroll_position =
                (self.files.cursor() * height / self.files.visible().len()) as u16;
            let scroll_resolution = (self.files.visible().len() / height + 1) as u16;

            for y in 0..placement.height {
                if y + scroll_resolution > scroll_position
                    && y.saturating_sub(scroll_resolution) < scroll_position
                {
                    execute!(stdout, MoveTo(scrollbar_x, y), Print("█"))?;
                } else {
                    execute!(stdout, MoveTo(scrollbar_x, y), Print("│"))?;
                }
            }
        }

        Ok(())
    }
}

struct FileLine<'a> {
    file: &'a File,
}

impl View for FileLine<'_> {
    type Placement = Area;
    type Style = (Color, Color);

    fn render(
        &self,
        placement: Self::Placement,
        (foreground_color, background_color): Self::Style,
        stdout: &mut impl Write,
    ) -> Result<(), io::Error> {
        let indent_width = self.file.depth as u16 * 3;
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
        let extra_width = (placement.width as usize)
            .saturating_sub(name_x as usize)
            .saturating_sub(file_name.chars().count());
        let (icon_id, linked_path) = self.file.icon_id_and_linked_path();

        execute!(
            stdout,
            MoveTo(0, placement.y),
            SetForegroundColor(foreground_color),
            SetBackgroundColor(background_color),
            Print(" ".repeat(indent_width as usize)),
            MoveTo(indent_width, placement.y),
            SetForegroundColor(Color::Rgb {
                r: 0,
                g: 0,
                b: icon_id.inner()
            }),
            Print("\u{10EEEE}\u{10EEEE}"),
            SetForegroundColor(foreground_color),
            SetBackgroundColor(background_color),
            Print(" "),
            MoveTo(name_x, placement.y),
            Print(file_name),
            Print(" ".repeat(extra_width)),
        )?;

        if let Some(linked_path) = linked_path {
            let file_name_end = name_x + file_name.chars().count() as u16;

            execute!(
                stdout,
                MoveTo(file_name_end + 1, placement.y),
                Print("->"),
                MoveTo(file_name_end + 4, placement.y),
                Print(linked_path.display()),
            )?;
        }

        execute!(stdout, ResetColor)?;

        Ok(())
    }
}
