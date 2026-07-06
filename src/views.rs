use log::error;
use ratatui::{
    prelude::*,
    widgets::{Scrollbar, ScrollbarOrientation},
};

use crate::models::{File, Files};

pub struct FileTree;

impl StatefulWidget for FileTree {
    type State = Files;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let height = area.height as usize;
        let scroll_offset = state.cursor().unwrap_or(0).saturating_sub(height - 1);

        for (y_offset, file_id) in state
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
            let file = state.get_file(file_id);
            let cursor = state.cursor() == Some(index);

            FileLine { file, cursor }.render(file_area, buffer);
        }

        let scrollbar_area = area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            scrollbar_area,
            buffer,
            state.scrollbar_state_mut(),
        );
    }
}

struct FileLine<'a> {
    file: &'a File,
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
        let [_indent, name_area] = area.layout(&Layout::horizontal([
            Constraint::Length(indent_width),
            Constraint::Fill(1),
        ]));

        Span::raw(file_name).style(style).render(name_area, buffer);
    }
}
