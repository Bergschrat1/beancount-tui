use std::{fs, path::PathBuf};

use beancount_parser::{BeancountFile, Directive, DirectiveContent, Posting, Transaction};
use color_eyre::Result;
use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};
use rust_decimal::Decimal;
use tui_textarea::TextArea;

use crate::{error::BeancountTuiError, utils::format_date};

macro_rules! create_textarea {
    ($name:expr, $value:expr) => {{
        let mut textarea = TextArea::new(vec![$value]);
        textarea.set_block(Block::default().borders(Borders::ALL).title($name));
        textarea.set_cursor_line_style(Style::default());
        textarea
    }};
}

// PostingTUI

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostingField {
    Account,
    Amount,
    Currency,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PostingTui<'t> {
    pub account_textarea: TextArea<'t>,
    pub amount_textarea: TextArea<'t>,
    pub currency_textarea: TextArea<'t>,
}

impl<'t> TryFrom<Posting<Decimal>> for PostingTui<'t> {
    type Error = BeancountTuiError;

    fn try_from(value: Posting<Decimal>) -> std::prelude::v1::Result<Self, Self::Error> {
        let account_textarea = create_textarea!("Account", value.account.to_string());
        let (amount, currency) = match value.amount {
            Some(a) => (a.value.to_string(), a.currency.to_string()),
            None => ("".to_string(), "".to_string()),
        };
        let amount_textarea = create_textarea!("Amount", amount);
        let currency_textarea = create_textarea!("Currency", currency);
        Ok(Self {
            account_textarea,
            amount_textarea,
            currency_textarea,
        })
    }
}

impl<'t> PostingTui<'t> {
    pub fn next_field(&mut self, current_field: &PostingField, forward: bool) -> PostingField {
        match (current_field, forward) {
            (PostingField::Account, true) => PostingField::Amount,
            (PostingField::Amount, true) => PostingField::Currency,
            (PostingField::Currency, true) => PostingField::Account,
            (PostingField::Account, false) => PostingField::Currency,
            (PostingField::Currency, false) => PostingField::Amount,
            (PostingField::Amount, false) => PostingField::Account,
        }
    }

    pub fn get_field_mut(&mut self, field: &PostingField) -> &mut TextArea<'t> {
        match field {
            PostingField::Account => &mut self.account_textarea,
            PostingField::Amount => &mut self.amount_textarea,
            PostingField::Currency => &mut self.currency_textarea,
        }
    }

    pub fn get_field(&self, field: &PostingField) -> &TextArea<'t> {
        match field {
            PostingField::Account => &self.account_textarea,
            PostingField::Amount => &self.amount_textarea,
            PostingField::Currency => &self.currency_textarea,
        }
    }
}

// TransactionTui

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TransactionTui<'t> {
    pub directive: Transaction<Decimal>,
    pub metadata_textareas: [TextArea<'t>; 4],
    pub postings_textareas: Vec<PostingTui<'t>>,
}

impl<'t> TryFrom<&Directive<Decimal>> for TransactionTui<'t> {
    type Error = BeancountTuiError;

    fn try_from(value: &Directive<Decimal>) -> Result<Self, BeancountTuiError> {
        let DirectiveContent::Transaction(transaction) = value.content.to_owned() else {
            return Err(BeancountTuiError::Parser(
                "Can only parse Transactions".to_string(),
            ));
        };
        let date_textarea = create_textarea!("Date", format_date(&value.date));
        let flag_textarea = create_textarea!(
            "",
            match transaction.flag {
                Some(c) => c.to_string(),
                None => "*".to_string(),
            }
        );
        let payee_textarea = create_textarea!(
            "Payee",
            transaction.payee.clone().unwrap_or_default()
        );
        let narration_textarea = create_textarea!(
            "Narration",
            transaction.narration.clone().unwrap_or_default()
        );
        let postings_textareas = transaction
            .postings
            .clone()
            .into_iter()
            .map(|p| p.try_into().expect("Couldn't parse posting."))
            .collect::<Vec<PostingTui>>();
        Ok(TransactionTui {
            directive: transaction,
            metadata_textareas: [
                date_textarea,
                flag_textarea,
                payee_textarea,
                narration_textarea,
            ],
            postings_textareas,
        })
    }
}

impl<'t> TransactionTui<'t> {
    pub fn format_transaction(&self) -> String {
        let metadata = self
            .metadata_textareas
            .iter()
            .map(|ta| ta.lines().join(" "))
            .collect::<Vec<_>>();
        let postings = self
            .postings_textareas
            .iter()
            .map(|posting| {
                format!(
                    "    {}    {} {}",
                    posting.account_textarea.lines().join(" "),
                    posting.amount_textarea.lines().join(" "),
                    posting.currency_textarea.lines().join(" "),
                )
            })
            .collect::<Vec<_>>();

        format!(
            "{} {} {} {}\n{}",
            metadata.first().unwrap_or(&"".to_string()),
            metadata.get(1).unwrap_or(&"".to_string()),
            metadata.get(2).unwrap_or(&"".to_string()),
            metadata.get(3).unwrap_or(&"".to_string()),
            postings.join("\n")
        )
    }
}

pub fn parse_beancount_file(file_path: &PathBuf) -> Result<BeancountFile<Decimal>> {
    let beancount_content = fs::read_to_string(file_path)?;
    let beancount: BeancountFile<Decimal> = beancount_content.parse()?;
    Ok(beancount)
}

/// Filters out everything that is not a DirectiveContent::Transaction
pub fn filter_transactions(beancount_file: BeancountFile<Decimal>) -> Vec<Directive<Decimal>> {
    beancount_file
        .directives
        .into_iter()
        .filter(|d| {
            if let DirectiveContent::Transaction(_) = &d.content {
                true
            } else {
                false
            }
        })
        .collect()
}
