use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeancountTuiError {
    #[error("couldn't parse input")]
    Parser(String),
}
