#[allow(unused)]
mod app;
mod beancount;
mod cli;
mod error;
mod terminal;
mod ui;
mod utils;

use clap::Parser;
use color_eyre::Result;

use crate::cli::Args;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    // create tui
    let mut terminal = terminal::init()?;
    let mut app = app::App::new(args)?;
    let app_result = app.run(&mut terminal);
    if let Err(err) = terminal::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}
