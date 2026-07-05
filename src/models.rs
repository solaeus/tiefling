use std::{fs::read_dir, io, path::PathBuf, range::Range};

pub struct Files {
    files: Vec<File>,
    children: Vec<FileId>,
    visible: Vec<FileId>,
}

impl Files {
    pub fn new() -> Self {
        Files {
            files: Vec::new(),
            children: Vec::new(),
            visible: Vec::new(),
        }
    }

    pub fn visible(&self) -> &Vec<FileId> {
        &self.visible
    }

    pub fn open(&mut self, path: PathBuf, depth: u8) -> Result<FileId, io::Error> {
        if !path.is_dir() {
            let kind = if path.is_symlink() {
                FileKind::Symlink(path.read_link()?)
            } else {
                FileKind::Regular
            };
            let file = File::new(path, kind, depth);
            let file_id = FileId(self.files.len() as u32);

            self.files.push(file);

            return Ok(file_id);
        }

        let file = File::new(path, FileKind::Directory(None), depth);
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

    pub fn expand(&mut self, id: FileId) -> Result<(), io::Error> {
        let file = self.get_file_mut(&id);

        if file.expanded {
            return Ok(());
        }

        file.expanded = true;

        match file.kind {
            FileKind::Directory(Some(children)) => {
                for child_index in children.as_index_range() {
                    let file_id = self.children[child_index];

                    self.visible.push(file_id);
                }
            }
            FileKind::Directory(None) => {
                let dir = read_dir(&file.path)?;
                let depth = file.depth + 1;

                for read_result in dir {
                    let entry = read_result?;
                    let file_id = self.open(entry.path(), depth)?;

                    self.children.push(file_id);
                    self.visible.push(file_id);
                }
            }
            _ => (),
        }

        Ok(())
    }
}

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

#[derive(Clone, Copy)]
pub struct FileId(u32);

#[derive(Clone, Copy)]
pub struct FileChildren(Range<u32>);

impl FileChildren {
    pub fn new(start: u32, end: u32) -> Self {
        Self(Range::from(start..end))
    }

    pub fn empty() -> Self {
        Self(Range::from(0..0))
    }

    pub fn as_index_range(&self) -> Range<usize> {
        Range {
            start: self.0.start as usize,
            end: self.0.end as usize,
        }
    }
}

pub enum FileKind {
    Regular,
    Directory(Option<FileChildren>),
    Symlink(PathBuf),
}
