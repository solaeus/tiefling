use std::{fs::read_dir, io, path::PathBuf, range::Range};

use ratatui::widgets::ScrollbarState;

#[derive(Debug)]
pub struct Files {
    files: Vec<File>,
    children: Vec<FileId>,
    visible: Vec<FileId>,
    cursor: Option<usize>,
    scrollbar_state: ScrollbarState,
}

impl Files {
    pub fn new() -> Self {
        Files {
            files: Vec::new(),
            children: Vec::new(),
            visible: Vec::new(),
            cursor: None,
            scrollbar_state: ScrollbarState::new(0),
        }
    }

    pub fn visible(&self) -> &Vec<FileId> {
        &self.visible
    }

    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn scrollbar_state_mut(&mut self) -> &mut ScrollbarState {
        &mut self.scrollbar_state
    }

    pub fn open(&mut self, path: PathBuf, depth: u8) -> Result<FileId, io::Error> {
        let kind = if path.is_dir() {
            FileKind::Directory(None)
        } else if path.is_symlink() {
            FileKind::Symlink(path.read_link()?)
        } else {
            FileKind::Regular
        };
        let file = File::new(path, kind, depth);
        let file_id = FileId(self.files.len() as u32);

        self.files.push(file);

        Ok(file_id)
    }

    pub fn get_file(&self, id: &FileId) -> &File {
        &self.files[id.0 as usize]
    }

    pub fn get_file_mut(&mut self, id: &FileId) -> &mut File {
        &mut self.files[id.0 as usize]
    }

    pub fn get_child_ids(&self, children: &FileChildren) -> &[FileId] {
        &self.children[children.as_index_range()]
    }

    pub fn select_file(&mut self, id: FileId, visible_index: usize) -> Result<(), io::Error> {
        let file = self.get_file_mut(&id);

        match file.kind {
            FileKind::Directory(Some(children)) => {
                if file.expanded || children.is_empty() {
                    return Ok(());
                }

                file.expanded = true;

                for child_index in children.as_index_range() {
                    let file_id = self.children[child_index];

                    self.visible.push(file_id);
                }

                self.update_scrollbar();
            }
            FileKind::Directory(None) => {
                if file.expanded {
                    return Ok(());
                }

                file.expanded = true;

                let dir = read_dir(&file.path)?;
                let depth = file.depth + 1;

                for (read_result, next_visible_index) in dir.zip(visible_index..) {
                    let entry = read_result?;
                    let file_id = self.open(entry.path(), depth)?;

                    self.children.push(file_id);
                    self.visible.insert(next_visible_index, file_id);
                }

                self.update_scrollbar();
            }
            _ => (),
        }

        Ok(())
    }

    pub fn move_cursor_down(&mut self) {
        if let Some(index) = self.cursor
            && index < self.visible.len() - 1
        {
            self.cursor = Some(index + 1);
            self.scrollbar_state = self.scrollbar_state.position(index + 1);
        } else {
            self.cursor = Some(0);
            self.scrollbar_state = self.scrollbar_state.position(0);
        }
    }

    pub fn move_cursor_up(&mut self) {
        if let Some(index) = self.cursor
            && index > 0
        {
            self.cursor = Some(index - 1);
            self.scrollbar_state = self.scrollbar_state.position(index - 1);
        } else {
            self.cursor = Some(self.visible.len() - 1);
            self.scrollbar_state = self.scrollbar_state.position(self.visible.len() - 1);
        }
    }

    pub fn select_file_under_cursor(&mut self) -> Result<(), io::Error> {
        let Some(cursor_index) = self.cursor else {
            return Ok(());
        };
        let file_id = self.visible[cursor_index];

        self.select_file(file_id, cursor_index + 1)
    }

    pub fn update_scrollbar(&mut self) {
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.visible.len())
            .position(self.cursor.unwrap_or(0));
    }
}

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub kind: FileKind,
    pub depth: u8,
    pub expanded: bool,
    pub marked: bool,
}

impl File {
    pub fn new(path: PathBuf, kind: FileKind, depth: u8) -> Self {
        Self {
            path,
            kind,
            depth,
            expanded: false,
            marked: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(u32);

#[derive(Clone, Copy, Debug)]
pub struct FileChildren(Range<u32>);

impl FileChildren {
    pub fn new(start: u32, end: u32) -> Self {
        Self(Range::from(start..end))
    }

    pub fn as_index_range(&self) -> Range<usize> {
        Range {
            start: self.0.start as usize,
            end: self.0.end as usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug)]
pub enum FileKind {
    Regular,
    Directory(Option<FileChildren>),
    Symlink(PathBuf),
}
