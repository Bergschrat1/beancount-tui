use std::{fs, path::PathBuf};

use beancount_parser::{BeancountFile, Directive, DirectiveContent};
use color_eyre::Result;
use rust_decimal::Decimal;

pub fn parse_beancount_file(file_path: &PathBuf) -> Result<BeancountFile<Decimal>> {
    let beancount_content = fs::read_to_string(file_path)?;
    let beancount: BeancountFile<Decimal> = beancount_content.parse()?;
    Ok(beancount)
}

pub fn filter_transactions(beancount_file: BeancountFile<Decimal>) -> Vec<Directive<Decimal>> {
    beancount_file
        .directives
        .into_iter()
        .filter(|d| {
            if let DirectiveContent::Transaction(_) = &d.content {
                return true;
            } else {
                return false;
            }
        })
        .collect()
}
