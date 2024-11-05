use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the file to handle(, use - to read from stdin (must not be a tty))
    #[arg(short, long)]
    pub file: PathBuf,
}
