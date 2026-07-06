mod app;
mod models;
mod views;

use std::io;

use crate::app::App;

fn main() -> Result<(), io::Error> {
    ratatui::run(|terminal| App::new().run(terminal))
}
