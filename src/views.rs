use ratatui::prelude::*;

use crate::models::{File, Files};

pub struct FileTree;

impl StatefulWidget for FileTree {
    type State = Files;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let mut y_offset = 0;

        for file_id in state.visible() {
            if y_offset > area.height {
                break;
            }

            let file_area = Rect {
                x: area.x,
                y: area.y + y_offset,
                width: area.width,
                height: 1,
            };
            y_offset += 1;

            let file = state.get_file(file_id);

            FileLine { file }.render(file_area, buffer);
        }
    }
}

struct FileLine<'a> {
    file: &'a File,
}

impl<'a> Widget for FileLine<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        Span::raw(self.file.path.to_string_lossy()).render(area, buffer);
    }
}
