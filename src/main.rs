mod app;
mod config;
mod models;
mod views;

use std::io;

use crate::{
    app::App,
    config::{Config, IconSetting},
    models::Icons,
};

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let config = Config::read_or_default();
    let icons = match config.icons {
        IconSetting::Emoji => Icons::emoji(),
        IconSetting::JetBrains => Icons::jet_brains(),
    };
    let mut app = App::new(icons);

    ratatui::run(|terminal| app.run(terminal))?;

    if cfg!(debug_assertions) {
        println!("{app:#?}");
    }

    Ok(())
}
