use std::{
    fmt::Debug,
    io::{self, Write, stdout},
    path::{Path, PathBuf},
    range::Range,
};

use ratatui::{style::Color, widgets::ScrollbarState};

#[derive(Debug)]
pub struct Files {
    files: Vec<File>,
    children: Vec<FileId>,
    visible: Vec<FileId>,
    cursor: Option<usize>,
    scrollbar_state: ScrollbarState,
    pub icons: Icons,
}

impl Files {
    pub fn new(icons: Icons) -> Self {
        Files {
            files: Vec::new(),
            children: Vec::new(),
            visible: Vec::new(),
            cursor: None,
            scrollbar_state: ScrollbarState::new(0),
            icons,
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
        let file = self.get_file(&id);
        let children_start = self.children.len() as u32;

        match file.kind {
            FileKind::Directory(Some(children)) => {
                if file.expanded || children.is_empty() {
                    return Ok(());
                }

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

                let depth = file.depth + 1;
                let dir = file.path.read_dir()?;
                let mut stdout = stdout().lock();

                for (read_result, next_visible_index) in dir.zip(visible_index..) {
                    let entry = read_result?;
                    let file_id = self.open_inner(entry.path(), depth, &mut stdout)?;

                    self.children.push(file_id);
                    self.visible.insert(next_visible_index, file_id);
                }

                self.update_scrollbar();
            }
            _ => return Ok(()),
        };

        let children_end = self.children.len() as u32;
        let file = self.get_file_mut(&id);
        file.children = FileChildren::new(children_start, children_end);

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
    pub children: FileChildren,
    pub depth: u8,
    pub expanded: bool,
    pub marked: bool,
}

impl File {
    pub fn new(path: PathBuf, kind: FileKind, depth: u8) -> Self {
        Self {
            path,
            kind,
            children: FileChildren::default(),
            depth,
            expanded: false,
            marked: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(u32);

#[derive(Clone, Copy, Debug, Default)]
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
pub enum FileExtension {
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
    pub fn emoji() -> Self {
        Self::Emoji(EmojiIconTheme)
    }

    pub fn jet_brains() -> Self {
        Self::JetBrains(JetBrainsIconTheme::default())
    }

    pub fn load_icon(&mut self, file: &File, stdout: impl Write) -> Result<(), io::Error> {
        match self {
            Icons::Emoji(theme) => theme.load_icon(file, stdout),
            Icons::JetBrains(theme) => theme.load_icon(file, stdout),
        }
    }

    pub fn get_icon(&self, file: &File) -> (&'static str, Option<Color>) {
        match self {
            Icons::Emoji(theme) => theme.get_icon(file),
            Icons::JetBrains(theme) => theme.get_icon(file),
        }
    }
}

pub trait IconTheme: Debug + Default {
    fn load_icon(&mut self, file: &File, stdout: impl Write) -> Result<(), io::Error>;
    fn get_icon(&self, file: &File) -> (&'static str, Option<Color>);
}

#[derive(Debug, Default)]
pub struct EmojiIconTheme;

impl IconTheme for EmojiIconTheme {
    fn load_icon(&mut self, _: &File, _: impl Write) -> Result<(), io::Error> {
        Ok(())
    }

    fn get_icon(&self, file: &File) -> (&'static str, Option<Color>) {
        let icon = match &file.kind {
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
        };

        (icon, None)
    }
}

#[derive(Debug, Default)]
pub struct JetBrainsIconTheme {
    unknown: bool,
    directory: bool,
    rust: bool,
    toml: bool,
}

impl JetBrainsIconTheme {
    pub fn get_id(&self, file: &File) -> u8 {
        match &file.kind {
            FileKind::Directory(_) => 1,
            FileKind::Regular(extension) => *extension as u8 + 1,
            _ => FileExtension::Unknown as u8 + 1,
        }
    }
}

impl IconTheme for JetBrainsIconTheme {
    fn load_icon(&mut self, file: &File, stdout: impl Write) -> Result<(), io::Error> {
        let id = self.get_id(file);
        let icon = match &file.kind {
            FileKind::Directory(_) if !self.directory => {
                self.directory = true;

                include_bytes!("../assets/jetbrains_icons/folder.b64").as_slice()
            }
            FileKind::Regular(extension) => match extension {
                FileExtension::Rust if !self.rust => {
                    self.rust = true;

                    include_bytes!("../assets/jetbrains_icons/rust.b64").as_slice()
                }
                FileExtension::Toml if !self.toml => {
                    self.toml = true;

                    include_bytes!("../assets/jetbrains_icons/toml.b64").as_slice()
                }
                _ if !self.unknown => {
                    self.unknown = true;

                    include_bytes!("../assets/jetbrains_icons/anyType.b64").as_slice()
                }
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };

        load_icon(id, icon, stdout)
    }

    fn get_icon(&self, file: &File) -> (&'static str, Option<Color>) {
        let id = self.get_id(file);

        ("\u{10EEEE}\u{10EEEE}", Some(Color::Rgb(0, 0, id)))
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

    write!(writer, "\x1b_Ga=p,U=1,i={id},c=2,r=1\x1b\\")?;

    Ok(())
}
