mod jetbrains;

use std::{
    fmt::Debug,
    io::{self, Write},
    num::NonZero,
};

use crate::{icons::jetbrains::JetBrainsIconTheme, models::FileFormat};

#[derive(Debug, Default)]
pub enum Icons {
    #[default]
    JetBrains,
}

impl Icons {
    pub fn load_icons(&self, stdout: &mut impl Write) -> Result<(), io::Error> {
        match self {
            Icons::JetBrains => JetBrainsIconTheme::load_icons(stdout),
        }
    }
}

pub trait IconTheme: Debug + Default {
    const COLLAPSED_ICON: &'static [u8];
    const EXPANDED_ICON: &'static [u8];
    const EXTENSION_ICONS: &'static [(IconId, &'static [u8])];

    fn load_icons(stdout: &mut impl Write) -> Result<(), io::Error> {
        load_kitty_icon(IconId::COLLAPSED, Self::COLLAPSED_ICON, stdout)?;
        load_kitty_icon(IconId::EXPANDED, Self::EXPANDED_ICON, stdout)?;

        for (icon_id, icon_data) in Self::EXTENSION_ICONS {
            load_kitty_icon(*icon_id, icon_data, stdout)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IconId(pub NonZero<u8>);

#[expect(clippy::disallowed_methods)]
impl IconId {
    pub const COLLAPSED: Self = Self(NonZero::new(1).unwrap());
    pub const EXPANDED: Self = Self(NonZero::new(2).unwrap());
}

impl IconId {
    pub const fn from_format(format: FileFormat) -> Self {
        let inner = format as u8 + 4;

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

    write!(stdout, "\x1b_Ga=p,U=1,i={id},c=2,r=1\x1b\\")?;

    Ok(())
}
