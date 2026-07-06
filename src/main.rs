mod app;
mod config;
mod models;
mod views;

use std::io;

use crate::{app::App, config::Config};

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let mut app = App::new(Config::read_or_default());

    ratatui::run(|terminal| app.run(terminal))?;

    if cfg!(debug_assertions) {
        println!("{app:#?}");
    }

    Ok(())
}
