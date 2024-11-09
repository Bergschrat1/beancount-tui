mod app;
mod beancount;
mod cli;
mod error;
mod tui;
mod utils;

use clap::Parser;
use color_eyre::Result;

use crate::cli::Args;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    // create tui
    let mut terminal = tui::init()?;
    let app_result = app::App::default().run(&mut terminal, args);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}
