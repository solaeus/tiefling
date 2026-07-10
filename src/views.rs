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
        let scroll_offset = self.files.cursor().saturating_sub(height.saturating_sub(5));

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
            let cursor = self.files.cursor() == index;

            FileLine { file, cursor }.render(file_area, buffer);
        }

        let scrollbar_area = area.inner(Margin {
            horizontal: 1,
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
    cursor: bool,
}

impl<'a> Widget for FileLine<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let indent_width = self.file.depth * 2;
        let icon_style = Style::default().fg(Color::Rgb(0, 0, self.file.icon_id().inner()));
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
        let [_indent, icon_area, _space, name_area, _scrollbar] =
            area.layout(&Layout::horizontal([
                Constraint::Length(indent_width as u16),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ]));

        Span::raw("\u{10EEEE}\u{10EEEE}")
            .style(icon_style)
            .render(icon_area, buffer);
        Span::raw(file_name)
            .style(name_style)
            .render(name_area, buffer);
    }
}
