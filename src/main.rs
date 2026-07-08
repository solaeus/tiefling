mod app;
mod config;
mod models;
mod views;

use std::io;

use crate::{
    app::App,
    config::{Config, IconSetting},
    models::{EmojiIconTheme, JetBrainsIconTheme},
};

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let config = Config::read_or_default();
    let mut app = match config.icons {
        IconSetting::Emoji => App::new::<EmojiIconTheme>(),
        IconSetting::JetBrains => App::new::<JetBrainsIconTheme>(),
    };

    ratatui::run(|terminal| app.run(terminal))?;

    if cfg!(debug_assertions) {
        println!("{app:#?}");
    }

    Ok(())
}
