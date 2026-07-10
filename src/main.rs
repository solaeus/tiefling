mod app;
mod config;
mod models;
mod views;

use std::{env::current_dir, io};

use crate::{app::App, config::ConfigFile, models::Icons};

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let config = ConfigFile::read_or_default();
    let mut app = App::new(current_dir()?, Icons::JetBrains)?;

    ratatui::run(|terminal| app.run(terminal))?;

    if cfg!(debug_assertions) {
        println!("{app:#?}");
    }

    Ok(())
}
