mod charmed;

use std::{
    fmt::Debug,
    io::{self, Write},
    num::NonZero,
};

use crate::icons::charmed::CharmedIconTheme;

#[derive(Debug, Default)]
pub enum Icons {
    #[default]
    Charmed,
}

impl Icons {
    pub fn load_icons(&self, stdout: &mut impl Write) -> Result<(), io::Error> {
        match self {
            Icons::Charmed => CharmedIconTheme::load_icons(stdout),
        }
    }
}

pub trait IconTheme: Debug + Default {
    const ICONS: &'static [(IconId, &'static [u8])];

    fn load_icons(stdout: &mut impl Write) -> Result<(), io::Error> {
        for (icon_id, icon_data) in Self::ICONS {
            load_kitty_icon(*icon_id, icon_data, stdout)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IconId(pub NonZero<u8>);

#[expect(clippy::disallowed_methods)]
impl IconId {
    pub const FILE: Self = Self(NonZero::<u8>::new(1).unwrap());
    pub const FOLDER: Self = Self(NonZero::<u8>::new(2).unwrap());
    pub const FOLDER_OPEN: Self = Self(NonZero::<u8>::new(2).unwrap());
    pub const ASSEMBLY: Self = Self(NonZero::<u8>::new(3).unwrap());
    pub const ASTRO_CONFIG: Self = Self(NonZero::<u8>::new(4).unwrap());
    pub const ASTRO: Self = Self(NonZero::<u8>::new(5).unwrap());
    pub const AUDIO: Self = Self(NonZero::<u8>::new(6).unwrap());
    pub const BINARY: Self = Self(NonZero::<u8>::new(7).unwrap());
    pub const BUN_LOCK: Self = Self(NonZero::<u8>::new(8).unwrap());
    pub const BUN: Self = Self(NonZero::<u8>::new(9).unwrap());
    pub const C: Self = Self(NonZero::<u8>::new(10).unwrap());
    pub const C_HEADER: Self = Self(NonZero::<u8>::new(11).unwrap());
    pub const CHANGELOG: Self = Self(NonZero::<u8>::new(12).unwrap());
    pub const CODE_OF_CONDUCT: Self = Self(NonZero::<u8>::new(13).unwrap());
    pub const CODEOWNERS: Self = Self(NonZero::<u8>::new(14).unwrap());
    pub const CONFIG: Self = Self(NonZero::<u8>::new(15).unwrap());
    pub const CPP_HEADER: Self = Self(NonZero::<u8>::new(16).unwrap());
    pub const CPP: Self = Self(NonZero::<u8>::new(17).unwrap());
    pub const CS: Self = Self(NonZero::<u8>::new(18).unwrap());
    pub const CSS: Self = Self(NonZero::<u8>::new(19).unwrap());
    pub const CSS3: Self = Self(NonZero::<u8>::new(20).unwrap());
    pub const CSV: Self = Self(NonZero::<u8>::new(21).unwrap());
    pub const DART: Self = Self(NonZero::<u8>::new(22).unwrap());
    pub const DATABASE: Self = Self(NonZero::<u8>::new(23).unwrap());
    pub const DOCKER: Self = Self(NonZero::<u8>::new(24).unwrap());
    pub const DRIZZLE_ORM: Self = Self(NonZero::<u8>::new(25).unwrap());
    pub const ESLINT: Self = Self(NonZero::<u8>::new(26).unwrap());
    pub const EVENT: Self = Self(NonZero::<u8>::new(27).unwrap());
    pub const FOLDER_ADMIN: Self = Self(NonZero::<u8>::new(28).unwrap());
    pub const FOLDER_ADMIN_OPEN: Self = Self(NonZero::<u8>::new(29).unwrap());
    pub const FOLDER_ANIMATION: Self = Self(NonZero::<u8>::new(30).unwrap());
    pub const FOLDER_ANIMATION_OPEN: Self = Self(NonZero::<u8>::new(31).unwrap());
    pub const FOLDER_ASSETS: Self = Self(NonZero::<u8>::new(32).unwrap());
    pub const FOLDER_ASSETS_OPEN: Self = Self(NonZero::<u8>::new(33).unwrap());
    pub const FOLDER_AUDIO: Self = Self(NonZero::<u8>::new(34).unwrap());
    pub const FOLDER_AUDIO_OPEN: Self = Self(NonZero::<u8>::new(35).unwrap());
    pub const FOLDER_AUTH: Self = Self(NonZero::<u8>::new(36).unwrap());
    pub const FOLDER_AUTH_OPEN: Self = Self(NonZero::<u8>::new(37).unwrap());
    pub const FOLDER_BENCHMARK: Self = Self(NonZero::<u8>::new(38).unwrap());
    pub const FOLDER_BENCHMARK_OPEN: Self = Self(NonZero::<u8>::new(39).unwrap());
    pub const FOLDER_BIN: Self = Self(NonZero::<u8>::new(40).unwrap());
    pub const FOLDER_BIN_OPEN: Self = Self(NonZero::<u8>::new(41).unwrap());
    pub const FOLDER_BUILDER: Self = Self(NonZero::<u8>::new(42).unwrap());
    pub const FOLDER_BUILDER_OPEN: Self = Self(NonZero::<u8>::new(43).unwrap());
    pub const FOLDER_CAMERA: Self = Self(NonZero::<u8>::new(44).unwrap());
    pub const FOLDER_CAMERA_OPEN: Self = Self(NonZero::<u8>::new(45).unwrap());
    pub const FOLDER_CHANGESETS: Self = Self(NonZero::<u8>::new(46).unwrap());
    pub const FOLDER_CHANGESETS_OPEN: Self = Self(NonZero::<u8>::new(47).unwrap());
    pub const FOLDER_CLIENT: Self = Self(NonZero::<u8>::new(48).unwrap());
    pub const FOLDER_CLIENT_OPEN: Self = Self(NonZero::<u8>::new(49).unwrap());
    pub const FOLDER_COMMANDS: Self = Self(NonZero::<u8>::new(50).unwrap());
    pub const FOLDER_COMMANDS_OPEN: Self = Self(NonZero::<u8>::new(51).unwrap());
    pub const FOLDER_COMPONENT: Self = Self(NonZero::<u8>::new(52).unwrap());
    pub const FOLDER_COMPONENT_OPEN: Self = Self(NonZero::<u8>::new(53).unwrap());
    pub const FOLDER_CONFIG: Self = Self(NonZero::<u8>::new(54).unwrap());
    pub const FOLDER_CONFIG_OPEN: Self = Self(NonZero::<u8>::new(55).unwrap());
    pub const FOLDER_CONNECTION: Self = Self(NonZero::<u8>::new(56).unwrap());
    pub const FOLDER_CONNECTION_OPEN: Self = Self(NonZero::<u8>::new(57).unwrap());
    pub const FOLDER_CONSTANT: Self = Self(NonZero::<u8>::new(58).unwrap());
    pub const FOLDER_CONSTANT_OPEN: Self = Self(NonZero::<u8>::new(59).unwrap());
    pub const FOLDER_CONTENT: Self = Self(NonZero::<u8>::new(60).unwrap());
    pub const FOLDER_CONTENT_OPEN: Self = Self(NonZero::<u8>::new(61).unwrap());
    pub const FOLDER_CONTEXT: Self = Self(NonZero::<u8>::new(62).unwrap());
    pub const FOLDER_CONTEXT_OPEN: Self = Self(NonZero::<u8>::new(63).unwrap());
    pub const FOLDER_COVERAGE: Self = Self(NonZero::<u8>::new(64).unwrap());
    pub const FOLDER_COVERAGE_OPEN: Self = Self(NonZero::<u8>::new(65).unwrap());
    pub const FOLDER_DATABASE: Self = Self(NonZero::<u8>::new(66).unwrap());
    pub const FOLDER_DATABASE_OPEN: Self = Self(NonZero::<u8>::new(67).unwrap());
    pub const FOLDER_DIST: Self = Self(NonZero::<u8>::new(68).unwrap());
    pub const FOLDER_DIST_OPEN: Self = Self(NonZero::<u8>::new(69).unwrap());
    pub const FOLDER_DOCS: Self = Self(NonZero::<u8>::new(70).unwrap());
    pub const FOLDER_DOCS_OPEN: Self = Self(NonZero::<u8>::new(71).unwrap());
    pub const FOLDER_EFFECTS: Self = Self(NonZero::<u8>::new(72).unwrap());
    pub const FOLDER_EFFECTS_OPEN: Self = Self(NonZero::<u8>::new(73).unwrap());
    pub const FOLDER_ERROR: Self = Self(NonZero::<u8>::new(74).unwrap());
    pub const FOLDER_ERROR_OPEN: Self = Self(NonZero::<u8>::new(75).unwrap());
    pub const FOLDER_EVENT: Self = Self(NonZero::<u8>::new(76).unwrap());
    pub const FOLDER_EVENT_OPEN: Self = Self(NonZero::<u8>::new(77).unwrap());
    pub const FOLDER_FONTS: Self = Self(NonZero::<u8>::new(78).unwrap());
    pub const FOLDER_FONTS_OPEN: Self = Self(NonZero::<u8>::new(79).unwrap());
    pub const FOLDER_FUNCTION: Self = Self(NonZero::<u8>::new(80).unwrap());
    pub const FOLDER_FUNCTION_OPEN: Self = Self(NonZero::<u8>::new(81).unwrap());
    pub const FOLDER_GITHUB: Self = Self(NonZero::<u8>::new(82).unwrap());
    pub const FOLDER_GITHUB_OPEN: Self = Self(NonZero::<u8>::new(83).unwrap());
    pub const FOLDER_HOOKS: Self = Self(NonZero::<u8>::new(84).unwrap());
    pub const FOLDER_HOOKS_OPEN: Self = Self(NonZero::<u8>::new(85).unwrap());
    pub const FOLDER_IMAGE: Self = Self(NonZero::<u8>::new(86).unwrap());
    pub const FOLDER_IMAGE_OPEN: Self = Self(NonZero::<u8>::new(87).unwrap());
    pub const FOLDER_INPUT: Self = Self(NonZero::<u8>::new(88).unwrap());
    pub const FOLDER_INPUT_OPEN: Self = Self(NonZero::<u8>::new(89).unwrap());
    pub const FOLDER_JAVASCRIPT: Self = Self(NonZero::<u8>::new(90).unwrap());
    pub const FOLDER_JAVASCRIPT_OPEN: Self = Self(NonZero::<u8>::new(91).unwrap());
    pub const FOLDER_JSON: Self = Self(NonZero::<u8>::new(92).unwrap());
    pub const FOLDER_JSON_OPEN: Self = Self(NonZero::<u8>::new(93).unwrap());
    pub const FOLDER_LAYOUT: Self = Self(NonZero::<u8>::new(94).unwrap());
    pub const FOLDER_LAYOUT_OPEN: Self = Self(NonZero::<u8>::new(95).unwrap());
    pub const FOLDER_LIB: Self = Self(NonZero::<u8>::new(96).unwrap());
    pub const FOLDER_LIB_OPEN: Self = Self(NonZero::<u8>::new(97).unwrap());
    pub const FOLDER_LUAU: Self = Self(NonZero::<u8>::new(98).unwrap());
    pub const FOLDER_LUAU_OPEN: Self = Self(NonZero::<u8>::new(99).unwrap());
    pub const FOLDER_LUNE: Self = Self(NonZero::<u8>::new(100).unwrap());
    pub const FOLDER_LUNE_OPEN: Self = Self(NonZero::<u8>::new(101).unwrap());
    pub const FOLDER_MARKETING: Self = Self(NonZero::<u8>::new(102).unwrap());
    pub const FOLDER_MARKETING_OPEN: Self = Self(NonZero::<u8>::new(103).unwrap());
    pub const FOLDER_MIDDLEWARE: Self = Self(NonZero::<u8>::new(104).unwrap());
    pub const FOLDER_MIDDLEWARE_OPEN: Self = Self(NonZero::<u8>::new(105).unwrap());
    pub const FOLDER_MODEL: Self = Self(NonZero::<u8>::new(106).unwrap());
    pub const FOLDER_MODEL_OPEN: Self = Self(NonZero::<u8>::new(107).unwrap());
    pub const FOLDER_MODULE: Self = Self(NonZero::<u8>::new(108).unwrap());
    pub const FOLDER_MODULE_OPEN: Self = Self(NonZero::<u8>::new(109).unwrap());
    pub const FOLDER_NODE: Self = Self(NonZero::<u8>::new(110).unwrap());
    pub const FOLDER_NODE_OPEN: Self = Self(NonZero::<u8>::new(111).unwrap());
    pub const FOLDER_NUXT: Self = Self(NonZero::<u8>::new(112).unwrap());
    pub const FOLDER_NUXT_OPEN: Self = Self(NonZero::<u8>::new(113).unwrap());
    pub const FOLDER_PACKAGE: Self = Self(NonZero::<u8>::new(114).unwrap());
    pub const FOLDER_PACKAGE_OPEN: Self = Self(NonZero::<u8>::new(115).unwrap());
    pub const FOLDER_PAGE: Self = Self(NonZero::<u8>::new(116).unwrap());
    pub const FOLDER_PAGE_OPEN: Self = Self(NonZero::<u8>::new(117).unwrap());
    pub const FOLDER_PROVIDER: Self = Self(NonZero::<u8>::new(118).unwrap());
    pub const FOLDER_PROVIDER_OPEN: Self = Self(NonZero::<u8>::new(119).unwrap());
    pub const FOLDER_ROBLOX: Self = Self(NonZero::<u8>::new(120).unwrap());
    pub const FOLDER_ROBLOX_OPEN: Self = Self(NonZero::<u8>::new(121).unwrap());
    pub const FOLDER_ROUTES: Self = Self(NonZero::<u8>::new(122).unwrap());
    pub const FOLDER_ROUTES_OPEN: Self = Self(NonZero::<u8>::new(123).unwrap());
    pub const FOLDER_SCRIPT: Self = Self(NonZero::<u8>::new(124).unwrap());
    pub const FOLDER_SCRIPT_OPEN: Self = Self(NonZero::<u8>::new(125).unwrap());
    pub const FOLDER_SERVER: Self = Self(NonZero::<u8>::new(126).unwrap());
    pub const FOLDER_SERVER_OPEN: Self = Self(NonZero::<u8>::new(127).unwrap());
    pub const FOLDER_SERVICE: Self = Self(NonZero::<u8>::new(128).unwrap());
    pub const FOLDER_SERVICE_OPEN: Self = Self(NonZero::<u8>::new(129).unwrap());
    pub const FOLDER_SOURCE: Self = Self(NonZero::<u8>::new(130).unwrap());
    pub const FOLDER_SOURCE_OPEN: Self = Self(NonZero::<u8>::new(131).unwrap());
    pub const FOLDER_STORYBOOK: Self = Self(NonZero::<u8>::new(132).unwrap());
    pub const FOLDER_STORYBOOK_OPEN: Self = Self(NonZero::<u8>::new(133).unwrap());
    pub const FOLDER_STYLES: Self = Self(NonZero::<u8>::new(134).unwrap());
    pub const FOLDER_STYLES_OPEN: Self = Self(NonZero::<u8>::new(135).unwrap());
    pub const FOLDER_SVG: Self = Self(NonZero::<u8>::new(136).unwrap());
    pub const FOLDER_SVG_OPEN: Self = Self(NonZero::<u8>::new(137).unwrap());
    pub const FOLDER_TEMP: Self = Self(NonZero::<u8>::new(138).unwrap());
    pub const FOLDER_TEMP_OPEN: Self = Self(NonZero::<u8>::new(139).unwrap());
    pub const FOLDER_TEMPLATE: Self = Self(NonZero::<u8>::new(140).unwrap());
    pub const FOLDER_TEMPLATE_OPEN: Self = Self(NonZero::<u8>::new(141).unwrap());
    pub const FOLDER_TEST: Self = Self(NonZero::<u8>::new(142).unwrap());
    pub const FOLDER_TEST_OPEN: Self = Self(NonZero::<u8>::new(143).unwrap());
    pub const FOLDER_TYPES: Self = Self(NonZero::<u8>::new(144).unwrap());
    pub const FOLDER_TYPES_OPEN: Self = Self(NonZero::<u8>::new(145).unwrap());
    pub const FOLDER_TYPESCRIPT: Self = Self(NonZero::<u8>::new(146).unwrap());
    pub const FOLDER_TYPESCRIPT_OPEN: Self = Self(NonZero::<u8>::new(147).unwrap());
    pub const FOLDER_UTIL: Self = Self(NonZero::<u8>::new(148).unwrap());
    pub const FOLDER_UTIL_OPEN: Self = Self(NonZero::<u8>::new(149).unwrap());
    pub const FOLDER_VIDEO: Self = Self(NonZero::<u8>::new(150).unwrap());
    pub const FOLDER_VIDEO_OPEN: Self = Self(NonZero::<u8>::new(151).unwrap());
    pub const FOLDER_VSCODE: Self = Self(NonZero::<u8>::new(152).unwrap());
    pub const FOLDER_VSCODE_OPEN: Self = Self(NonZero::<u8>::new(153).unwrap());
    pub const FOLDER_WEB: Self = Self(NonZero::<u8>::new(154).unwrap());
    pub const FOLDER_WEB_OPEN: Self = Self(NonZero::<u8>::new(155).unwrap());
    pub const FOLDER_YARN: Self = Self(NonZero::<u8>::new(156).unwrap());
    pub const FOLDER_YARN_OPEN: Self = Self(NonZero::<u8>::new(157).unwrap());
    pub const FONT: Self = Self(NonZero::<u8>::new(158).unwrap());
    pub const FORTRAN_FIXED: Self = Self(NonZero::<u8>::new(159).unwrap());
    pub const FORTRAN: Self = Self(NonZero::<u8>::new(160).unwrap());
    pub const GIT: Self = Self(NonZero::<u8>::new(161).unwrap());
    pub const GLEAM: Self = Self(NonZero::<u8>::new(162).unwrap());
    pub const GO_MOD: Self = Self(NonZero::<u8>::new(163).unwrap());
    pub const GO: Self = Self(NonZero::<u8>::new(164).unwrap());
    pub const GODOT_ASSETS: Self = Self(NonZero::<u8>::new(165).unwrap());
    pub const GODOT: Self = Self(NonZero::<u8>::new(166).unwrap());
    pub const HCL: Self = Self(NonZero::<u8>::new(167).unwrap());
    pub const HTML: Self = Self(NonZero::<u8>::new(168).unwrap());
    pub const IMAGE: Self = Self(NonZero::<u8>::new(169).unwrap());
    pub const JAVA: Self = Self(NonZero::<u8>::new(170).unwrap());
    pub const JAVASCRIPT_CONFIG: Self = Self(NonZero::<u8>::new(171).unwrap());
    pub const JAVASCRIPT: Self = Self(NonZero::<u8>::new(172).unwrap());
    pub const JSON: Self = Self(NonZero::<u8>::new(173).unwrap());
    pub const JULIA: Self = Self(NonZero::<u8>::new(174).unwrap());
    pub const JUST: Self = Self(NonZero::<u8>::new(175).unwrap());
    pub const KEY: Self = Self(NonZero::<u8>::new(176).unwrap());
    pub const KOTLIN: Self = Self(NonZero::<u8>::new(177).unwrap());
    pub const LATEX: Self = Self(NonZero::<u8>::new(178).unwrap());
    pub const LICENSE: Self = Self(NonZero::<u8>::new(179).unwrap());
    pub const LOCK: Self = Self(NonZero::<u8>::new(180).unwrap());
    pub const LUA_CONFIG: Self = Self(NonZero::<u8>::new(181).unwrap());
    pub const LUA: Self = Self(NonZero::<u8>::new(182).unwrap());
    pub const LUAU_CONFIG: Self = Self(NonZero::<u8>::new(183).unwrap());
    pub const LUAU_DEF: Self = Self(NonZero::<u8>::new(184).unwrap());
    pub const LUAU: Self = Self(NonZero::<u8>::new(185).unwrap());
    pub const MAKEFILE: Self = Self(NonZero::<u8>::new(186).unwrap());
    pub const MARKDOWN: Self = Self(NonZero::<u8>::new(187).unwrap());
    pub const MARKDOWNX: Self = Self(NonZero::<u8>::new(188).unwrap());
    pub const NEXT: Self = Self(NonZero::<u8>::new(189).unwrap());
    pub const NIM: Self = Self(NonZero::<u8>::new(190).unwrap());
    pub const NIX: Self = Self(NonZero::<u8>::new(191).unwrap());
    pub const NODE: Self = Self(NonZero::<u8>::new(192).unwrap());
    pub const NPM_LOCK: Self = Self(NonZero::<u8>::new(193).unwrap());
    pub const NPM: Self = Self(NonZero::<u8>::new(194).unwrap());
    pub const NUXT: Self = Self(NonZero::<u8>::new(195).unwrap());
    pub const ODIN: Self = Self(NonZero::<u8>::new(196).unwrap());
    pub const PACKAGE_CONFIG: Self = Self(NonZero::<u8>::new(197).unwrap());
    pub const PACKAGE_LOCK: Self = Self(NonZero::<u8>::new(198).unwrap());
    pub const PCSS: Self = Self(NonZero::<u8>::new(199).unwrap());
    pub const PDF: Self = Self(NonZero::<u8>::new(200).unwrap());
    pub const PERL: Self = Self(NonZero::<u8>::new(201).unwrap());
    pub const PHP: Self = Self(NonZero::<u8>::new(202).unwrap());
    pub const POWERSHELL: Self = Self(NonZero::<u8>::new(203).unwrap());
    pub const PYTHON: Self = Self(NonZero::<u8>::new(204).unwrap());
    pub const REACT_TYPESCRIPT: Self = Self(NonZero::<u8>::new(205).unwrap());
    pub const REACT: Self = Self(NonZero::<u8>::new(206).unwrap());
    pub const README: Self = Self(NonZero::<u8>::new(207).unwrap());
    pub const ROBLOX_CONFIG: Self = Self(NonZero::<u8>::new(208).unwrap());
    pub const ROBLOX_LOCK: Self = Self(NonZero::<u8>::new(209).unwrap());
    pub const ROBLOX_MODEL: Self = Self(NonZero::<u8>::new(210).unwrap());
    pub const ROBLOX: Self = Self(NonZero::<u8>::new(211).unwrap());
    pub const RUBY: Self = Self(NonZero::<u8>::new(212).unwrap());
    pub const RUST_CONFIG: Self = Self(NonZero::<u8>::new(213).unwrap());
    pub const RUST: Self = Self(NonZero::<u8>::new(214).unwrap());
    pub const SCALA: Self = Self(NonZero::<u8>::new(215).unwrap());
    pub const SCSS: Self = Self(NonZero::<u8>::new(216).unwrap());
    pub const SECURITY: Self = Self(NonZero::<u8>::new(217).unwrap());
    pub const SHELL: Self = Self(NonZero::<u8>::new(218).unwrap());
    pub const STORYBOOK: Self = Self(NonZero::<u8>::new(219).unwrap());
    pub const SVELTE: Self = Self(NonZero::<u8>::new(220).unwrap());
    pub const SVG: Self = Self(NonZero::<u8>::new(221).unwrap());
    pub const SWIFT: Self = Self(NonZero::<u8>::new(222).unwrap());
    pub const TAILWIND: Self = Self(NonZero::<u8>::new(223).unwrap());
    pub const TERRAFORM: Self = Self(NonZero::<u8>::new(224).unwrap());
    pub const TEST_BLUE: Self = Self(NonZero::<u8>::new(225).unwrap());
    pub const TEST_TEAL: Self = Self(NonZero::<u8>::new(226).unwrap());
    pub const TEST_YELLOW: Self = Self(NonZero::<u8>::new(227).unwrap());
    pub const TEXT: Self = Self(NonZero::<u8>::new(228).unwrap());
    pub const TODO: Self = Self(NonZero::<u8>::new(229).unwrap());
    pub const TOML: Self = Self(NonZero::<u8>::new(230).unwrap());
    pub const TYPESCRIPT_CONFIG: Self = Self(NonZero::<u8>::new(231).unwrap());
    pub const TYPESCRIPT_DEF: Self = Self(NonZero::<u8>::new(232).unwrap());
    pub const TYPESCRIPT: Self = Self(NonZero::<u8>::new(233).unwrap());
    pub const VIDEO: Self = Self(NonZero::<u8>::new(234).unwrap());
    pub const VISUAL_STUDIO: Self = Self(NonZero::<u8>::new(235).unwrap());
    pub const VITE: Self = Self(NonZero::<u8>::new(236).unwrap());
    pub const VSCODE: Self = Self(NonZero::<u8>::new(237).unwrap());
    pub const VUE: Self = Self(NonZero::<u8>::new(238).unwrap());
    pub const WALLY_LOCK: Self = Self(NonZero::<u8>::new(239).unwrap());
    pub const WALLY: Self = Self(NonZero::<u8>::new(240).unwrap());
    pub const WEB_ASSEMBLY: Self = Self(NonZero::<u8>::new(241).unwrap());
    pub const WORKFLOW: Self = Self(NonZero::<u8>::new(242).unwrap());
    pub const XML: Self = Self(NonZero::<u8>::new(243).unwrap());
    pub const YAML: Self = Self(NonZero::<u8>::new(244).unwrap());
    pub const YARN_LOCK: Self = Self(NonZero::<u8>::new(245).unwrap());
    pub const YARN: Self = Self(NonZero::<u8>::new(246).unwrap());
    pub const ZIG: Self = Self(NonZero::<u8>::new(247).unwrap());
    pub const ZIP: Self = Self(NonZero::<u8>::new(248).unwrap());

    pub const fn inner(self) -> u8 {
        self.0.get()
    }
}

/// https://sw.kov_dgoyal.net/kitty/graphics-protocol/#a-minimal-example
fn load_kitty_icon(
    IconId(id): IconId,
    icon: &[u8],
    stdout: &mut impl Write,
) -> Result<(), io::Error> {
    let mut chunks = icon.chunks(4096).peekable();
    let mut first = true;

    while let Some(chunk) = chunks.next() {
        let remaining = u8::from(chunks.peek().is_some());

        if first {
            first = false;
            write!(stdout, "\x1b_Ga=t,f=100,q=2,i={id},m={remaining};")?;
        } else {
            write!(stdout, "\x1b_Gm={remaining};")?;
        }

        stdout.write_all(chunk)?;
        stdout.write_all(b"\x1b\\")?;
    }

    write!(stdout, "\x1b_Ga=p,U=1,q=2,i={id},c=2,r=1\x1b\\")?;

    Ok(())
}
