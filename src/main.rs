mod app;
mod config;
mod models;
mod terminal;
mod views;

use std::{
    env::current_dir,
    io::{self, stdout},
};

use crate::{app::App, config::ConfigFile, models::Icons};

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let config = ConfigFile::read_or_default();
    let mut app = App::new(current_dir()?, Icons::JetBrains)?;

    app.run()?;

    if cfg!(debug_assertions) {
        println!("{app:#?}");
    }

    Ok(())
}
