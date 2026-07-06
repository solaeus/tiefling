use log::error;
use ratatui::prelude::*;

use crate::models::{File, Files};

pub struct FileTree<'a> {
    files: &'a Files,
}

impl<'a> FileTree<'a> {
    pub fn new(files: &'a Files) -> Self {
        Self { files }
    }
}

impl<'a> Widget for FileTree<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        for (y_offset, file_id) in self.files.visible().iter().enumerate() {
            let y_offset = y_offset as u16;

            if y_offset > area.height {
                break;
            }

            let file_area = Rect {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: 1,
            };
            let file = self.files.get_file(file_id);
            let cursor = self.files.cursor().is_some_and(|visible_index| {
                let cursor_file_id = self.files.visible()[visible_index];

                cursor_file_id == *file_id
            });

            FileLine { file, cursor }.render(file_area, buffer);
        }
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

                "file error: see logs for info"
            });
        let style = if self.cursor {
            Style::default().bold()
        } else {
            Style::default()
        };

        Span::raw(file_name).style(style).render(area, buffer);
    }
}
