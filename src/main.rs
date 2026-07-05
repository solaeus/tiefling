mod app;
mod models;
mod views;

use std::error::Error;

use crate::app::App;

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(|terminal| App::new().run(terminal))?;

    Ok(())
}
