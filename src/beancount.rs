use std::{fs, path::PathBuf};

use beancount_parser::{BeancountFile, Directive, DirectiveContent, Posting};
use color_eyre::{owo_colors::OwoColorize, Result};
use rust_decimal::Decimal;

use crate::{error::BeancountTuiError, utils::format_date};

pub struct PostingTui {
    pub account: String,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
}

impl TryFrom<Posting<Decimal>> for PostingTui {
    type Error = BeancountTuiError;

    fn try_from(value: Posting<Decimal>) -> std::prelude::v1::Result<Self, Self::Error> {
        let amount = value.amount.clone().map(|a| a.value);
        let currency = value.amount.map(|a| a.currency.to_string());
        Ok(Self {
            account: value.account.to_string(),
            amount,
            currency,
        })
    }
}

pub struct TransactionTui {
    pub date: String,
    pub flag: String,
    pub payee: String,
    pub narration: String,
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub postings: Vec<PostingTui>,
}

impl TryFrom<Directive<Decimal>> for TransactionTui {
    type Error = BeancountTuiError;

    fn try_from(value: Directive<Decimal>) -> std::prelude::v1::Result<Self, Self::Error> {
        let DirectiveContent::Transaction(t) = value.content else {
            return Err(BeancountTuiError::Parser);
        };
        let date = format_date(&value.date);
        let flag = match t.flag {
            Some(c) => c.to_string(),
            None => "*".to_string(),
        };
        let links = t.links.into_iter().map(|l| l.to_string()).collect();
        let tags = t.tags.into_iter().map(|t| t.to_string()).collect();
        let payee = match t.payee {
            Some(p) => p,
            None => String::from(""),
        };
        let narration = match t.narration {
            Some(n) => n,
            None => String::from(""),
        };
        let postings = t
            .postings
            .into_iter()
            .map(|p| p.try_into().unwrap())
            .collect::<Vec<PostingTui>>();
        Ok(TransactionTui {
            date,
            flag,
            payee,
            narration,
            links,
            tags,
            postings,
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
