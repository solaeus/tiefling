use std::{
    io::{self, Stdout, StdoutLock, Write, stdout},
    panic,
};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

pub struct Terminal<'a> {
    stdout: StdoutLock<'a>,
}

impl<'a> Terminal<'a> {
    pub fn new(mut stdout: StdoutLock<'a>) -> Result<Self, io::Error> {
        install_panic_hook();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;

        Ok(Self { stdout })
    }

    pub fn stdout(&mut self) -> &mut StdoutLock<'a> {
        &mut self.stdout
    }
}

impl<'a> Drop for Terminal<'a> {
    fn drop(&mut self) {
        restore_terminal(&mut self.stdout);
    }
}

fn install_panic_hook() {
    let old_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        let mut stdout = stdout();

        restore_terminal(&mut stdout);
        old_hook(info);
    }));
}

fn restore_terminal(stdout: &mut impl Write) {
    let _ = execute!(stdout, Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    let _ = stdout.flush();
}
