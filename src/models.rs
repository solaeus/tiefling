use std::{
    fmt::Debug,
    io::{self, Write, stdout},
    num::NonZero,
    ops::Range,
    path::{Path, PathBuf},
};

use ratatui::widgets::ScrollbarState;

#[derive(Debug)]
pub struct Files {
    /// All files known to the program, indexed by their `FileId`. This list is append-only; files
    /// are never removed from it.
    files: Vec<File>,

    /// Concatenated list of IDs for each directory's contents.
    children: Vec<FileId>,

    /// Authoritative list of which files are shown to the user.
    visible: Vec<FileId>,

    /// Position of the cursor in the `visible` list of files.
    cursor: usize,

    scrollbar_state: ScrollbarState,
}

impl Files {
    pub fn new(root: PathBuf) -> Result<Self, io::Error> {
        let mut files = Files {
            files: Vec::new(),
            children: Vec::new(),
            visible: Vec::new(),
            cursor: 0,
            scrollbar_state: ScrollbarState::new(0),
        };

        files.open_root(root).unwrap();

        Ok(files)
    }

    pub fn visible(&self) -> &Vec<FileId> {
        &self.visible
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn scrollbar_state_mut(&mut self) -> &mut ScrollbarState {
        &mut self.scrollbar_state
    }

    pub fn open_root(&mut self, path: PathBuf) -> Result<(), io::Error> {
        self.files.clear();
        self.children.clear();
        self.visible.clear();
        self.cursor = 0;
        self.open(path, 0)?;
        self.visible.push(FileId::ROOT);

        Ok(())
    }

    fn open(&mut self, path: PathBuf, depth: u8) -> Result<FileId, io::Error> {
        let kind = if path.is_symlink() {
            FileKind::Symlink(path.read_link()?)
        } else if path.is_dir() {
            FileKind::Directory(Directory::default())
        } else {
            FileKind::Regular(FileExtension::from_path(&path))
        };
        let file = File::new(path, kind, depth);
        let file_id = FileId(self.files.len() as u32);

        self.files.push(file);

        Ok(file_id)
    }

    pub fn get_file(&self, id: &FileId) -> &File {
        &self.files[id.0 as usize]
    }

    pub fn toggle_file_under_cursor(&mut self) -> Result<(), io::Error> {
        self.toggle_file(self.visible[self.cursor], self.cursor, false)?;
        self.update_scrollbar();

        Ok(())
    }

    fn toggle_file(
        &mut self,
        file_id: FileId,
        visible_index: usize,
        reexpand: bool,
    ) -> Result<usize, io::Error> {
        let file = &mut self.files[file_id.0 as usize];
        let FileKind::Directory(Directory {
            children,
            expanded,
            previously_expanded,
        }) = &mut file.kind
        else {
            return Ok(0);
        };
        let next = visible_index + 1;

        if *expanded && !reexpand {
            *expanded = false;

            let child_count = children.length();
            let depth = file.depth;
            let last_nested_child_index = self.visible[next..]
                .iter()
                .position(|file_id| self.files[file_id.0 as usize].depth <= depth)
                .map_or(self.visible.len(), |offset| next + offset);

            self.visible.drain(next..last_nested_child_index);

            return Ok(child_count);
        }

        if (*previously_expanded && !reexpand) || (*expanded && reexpand) {
            *expanded = true;
            *previously_expanded = true;

            self.visible
                .extend_from_slice(&self.children[children.as_index_range()]);
            self.visible[next..].rotate_right(children.length());

            let mut child_visible_index = next;
            let mut expansion_count = children.length();

            for children_index in children.as_index_range() {
                let child_id = self.children[children_index];
                let expanded = self.toggle_file(child_id, child_visible_index, true)?;

                child_visible_index += expanded + 1;
                expansion_count += expanded;
            }

            return Ok(expansion_count);
        }

        if reexpand {
            return Ok(0);
        }

        let child_depth = file.depth + 1;
        let children_start = self.children.len() as u32;

        for read_file in file.path.read_dir()? {
            let path = read_file?.path();
            let child_id = self.open(path, child_depth)?;

            self.children.push(child_id);
            self.visible.push(child_id);
        }

        let children_end = self.children.len() as u32;
        let children = FileChildren::new(children_start, children_end);
        let file = &mut self.files[file_id.0 as usize];
        file.kind = FileKind::Directory(Directory {
            expanded: true,
            previously_expanded: true,
            children,
        });

        self.visible[next..].rotate_right(children.length());

        Ok(children.length())
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor == self.visible.len() - 1 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor == 0 {
            self.cursor = self.visible.len() - 1;
        } else {
            self.cursor -= 1;
        }
    }

    pub fn update_scrollbar(&mut self) {
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.visible.len())
            .position(self.cursor);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileId(u32);

impl FileId {
    const ROOT: Self = Self(0);
}

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub kind: FileKind,
    pub depth: u8,
    pub marked: bool,
}

impl File {
    pub fn new(path: PathBuf, kind: FileKind, depth: u8) -> Self {
        Self {
            path,
            kind,
            depth,
            marked: false,
        }
    }

    pub fn icon_id(&self) -> IconId {
        match self.kind {
            FileKind::Directory(Directory { expanded: true, .. }) => IconId::EXPANDED,
            FileKind::Directory(Directory {
                expanded: false, ..
            }) => IconId::COLLAPSED,
            FileKind::Regular(extension) => IconId::from_extension(extension),
            FileKind::Symlink(_) => IconId::SYMLINK,
        }
    }
}

#[derive(Debug)]
pub enum FileKind {
    Directory(Directory),
    Regular(FileExtension),
    Symlink(PathBuf),
}

#[derive(Debug, Default)]
pub struct Directory {
    children: FileChildren,
    expanded: bool,
    previously_expanded: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FileChildren {
    start: u32,
    end: u32,
}

impl FileChildren {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn as_index_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn length(&self) -> usize {
        (self.end - self.start) as usize
    }
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

#[derive(Debug, Default)]
pub enum Icons {
    #[default]
    JetBrains,
}

impl Icons {
    pub fn load_icons(&self) -> Result<(), io::Error> {
        match self {
            Icons::JetBrains => JetBrainsIconTheme::load_icons(),
        }
    }
}

pub trait IconTheme: Debug + Default {
    const COLLAPSED_ICON: &'static [u8];
    const EXPANDED_ICON: &'static [u8];
    const EXTENSION_ICONS: &'static [(IconId, &'static [u8])];

    fn load_icons() -> Result<(), io::Error> {
        let mut stdout = stdout().lock();

        load_kitty_icon(IconId::COLLAPSED, Self::COLLAPSED_ICON, &mut stdout)?;
        load_kitty_icon(IconId::EXPANDED, Self::EXPANDED_ICON, &mut stdout)?;

        for (icon_id, icon_data) in Self::EXTENSION_ICONS {
            load_kitty_icon(*icon_id, icon_data, &mut stdout)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct JetBrainsIconTheme;

impl IconTheme for JetBrainsIconTheme {
    const COLLAPSED_ICON: &'static [u8] = include_bytes!("../assets/jetbrains_icons/folder.b64");
    const EXPANDED_ICON: &'static [u8] = include_bytes!("../assets/jetbrains_icons/folder.b64");
    const EXTENSION_ICONS: &'static [(IconId, &'static [u8])] = &[
        (
            IconId::from_extension(FileExtension::Unknown),
            include_bytes!("../assets/jetbrains_icons/anyType.b64"),
        ),
        (
            IconId::from_extension(FileExtension::Rust),
            include_bytes!("../assets/jetbrains_icons/rust.b64"),
        ),
    ];
}

#[derive(Debug, Clone, Copy)]
pub struct IconId(pub NonZero<u8>);

#[expect(clippy::disallowed_methods)]
impl IconId {
    const COLLAPSED: Self = Self(NonZero::new(1).unwrap());
    const EXPANDED: Self = Self(NonZero::new(2).unwrap());
    const SYMLINK: Self = Self(NonZero::new(3).unwrap());
}

impl IconId {
    pub const fn from_extension(extension: FileExtension) -> Self {
        let inner = extension as u8 + 4;

        // SAFETY: `id_inner` is guaranteed to be non-zero by the addition expression above.
        unsafe { IconId(NonZero::new(inner).unwrap_unchecked()) }
    }

    pub const fn inner(self) -> u8 {
        self.0.get()
    }
}

/// https://sw.kovidgoyal.net/kitty/graphics-protocol/#a-minimal-example
fn load_kitty_icon(
    IconId(id): IconId,
    icon: &[u8],
    mut writer: impl Write,
) -> Result<(), io::Error> {
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
