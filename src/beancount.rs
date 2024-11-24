use std::{fs, path::PathBuf};

use beancount_parser::{BeancountFile, Directive, DirectiveContent, Posting, Transaction};
use color_eyre::Result;
use rust_decimal::Decimal;
use tui_textarea::TextArea;

use crate::{error::BeancountTuiError, utils::format_date};

#[derive(Clone, Debug)]
pub struct PostingTui<'t> {
    pub account_textarea: TextArea<'t>,
    pub amount_textarea: TextArea<'t>,
    pub currency_textarea: TextArea<'t>,
}

impl<'t> TryFrom<Posting<Decimal>> for PostingTui<'t> {
    type Error = BeancountTuiError;

    fn try_from(value: Posting<Decimal>) -> std::prelude::v1::Result<Self, Self::Error> {
        let account_textarea = TextArea::new(vec![value.account.to_string()]);
        let (amount, currency) = match value.amount {
            Some(a) => (a.value.to_string(), a.currency.to_string()),
            None => ("".to_string(), "".to_string()),
        };
        let amount_textarea = TextArea::new(vec![amount]);
        let currency_textarea = TextArea::new(vec![currency]);
        Ok(Self {
            account_textarea,
            amount_textarea,
            currency_textarea,
        })
    }
}

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
        let date_textarea = TextArea::new(vec![format_date(&value.date)]);
        let flag_textarea = TextArea::new(vec![match transaction.flag {
            Some(c) => c.to_string(),
            None => "*".to_string(),
        }]);
        // let links = t.links.into_iter().map(|l| l.to_string()).collect();
        // let tags = t.tags.into_iter().map(|t| t.to_string()).collect();
        let payee_textarea = TextArea::new(vec![match transaction.payee.clone() {
            Some(p) => p,
            None => String::from(""),
        }]);
        let narration_textarea = TextArea::new(vec![match transaction.narration.clone() {
            Some(n) => n,
            None => String::from(""),
        }]);
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
