use std::{
    fmt::Debug,
    io,
    ops::Range,
    path::{Path, PathBuf},
};

use crate::icons::IconId;

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
}

impl Files {
    pub fn new(root: PathBuf) -> Result<Self, io::Error> {
        let mut files = Files {
            files: Vec::new(),
            children: Vec::new(),
            visible: Vec::new(),
            cursor: 0,
        };

        files.open_root(root)?;

        Ok(files)
    }

    pub fn visible(&self) -> &Vec<FileId> {
        &self.visible
    }

    pub fn cursor(&self) -> usize {
        self.cursor
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

    pub fn toggle_file_under_cursor_marked(&mut self) {
        let file_id = self.visible[self.cursor];
        let file = &mut self.files[file_id.0 as usize];

        file.marked = !file.marked;
    }

    pub fn expand_directory_under_cursor(&mut self) -> Result<(), io::Error> {
        self.expand_directory(self.visible[self.cursor], self.cursor, false)?;

        Ok(())
    }

    fn expand_directory(
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
                let expanded = self.expand_directory(child_id, child_visible_index, true)?;

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

        let mut entries = file.path.read_dir()?.flatten().collect::<Vec<_>>();

        entries.sort_by(sort_entries);

        for entry in entries {
            let file_id = self.open(entry.path(), child_depth)?;

            self.children.push(file_id);
            self.visible.push(file_id);
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

    pub fn icon_id_and_linked_path(&self) -> (IconId, Option<&PathBuf>) {
        match &self.kind {
            FileKind::Directory(Directory { expanded: true, .. }) => (IconId::EXPANDED, None),
            FileKind::Directory(Directory {
                expanded: false, ..
            }) => (IconId::COLLAPSED, None),
            FileKind::Regular(extension) => (IconId::from_extension(*extension), None),
            FileKind::Symlink(linked_path) => (IconId::SYMLINK, Some(linked_path)),
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
    ActionScript,
    Angular,
    Application,
    Archive,
    Beam,
    Biome,
    Bun,
    C,
    C3,
    C3Interface,
    C3Library,
    C3Test,
    Cargo,
    CargoLock,
    CHeader,
    Clojure,
    CMake,
    Config,
    Cpp,
    CSharp,
    CsHtml,
    CsProj,
    Css,
    Csv,
    Cuda,
    CudaHeader,
    Dart,
    Database,
    Docker,
    Dune,
    EditorConfig,
    Eex,
    Elixir,
    Erb,
    Erlang,
    Eslint,
    Font,
    GitIgnore,
    Gleam,
    Go,
    GoMod,
    GoSum,
    GoWork,
    GraphQl,
    Handlebars,
    Haskell,
    Hcl,
    Html,
    Http,
    Image,
    Ino,
    Java,
    JavaScript,
    Json,
    JsTest,
    Jsx,
    JsxTest,
    Jupyter,
    Kotlin,
    KotlinScript,
    Less,
    Lock,
    Lua,
    Markdown,
    Mdx,
    Module,
    NodeJs,
    Npm,
    OCaml,
    OCamlInterface,
    Opam,
    OpenTofu,
    Php,
    Pnpm,
    PostCss,
    Prettier,
    ProjectProperties,
    Properties,
    Proto,
    Protobuf,
    Python,
    Rake,
    Rego,
    Ruby,
    Rust,
    Scala,
    Scss,
    Shell,
    Slim,
    Solution,
    Sql,
    Svelte,
    Swift,
    Tailwind,
    Terraform,
    Text,
    Toml,
    TsTest,
    Tsx,
    TsxTest,
    TypeScript,
    Vercel,
    Vite,
    VLang,
    Vue,
    Xml,
    Yaml,
    Yarn,
    Zig,
}

impl FileExtension {
    fn from_path(path: &Path) -> Self {
        let file_name = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();

        match file_name {
            "Cargo.toml" => return Self::Cargo,
            "Cargo.lock" => return Self::CargoLock,
            "go.mod" => return Self::GoMod,
            "go.sum" => return Self::GoSum,
            "go.work" => return Self::GoWork,
            "dune" | "dune-project" => return Self::Dune,
            "biome.json" | "biome.jsonc" => return Self::Biome,
            "bun.lockb" | "bunfig.toml" => return Self::Bun,
            "package.json" | "package-lock.json" | ".npmrc" => return Self::Npm,
            "pnpm-lock.yaml" | "pnpm-workspace.yaml" | ".pnpmfile.cjs" => return Self::Pnpm,
            "yarn.lock" | ".yarnrc" | ".yarnrc.yml" => return Self::Yarn,
            "gradle.properties" | "project.properties" => return Self::ProjectProperties,
            "vercel.json" | ".vercelignore" | "now.json" => return Self::Vercel,
            ".editorconfig" => return Self::EditorConfig,
            ".gitignore" => return Self::GitIgnore,
            ".nvmrc" | ".node-version" => return Self::NodeJs,
            _ => {}
        }

        if file_name == "Dockerfile"
            || file_name.starts_with("Dockerfile.")
            || file_name.ends_with(".dockerfile")
        {
            return Self::Docker;
        }

        if file_name.starts_with(".eslintrc") || file_name.starts_with("eslint.config.") {
            return Self::Eslint;
        }

        if file_name.starts_with(".prettier") || file_name.starts_with("prettier.config.") {
            return Self::Prettier;
        }

        if file_name.starts_with("tailwind.config.") {
            return Self::Tailwind;
        }

        if file_name.starts_with("vite.config.") {
            return Self::Vite;
        }

        if file_name.starts_with("postcss.config.") {
            return Self::PostCss;
        }

        if file_name.ends_with(".test.tsx") || file_name.ends_with(".spec.tsx") {
            return Self::TsxTest;
        }

        if file_name.ends_with(".test.ts") || file_name.ends_with(".spec.ts") {
            return Self::TsTest;
        }

        if file_name.ends_with(".test.jsx") || file_name.ends_with(".spec.jsx") {
            return Self::JsxTest;
        }

        if file_name.ends_with(".test.js") || file_name.ends_with(".spec.js") {
            return Self::JsTest;
        }

        if file_name.ends_with(".component.ts")
            || file_name.ends_with(".module.ts")
            || file_name.ends_with(".service.ts")
            || file_name.ends_with(".directive.ts")
            || file_name.ends_with(".pipe.ts")
            || file_name.ends_with(".guard.ts")
        {
            return Self::Angular;
        }

        if file_name.ends_with("_test.c3") {
            return Self::C3Test;
        }

        match path.extension().and_then(|extension| extension.to_str()) {
            Some("as") => Self::ActionScript,
            Some("zip" | "tar" | "gz" | "tgz" | "rar" | "7z" | "xz" | "bz2") => Self::Archive,
            Some("beam") => Self::Beam,
            Some("c") => Self::C,
            Some("c3") => Self::C3,
            Some("c3i") => Self::C3Interface,
            Some("c3l") => Self::C3Library,
            Some("h") => Self::CHeader,
            Some("clj" | "cljs" | "cljc" | "edn") => Self::Clojure,
            Some("cmake") => Self::CMake,
            Some("conf" | "ini" | "cfg") => Self::Config,
            Some("cpp" | "cc" | "cxx" | "hpp") => Self::Cpp,
            Some("cs") => Self::CSharp,
            Some("cshtml") => Self::CsHtml,
            Some("csproj") => Self::CsProj,
            Some("css") => Self::Css,
            Some("csv") => Self::Csv,
            Some("cu") => Self::Cuda,
            Some("cuh") => Self::CudaHeader,
            Some("dart") => Self::Dart,
            Some("db" | "sqlite" | "sqlite3") => Self::Database,
            Some("eex") => Self::Eex,
            Some("ex" | "exs") => Self::Elixir,
            Some("erb") => Self::Erb,
            Some("erl" | "hrl") => Self::Erlang,
            Some("ttf" | "otf" | "woff" | "woff2") => Self::Font,
            Some("gleam") => Self::Gleam,
            Some("go") => Self::Go,
            Some("graphql" | "gql") => Self::GraphQl,
            Some("hbs") => Self::Handlebars,
            Some("hs") => Self::Haskell,
            Some("hcl") => Self::Hcl,
            Some("html" | "htm") => Self::Html,
            Some("http") => Self::Http,
            Some("png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico") => Self::Image,
            Some("ino") => Self::Ino,
            Some("java") => Self::Java,
            Some("js" | "mjs" | "cjs") => Self::JavaScript,
            Some("json") => Self::Json,
            Some("jsx") => Self::Jsx,
            Some("ipynb") => Self::Jupyter,
            Some("kt") => Self::Kotlin,
            Some("kts") => Self::KotlinScript,
            Some("less") => Self::Less,
            Some("lock") => Self::Lock,
            Some("lua") => Self::Lua,
            Some("md" | "markdown") => Self::Markdown,
            Some("mdx") => Self::Mdx,
            Some("ml") => Self::OCaml,
            Some("mli") => Self::OCamlInterface,
            Some("opam") => Self::Opam,
            Some("php") => Self::Php,
            Some("properties") => Self::Properties,
            Some("proto") => Self::Proto,
            Some("py") => Self::Python,
            Some("rake") => Self::Rake,
            Some("rego") => Self::Rego,
            Some("rb") => Self::Ruby,
            Some("rs") => Self::Rust,
            Some("scala") => Self::Scala,
            Some("scss") => Self::Scss,
            Some("sh" | "bash" | "zsh") => Self::Shell,
            Some("slim") => Self::Slim,
            Some("sln") => Self::Solution,
            Some("sql") => Self::Sql,
            Some("svelte") => Self::Svelte,
            Some("swift") => Self::Swift,
            Some("tf") => Self::Terraform,
            Some("txt") => Self::Text,
            Some("toml") => Self::Toml,
            Some("tsx") => Self::Tsx,
            Some("ts") => Self::TypeScript,
            Some("v") => Self::VLang,
            Some("vue") => Self::Vue,
            Some("xml") => Self::Xml,
            Some("yaml" | "yml") => Self::Yaml,
            Some("zig") => Self::Zig,
            Some("exe" | "bin" | "app" | "dll" | "so" | "dylib" | "msi") => Self::Application,
            Some("iml") => Self::Module,
            Some("tofu") => Self::OpenTofu,
            Some("pb") => Self::Protobuf,
            Some("pcss") => Self::PostCss,
            _ => Self::Unknown,
        }
    }
}

fn sort_entries(left: &std::fs::DirEntry, right: &std::fs::DirEntry) -> std::cmp::Ordering {
    let Ok(left_type) = left.file_type() else {
        return std::cmp::Ordering::Equal;
    };
    let Ok(right_type) = right.file_type() else {
        return std::cmp::Ordering::Equal;
    };
    let type_comparison = left_type.is_dir().cmp(&right_type.is_dir());

    if type_comparison != std::cmp::Ordering::Equal {
        return type_comparison.reverse();
    }

    left.file_name().cmp(&right.file_name())
}
