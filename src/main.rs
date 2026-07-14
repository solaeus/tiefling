mod app;
mod config;
mod icons;
mod models;
mod terminal;
mod views;

use std::{
    env::current_dir,
    io::{self},
};

use crate::{app::App, config::ConfigFile, icons::Icons};

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let _config = ConfigFile::read_or_default();
    let mut app = App::new(current_dir()?, Icons::default())?;

    app.run()?;

    if cfg!(debug_assertions) {
        println!("{app:#?}");
    }

    Ok(())
}
