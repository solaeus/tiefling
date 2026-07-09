use log::error;
use ratatui::{
    prelude::*,
    widgets::{Scrollbar, ScrollbarOrientation},
};

use crate::models::{File, Files};

pub struct FileTree<'a> {
    files: &'a mut Files,
}

impl<'a> FileTree<'a> {
    pub fn new(files: &'a mut Files) -> Self {
        Self { files }
    }
}

impl<'a> Widget for FileTree<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let height = area.height as usize;
        let scroll_offset = self
            .files
            .cursor()
            .unwrap_or(0)
            .saturating_sub(height.saturating_sub(5));

        for (y_offset, file_id) in self
            .files
            .visible()
            .iter()
            .skip(scroll_offset)
            .take(height)
            .enumerate()
        {
            let index = y_offset + scroll_offset;
            let y_offset = y_offset as u16;

            let file_area = Rect {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: 1,
            };
            let file = self.files.get_file(file_id);
            let cursor = self.files.cursor() == Some(index);
            let (icon, icon_color) = self.files.icons.get_icon(file);

            FileLine {
                file,
                icon,
                icon_color,
                cursor,
            }
            .render(file_area, buffer);
        }

        let scrollbar_area = area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            scrollbar_area,
            buffer,
            self.files.scrollbar_state_mut(),
        );
    }
}

struct FileLine<'a> {
    file: &'a File,
    icon: &'static str,
    icon_color: Option<Color>,
    cursor: bool,
}

impl<'a> Widget for FileLine<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let file_name = self
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
        let name_style = if self.cursor {
            Style::default().bold()
        } else {
            Style::default()
        };
        let indent_width = {
            if self.file.depth <= 1 {
                0
            } else {
                self.file
                    .depth
                    .saturating_sub(1)
                    .saturating_mul(2)
                    .saturating_add(1) as u16
            }
        };
        let icon_style = if let Some(icon_color) = self.icon_color {
            Style::default().fg(icon_color)
        } else {
            Style::default()
        };
        let [_indent, icon_area, _empty, name_area] = area.layout(&Layout::horizontal([
            Constraint::Length(indent_width),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]));

        Span::raw(self.icon)
            .style(icon_style)
            .render(icon_area, buffer);
        Span::raw(file_name)
            .style(name_style)
            .render(name_area, buffer);
    }
}
