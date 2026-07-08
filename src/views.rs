use std::marker::PhantomData;

use log::error;
use ratatui::{
    prelude::*,
    widgets::{Scrollbar, ScrollbarOrientation},
};

use crate::models::{File, Files, IconTheme};

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
            let icon = self.files.icons.use_icon(file);

            FileLine { file, icon, cursor }.render(file_area, buffer);
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
        let style = if self.cursor {
            Style::default().bold()
        } else {
            Style::default()
        };
        let indent_width = self.file.depth.saturating_sub(1).saturating_mul(2) as u16;
        let [_indent, icon_area, _empty, name_area] = area.layout(&Layout::horizontal([
            Constraint::Length(indent_width),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]));

        Span::raw(self.icon)
            .style(Style::default())
            .render(icon_area, buffer);
        Span::raw(file_name).style(style).render(name_area, buffer);
    }
}
