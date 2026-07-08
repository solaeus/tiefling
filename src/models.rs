use std::{
    fmt::Debug,
    fs::read_dir,
    io::{self, Write, stdout},
    path::{Path, PathBuf},
    range::Range,
};

use ratatui::widgets::ScrollbarState;

#[derive(Debug)]
pub struct Files {
    files: Vec<File>,
    children: Vec<FileId>,
    visible: Vec<FileId>,
    loaded_icons: Vec<FileExtension>,
    cursor: Option<usize>,
    scrollbar_state: ScrollbarState,
    pub icons: Icons,
}

impl Files {
    pub fn new<I: IconTheme>() -> Self {
        Files {
            files: Vec::new(),
            children: Vec::new(),
            visible: Vec::new(),
            loaded_icons: Vec::new(),
            cursor: None,
            scrollbar_state: ScrollbarState::new(0),
            icons: I::default().into_icons(),
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
        self.open_inner(path, depth, stdout())
    }

    fn open_inner(
        &mut self,
        path: PathBuf,
        depth: u8,
        stdout: impl Write,
    ) -> Result<FileId, io::Error> {
        let kind = if path.is_dir() {
            FileKind::Directory(None)
        } else if path.is_symlink() {
            FileKind::Symlink(path.read_link()?)
        } else {
            FileKind::Regular(FileExtension::from_path(&path))
        };
        let file = File::new(path, kind, depth);
        let file_id = FileId(self.files.len() as u32);

        self.icons.load_icon(&file, stdout)?;
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

                let mut stdout = stdout().lock();

                for (read_result, next_visible_index) in dir.zip(visible_index..) {
                    let entry = read_result?;
                    let file_id = self.open_inner(entry.path(), depth, &mut stdout)?;

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
    Regular(FileExtension),
    Directory(Option<FileChildren>),
    Symlink(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FileExtension {
    Unknown,
    Rust,
    Toml,
}

impl FileExtension {
    fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("rs") => Self::Rust,
            Some("toml") => Self::Toml,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Icons {
    Emoji(EmojiIconTheme),
    JetBrains(JetBrainsIconTheme),
}

impl Icons {
    pub fn load_icon(&mut self, file: &File, stdout: impl Write) -> Result<(), io::Error> {
        match self {
            Icons::Emoji(theme) => theme.load_icon(file, stdout),
            Icons::JetBrains(theme) => theme.load_icon(file, stdout),
        }
    }

    pub fn use_icon(&self, file: &File) -> &'static str {
        match self {
            Icons::Emoji(theme) => theme.use_icon(file),
            Icons::JetBrains(theme) => theme.use_icon(file),
        }
    }
}

pub trait IconTheme: Debug + Default {
    fn into_icons(self) -> Icons;
    fn load_icon(&mut self, file: &File, stdout: impl Write) -> Result<(), io::Error>;
    fn use_icon(&self, file: &File) -> &'static str;
}

#[derive(Debug, Default)]
pub struct EmojiIconTheme;

impl IconTheme for EmojiIconTheme {
    fn into_icons(self) -> Icons {
        Icons::Emoji(self)
    }

    fn load_icon(&mut self, _: &File, _: impl Write) -> Result<(), io::Error> {
        Ok(())
    }

    fn use_icon(&self, file: &File) -> &'static str {
        match &file.kind {
            FileKind::Regular(extension) => match extension {
                FileExtension::Rust => "\u{1F980}",
                _ => "\u{1F4C4}",
            },
            FileKind::Directory(_) => {
                if file.expanded {
                    "\u{1F4C2}"
                } else {
                    "\u{1F4C1}"
                }
            }
            FileKind::Symlink(_) => "",
        }
    }
}

#[derive(Debug, Default)]
pub struct JetBrainsIconTheme {
    unknown: bool,
    directory: bool,
    rust: bool,
    toml: bool,
}

impl IconTheme for JetBrainsIconTheme {
    fn into_icons(self) -> Icons {
        Icons::JetBrains(self)
    }

    fn load_icon(&mut self, file: &File, stdout: impl Write) -> Result<(), io::Error> {
        let (id, icon) = match &file.kind {
            FileKind::Directory(_) if !self.directory => (
                1,
                include_bytes!("../assets/jetbrains_icons/folder.b64").as_slice(),
            ),
            FileKind::Regular(extension) => match extension {
                FileExtension::Rust if !self.rust => (
                    2,
                    include_bytes!("../assets/jetbrains_icons/rust.b64").as_slice(),
                ),
                FileExtension::Toml if !self.toml => (
                    3,
                    include_bytes!("../assets/jetbrains_icons/toml.b64").as_slice(),
                ),
                _ if !self.unknown => (
                    4,
                    include_bytes!("../assets/jetbrains_icons/anyType.b64").as_slice(),
                ),
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };

        load_icon(id, icon, stdout)
    }

    fn use_icon(&self, file: &File) -> &'static str {
        match &file.kind {
            FileKind::Directory(_) => "\x1b_Gi=1\x1b",
            FileKind::Regular(extension) => match extension {
                FileExtension::Rust => "\x1b_Gi=2\x1b",
                FileExtension::Toml => "\x1b_Gi=3\x1b",
                _ => "\x1b_Gi=4\x1b",
            },
            _ => "\x1b_Gi=4\x1b",
        }
    }
}

/// https://sw.kovidgoyal.net/kitty/graphics-protocol/#a-minimal-example
fn load_icon(id: u8, icon: &[u8], mut writer: impl Write) -> Result<(), io::Error> {
    let mut chunks = icon.chunks(4096).peekable();
    let mut first = true;

    while let Some(chunk) = chunks.next() {
        let remaining = u8::from(chunks.peek().is_some());

        if first {
            first = false;
            write!(writer, "\x1b_Ga=t,f=100,q=2,i={id},m={remaining};")?;
        } else {
            write!(writer, "\x1b_Gm={remaining};")?;
        }

        writer.write_all(chunk)?;
        writer.write_all(b"\x1b\\")?;
    }

    Ok(())
}
