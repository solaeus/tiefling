use std::{fmt::Debug, io, ops::Range, path::PathBuf};

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

    pub fn icon_id(&self) -> Option<IconId> {
        let name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match &self.kind {
            FileKind::Directory(directory) => {
                let (closed, open) = match name.as_str() {
                    "admin" => (IconId::FOLDER_ADMIN, IconId::FOLDER_ADMIN_OPEN),
                    "animation" | "animations" => {
                        (IconId::FOLDER_ANIMATION, IconId::FOLDER_ANIMATION_OPEN)
                    }
                    "asset" | "assets" => (IconId::FOLDER_ASSETS, IconId::FOLDER_ASSETS_OPEN),
                    "audio" | "sound" | "sounds" => {
                        (IconId::FOLDER_AUDIO, IconId::FOLDER_AUDIO_OPEN)
                    }
                    "auth" | "authentication" => (IconId::FOLDER_AUTH, IconId::FOLDER_AUTH_OPEN),
                    "bench" | "benchmark" | "benchmarks" => {
                        (IconId::FOLDER_BENCHMARK, IconId::FOLDER_BENCHMARK_OPEN)
                    }
                    "bin" => (IconId::FOLDER_BIN, IconId::FOLDER_BIN_OPEN),
                    "builder" | "builders" => (IconId::FOLDER_BUILDER, IconId::FOLDER_BUILDER_OPEN),
                    "camera" | "cameras" => (IconId::FOLDER_CAMERA, IconId::FOLDER_CAMERA_OPEN),
                    ".changeset" | "changeset" | "changesets" => {
                        (IconId::FOLDER_CHANGESETS, IconId::FOLDER_CHANGESETS_OPEN)
                    }
                    "client" => (IconId::FOLDER_CLIENT, IconId::FOLDER_CLIENT_OPEN),
                    "command" | "commands" => {
                        (IconId::FOLDER_COMMANDS, IconId::FOLDER_COMMANDS_OPEN)
                    }
                    "component" | "components" => {
                        (IconId::FOLDER_COMPONENT, IconId::FOLDER_COMPONENT_OPEN)
                    }
                    ".config" | "config" | "conf" => {
                        (IconId::FOLDER_CONFIG, IconId::FOLDER_CONFIG_OPEN)
                    }
                    "connection" | "connections" => {
                        (IconId::FOLDER_CONNECTION, IconId::FOLDER_CONNECTION_OPEN)
                    }
                    "constant" | "constants" => {
                        (IconId::FOLDER_CONSTANT, IconId::FOLDER_CONSTANT_OPEN)
                    }
                    "content" | "contents" => (IconId::FOLDER_CONTENT, IconId::FOLDER_CONTENT_OPEN),
                    "context" | "contexts" => (IconId::FOLDER_CONTEXT, IconId::FOLDER_CONTEXT_OPEN),
                    "coverage" => (IconId::FOLDER_COVERAGE, IconId::FOLDER_COVERAGE_OPEN),
                    "database" | "db" => (IconId::FOLDER_DATABASE, IconId::FOLDER_DATABASE_OPEN),
                    "dist" | "build" | "out" => (IconId::FOLDER_DIST, IconId::FOLDER_DIST_OPEN),
                    "doc" | "docs" | "documentation" => {
                        (IconId::FOLDER_DOCS, IconId::FOLDER_DOCS_OPEN)
                    }
                    "effect" | "effects" => (IconId::FOLDER_EFFECTS, IconId::FOLDER_EFFECTS_OPEN),
                    "error" | "errors" => (IconId::FOLDER_ERROR, IconId::FOLDER_ERROR_OPEN),
                    "event" | "events" => (IconId::FOLDER_EVENT, IconId::FOLDER_EVENT_OPEN),
                    "font" | "fonts" => (IconId::FOLDER_FONTS, IconId::FOLDER_FONTS_OPEN),
                    "function" | "functions" => {
                        (IconId::FOLDER_FUNCTION, IconId::FOLDER_FUNCTION_OPEN)
                    }
                    ".github" => (IconId::FOLDER_GITHUB, IconId::FOLDER_GITHUB_OPEN),
                    "hook" | "hooks" => (IconId::FOLDER_HOOKS, IconId::FOLDER_HOOKS_OPEN),
                    "image" | "images" | "img" => (IconId::FOLDER_IMAGE, IconId::FOLDER_IMAGE_OPEN),
                    "input" | "inputs" => (IconId::FOLDER_INPUT, IconId::FOLDER_INPUT_OPEN),
                    "javascript" | "js" => {
                        (IconId::FOLDER_JAVASCRIPT, IconId::FOLDER_JAVASCRIPT_OPEN)
                    }
                    "json" => (IconId::FOLDER_JSON, IconId::FOLDER_JSON_OPEN),
                    "layout" | "layouts" => (IconId::FOLDER_LAYOUT, IconId::FOLDER_LAYOUT_OPEN),
                    "lib" | "libs" | "library" => (IconId::FOLDER_LIB, IconId::FOLDER_LIB_OPEN),
                    "luau" => (IconId::FOLDER_LUAU, IconId::FOLDER_LUAU_OPEN),
                    ".lune" | "lune" => (IconId::FOLDER_LUNE, IconId::FOLDER_LUNE_OPEN),
                    "marketing" => (IconId::FOLDER_MARKETING, IconId::FOLDER_MARKETING_OPEN),
                    "middleware" | "middlewares" => {
                        (IconId::FOLDER_MIDDLEWARE, IconId::FOLDER_MIDDLEWARE_OPEN)
                    }
                    "model" | "models" => (IconId::FOLDER_MODEL, IconId::FOLDER_MODEL_OPEN),
                    "module" | "modules" => (IconId::FOLDER_MODULE, IconId::FOLDER_MODULE_OPEN),
                    "node_modules" => (IconId::FOLDER_NODE, IconId::FOLDER_NODE_OPEN),
                    ".nuxt" => (IconId::FOLDER_NUXT, IconId::FOLDER_NUXT_OPEN),
                    "package" | "packages" => (IconId::FOLDER_PACKAGE, IconId::FOLDER_PACKAGE_OPEN),
                    "page" | "pages" => (IconId::FOLDER_PAGE, IconId::FOLDER_PAGE_OPEN),
                    "provider" | "providers" => {
                        (IconId::FOLDER_PROVIDER, IconId::FOLDER_PROVIDER_OPEN)
                    }
                    "roblox" => (IconId::FOLDER_ROBLOX, IconId::FOLDER_ROBLOX_OPEN),
                    "route" | "routes" => (IconId::FOLDER_ROUTES, IconId::FOLDER_ROUTES_OPEN),
                    "script" | "scripts" => (IconId::FOLDER_SCRIPT, IconId::FOLDER_SCRIPT_OPEN),
                    "server" => (IconId::FOLDER_SERVER, IconId::FOLDER_SERVER_OPEN),
                    "service" | "services" => (IconId::FOLDER_SERVICE, IconId::FOLDER_SERVICE_OPEN),
                    "src" | "source" | "sources" => {
                        (IconId::FOLDER_SOURCE, IconId::FOLDER_SOURCE_OPEN)
                    }
                    ".storybook" | "storybook" => {
                        (IconId::FOLDER_STORYBOOK, IconId::FOLDER_STORYBOOK_OPEN)
                    }
                    "style" | "styles" => (IconId::FOLDER_STYLES, IconId::FOLDER_STYLES_OPEN),
                    "svg" | "svgs" => (IconId::FOLDER_SVG, IconId::FOLDER_SVG_OPEN),
                    "temp" | "tmp" => (IconId::FOLDER_TEMP, IconId::FOLDER_TEMP_OPEN),
                    "template" | "templates" => {
                        (IconId::FOLDER_TEMPLATE, IconId::FOLDER_TEMPLATE_OPEN)
                    }
                    "test" | "tests" | "__tests__" | "spec" | "specs" => {
                        (IconId::FOLDER_TEST, IconId::FOLDER_TEST_OPEN)
                    }
                    "type" | "types" => (IconId::FOLDER_TYPES, IconId::FOLDER_TYPES_OPEN),
                    "typescript" | "ts" => {
                        (IconId::FOLDER_TYPESCRIPT, IconId::FOLDER_TYPESCRIPT_OPEN)
                    }
                    "util" | "utils" | "utilities" => {
                        (IconId::FOLDER_UTIL, IconId::FOLDER_UTIL_OPEN)
                    }
                    "video" | "videos" => (IconId::FOLDER_VIDEO, IconId::FOLDER_VIDEO_OPEN),
                    ".vscode" => (IconId::FOLDER_VSCODE, IconId::FOLDER_VSCODE_OPEN),
                    "web" => (IconId::FOLDER_WEB, IconId::FOLDER_WEB_OPEN),
                    ".yarn" => (IconId::FOLDER_YARN, IconId::FOLDER_YARN_OPEN),
                    _ => (IconId::FOLDER, IconId::FOLDER_OPEN),
                };

                Some(if directory.expanded { open } else { closed })
            }
            FileKind::Regular => {
                let extension = self
                    .path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                let icon_id = match name.as_str() {
                    "cargo.toml"
                    | "rustfmt.toml"
                    | ".rustfmt.toml"
                    | "rust-toolchain"
                    | "rust-toolchain.toml" => IconId::RUST_CONFIG,
                    "package.json" => IconId::PACKAGE_CONFIG,
                    "package-lock.json" => IconId::PACKAGE_LOCK,
                    "npm-shrinkwrap.json" => IconId::NPM_LOCK,
                    ".npmrc" => IconId::NPM,
                    ".node-version" | ".nvmrc" => IconId::NODE,
                    "yarn.lock" => IconId::YARN_LOCK,
                    ".yarnrc" | ".yarnrc.yml" => IconId::YARN,
                    "bun.lock" | "bun.lockb" => IconId::BUN_LOCK,
                    "bunfig.toml" => IconId::BUN,
                    "go.mod" | "go.sum" => IconId::GO_MOD,
                    "wally.toml" => IconId::WALLY,
                    "wally.lock" => IconId::WALLY_LOCK,
                    ".luaurc" => IconId::LUAU_CONFIG,
                    "stylua.toml" | ".stylua.toml" | "selene.toml" | ".selene.toml" => {
                        IconId::LUA_CONFIG
                    }
                    "tsconfig.json" => IconId::TYPESCRIPT_CONFIG,
                    "jsconfig.json" => IconId::JAVASCRIPT_CONFIG,
                    "eslint.config.js" | ".eslintrc" | ".eslintrc.js" | ".eslintrc.json" => {
                        IconId::ESLINT
                    }
                    "vite.config.js" | "vite.config.ts" => IconId::VITE,
                    "nuxt.config.js" | "nuxt.config.ts" => IconId::NUXT,
                    "next.config.js" | "next.config.mjs" | "next.config.ts" => IconId::NEXT,
                    "svelte.config.js" => IconId::SVELTE,
                    "astro.config.js" | "astro.config.mjs" | "astro.config.ts" => {
                        IconId::ASTRO_CONFIG
                    }
                    "tailwind.config.js" | "tailwind.config.ts" => IconId::TAILWIND,
                    "drizzle.config.js" | "drizzle.config.ts" => IconId::DRIZZLE_ORM,
                    "dockerfile" | ".dockerignore" | "docker-compose.yml" | "compose.yml" => {
                        IconId::DOCKER
                    }
                    "makefile" | "gnumakefile" => IconId::MAKEFILE,
                    "justfile" | ".justfile" => IconId::JUST,
                    ".gitignore" | ".gitattributes" | ".gitmodules" | ".gitconfig" => IconId::GIT,
                    "changelog" | "changelog.md" => IconId::CHANGELOG,
                    "readme" | "readme.md" => IconId::README,
                    "license" | "license.md" | "licence" | "copying" => IconId::LICENSE,
                    "code_of_conduct" | "code_of_conduct.md" => IconId::CODE_OF_CONDUCT,
                    "codeowners" => IconId::CODEOWNERS,
                    "security" | "security.md" => IconId::SECURITY,
                    "todo" | "todo.md" => IconId::TODO,
                    "project.godot" => IconId::GODOT,
                    _ => match extension.as_str() {
                        "asm" | "s" => IconId::ASSEMBLY,
                        "astro" => IconId::ASTRO,
                        "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "opus" | "mid"
                        | "midi" => IconId::AUDIO,
                        "bin" | "exe" | "dll" | "so" | "o" | "a" | "obj" | "dat" => IconId::BINARY,
                        "c" => IconId::C,
                        "h" => IconId::C_HEADER,
                        "cpp" | "cc" | "cxx" | "c++" => IconId::CPP,
                        "hpp" | "hh" | "hxx" | "h++" => IconId::CPP_HEADER,
                        "cs" => IconId::CS,
                        "css" => IconId::CSS,
                        "scss" => IconId::SCSS,
                        "pcss" | "postcss" => IconId::PCSS,
                        "csv" | "tsv" => IconId::CSV,
                        "dart" => IconId::DART,
                        "db" | "sql" | "sqlite" | "sqlite3" => IconId::DATABASE,
                        "ttf" | "otf" | "woff" | "woff2" | "eot" => IconId::FONT,
                        "f" | "for" | "f77" => IconId::FORTRAN_FIXED,
                        "f90" | "f95" | "f03" | "f08" => IconId::FORTRAN,
                        "gleam" => IconId::GLEAM,
                        "go" => IconId::GO,
                        "gd" | "tscn" => IconId::GODOT,
                        "tres" | "res" | "import" => IconId::GODOT_ASSETS,
                        "hcl" => IconId::HCL,
                        "html" | "htm" | "xhtml" => IconId::HTML,
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "tiff"
                        | "avif" => IconId::IMAGE,
                        "java" | "class" | "jar" => IconId::JAVA,
                        "js" | "cjs" | "mjs" => IconId::JAVASCRIPT,
                        "jsx" => IconId::REACT,
                        "ts" | "mts" | "cts" => IconId::TYPESCRIPT,
                        "tsx" => IconId::REACT_TYPESCRIPT,
                        "json" | "jsonc" | "json5" => IconId::JSON,
                        "jl" => IconId::JULIA,
                        "key" | "pem" | "pub" | "crt" | "cert" | "cer" | "p12" | "pfx" | "asc"
                        | "gpg" => IconId::KEY,
                        "kt" | "kts" => IconId::KOTLIN,
                        "tex" | "sty" | "cls" => IconId::LATEX,
                        "lock" => IconId::LOCK,
                        "lua" => IconId::LUA,
                        "luau" => IconId::LUAU,
                        "md" | "markdown" => IconId::MARKDOWN,
                        "mdx" => IconId::MARKDOWNX,
                        "nim" | "nims" | "nimble" => IconId::NIM,
                        "nix" => IconId::NIX,
                        "odin" => IconId::ODIN,
                        "pdf" => IconId::PDF,
                        "pl" | "pm" | "perl" => IconId::PERL,
                        "php" => IconId::PHP,
                        "ps1" | "psm1" | "psd1" => IconId::POWERSHELL,
                        "py" | "pyw" | "pyi" => IconId::PYTHON,
                        "rbxl" | "rbxlx" => IconId::ROBLOX,
                        "rbxm" | "rbxmx" => IconId::ROBLOX_MODEL,
                        "rb" | "erb" | "gemspec" => IconId::RUBY,
                        "rs" | "rust" => IconId::RUST,
                        "scala" | "sc" | "sbt" => IconId::SCALA,
                        "sh" | "bash" | "zsh" | "fish" | "ksh" => IconId::SHELL,
                        "svelte" => IconId::SVELTE,
                        "svg" => IconId::SVG,
                        "swift" => IconId::SWIFT,
                        "tf" | "tfvars" => IconId::TERRAFORM,
                        "txt" | "text" | "log" => IconId::TEXT,
                        "toml" => IconId::TOML,
                        "cfg" | "conf" | "ini" | "config" | "editorconfig" | "env" => {
                            IconId::CONFIG
                        }
                        "mp4" | "mkv" | "mov" | "avi" | "webm" | "flv" | "wmv" | "m4v" => {
                            IconId::VIDEO
                        }
                        "sln" | "csproj" | "vbproj" | "fsproj" => IconId::VISUAL_STUDIO,
                        "code-workspace" => IconId::VSCODE,
                        "vue" => IconId::VUE,
                        "wasm" | "wat" => IconId::WEB_ASSEMBLY,
                        "xml" | "plist" | "xsd" | "xsl" => IconId::XML,
                        "yaml" | "yml" => IconId::YAML,
                        "zig" | "zon" => IconId::ZIG,
                        "zip" | "tar" | "gz" | "tgz" | "rar" | "7z" | "xz" | "bz2" | "zst" => {
                            IconId::ZIP
                        }
                        _ => {
                            if [".d.ts", ".d.mts", ".d.cts"]
                                .iter()
                                .any(|suffix| name.ends_with(suffix))
                            {
                                IconId::TYPESCRIPT_DEF
                            } else if name.ends_with(".d.luau") {
                                IconId::LUAU_DEF
                            } else if name.ends_with(".project.json") {
                                IconId::ROBLOX_CONFIG
                            } else if [
                                ".stories.ts",
                                ".stories.tsx",
                                ".stories.js",
                                ".stories.jsx",
                                ".stories.mdx",
                            ]
                            .iter()
                            .any(|suffix| name.ends_with(suffix))
                            {
                                IconId::STORYBOOK
                            } else if self
                                .path
                                .parent()
                                .and_then(|parent| {
                                    parent.file_name().and_then(|file_name| file_name.to_str())
                                })
                                .is_some_and(|parent| parent.eq_ignore_ascii_case("workflows"))
                                && (extension == "yml" || extension == "yaml")
                            {
                                IconId::WORKFLOW
                            } else {
                                IconId::FILE
                            }
                        }
                    },
                };

                Some(icon_id)
            }
            FileKind::Symlink(_) => None,
        }
    }

    pub fn linked_path(&self) -> Option<&PathBuf> {
        match &self.kind {
            FileKind::Symlink(linked_path) => Some(linked_path),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum FileKind {
    Directory(Directory),
    Regular,
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
