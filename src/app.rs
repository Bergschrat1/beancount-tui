use beancount_parser::Directive;
use color_eyre::{eyre::Context, Result};
use crossterm::event::KeyModifiers;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::{Block, Borders},
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tui_textarea::{Input, Key, TextArea};

use crate::{
    beancount::{filter_transactions, parse_beancount_file, TransactionTui},
    cli::Args,
    terminal, ui,
};

const METAFIELD_ORDER: [InputFieldType; 4] = [
    InputFieldType::Date,
    InputFieldType::Flag,
    InputFieldType::Payee,
    InputFieldType::Narration,
];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputFieldType {
    Date,
    Flag,
    Payee,
    Narration,
    Account,
    Amount,
    Currency,
}

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct InputField<'t> {
    textarea: TextArea<'t>,
    field_type: InputFieldType,
}

#[derive(Debug)]
pub struct App<'t> {
    pub exit: bool,
    pub transactions: Vec<Directive<Decimal>>,
    pub current_index: usize,
    pub currently_selected_field: usize,
    pub current_mode: InputMode,
    pub metadata_fields: HashMap<InputFieldType, TextArea<'t>>,
    pub account_fields: Vec<HashMap<InputFieldType, TextArea<'t>>>,
    // TODO field to hold the InputFields
}

impl<'t> App<'t> {
    pub fn new(args: Args) -> Result<Self> {
        // handle inputs
        let beancount = parse_beancount_file(&args.file)?;
        let transactions = filter_transactions(beancount);
        let first_transaction = TransactionTui::try_from(transactions[0].clone()).unwrap();
        let mut ret = Self {
            exit: false,
            transactions,
            current_index: 0,
            currently_selected_field: 2, // payee field
            current_mode: InputMode::Normal,
            metadata_fields: HashMap::default(),
            account_fields: vec![],
        };
        ret.update_textareas();
        Ok(ret)
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, &self))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.into() {
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => self.exit(),
            Input {
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => self.next_transaction()?,
            Input {
                key: Key::Char('p'),
                ctrl: true,
                ..
            } => self.prev_transaction()?,
            Input { key: Key::Tab, .. }
            | Input {
                key: Key::Right,
                ctrl: true,
                ..
            } => self.next_field()?,
            Input {
                key: Key::Left,
                ctrl: true,
                ..
            } => self.prev_field()?,
            text_input => {
                self.metadata_fields
                    .get_mut(&METAFIELD_ORDER[self.currently_selected_field])
                    .unwrap()
                    .input(text_input);
            }
        }
        Ok(())
    }
    fn update_textareas(&mut self) -> Result<()> {
        let current_transaction =
            TransactionTui::try_from(self.transactions[self.current_index].clone()).unwrap();
        let mut date_textarea = TextArea::new(vec![current_transaction.date]);
        let mut flag_textarea = TextArea::new(vec![current_transaction.flag]);
        let mut payee_textarea = TextArea::new(vec![current_transaction.payee]);
        let mut narration_textarea = TextArea::new(vec![current_transaction.narration]);
        date_textarea.set_block(Block::default().borders(Borders::ALL).title("Date"));
        flag_textarea.set_block(Block::default().borders(Borders::ALL));
        payee_textarea.set_block(Block::default().borders(Borders::ALL).title("Payee"));
        narration_textarea.set_block(Block::default().borders(Borders::ALL).title("Narration"));
        self.metadata_fields
            .insert(InputFieldType::Date, date_textarea);
        self.metadata_fields
            .insert(InputFieldType::Flag, flag_textarea);
        self.metadata_fields
            .insert(InputFieldType::Payee, payee_textarea);
        self.metadata_fields
            .insert(InputFieldType::Narration, narration_textarea);

        // self.account_fields = vec![HashMap::from([
        //     (InputFieldType::Account, TextArea::new(vec!["".to_string()])),
        //     (InputFieldType::Amount, TextArea::new(vec!["".to_string()])),
        //     (
        //         InputFieldType::Currency,
        //         TextArea::new(vec!["".to_string()]),
        //     ),
        // ])]; // TODO create all account entries
        Ok(())
    }

    fn next_field(&mut self) -> Result<()> {
        if self.currently_selected_field < METAFIELD_ORDER.len() - 1 {
            self.currently_selected_field += 1;
        } else {
            self.currently_selected_field = 0;
        }
        Ok(())
    }

    fn prev_field(&mut self) -> Result<()> {
        if self.currently_selected_field > 0 {
            self.currently_selected_field -= 1;
        } else {
            self.currently_selected_field = METAFIELD_ORDER.len() - 1;
        }
        Ok(())
    }

    fn next_transaction(&mut self) -> Result<()> {
        if self.current_index < self.transactions.len() - 1 {
            self.current_index = self.current_index.saturating_add(1);
        }
        self.update_textareas();
        Ok(())
    }

    fn prev_transaction(&mut self) -> Result<()> {
        self.current_index = self.current_index.saturating_sub(1);
        self.update_textareas();
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

// #[cfg(test)]
// mod tests {
//     use ratatui::style::Style;

//     use super::*;

//     #[test]
//     fn render() {
//         let app = App::default();
//         let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

//         app.render(buf.area, &mut buf);

//         let mut expected = Buffer::with_lines(vec![
//             "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
//             "┃                    Value: 0                    ┃",
//             "┃                                                ┃",
//             "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
//         ]);
//         let title_style = Style::new().bold();
//         let counter_style = Style::new().yellow();
//         let key_style = Style::new().blue().bold();
//         expected.set_style(Rect::new(14, 0, 22, 1), title_style);
//         expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
//         expected.set_style(Rect::new(13, 3, 6, 1), key_style);
//         expected.set_style(Rect::new(30, 3, 7, 1), key_style);
//         expected.set_style(Rect::new(43, 3, 4, 1), key_style);

//         assert_eq!(buf, expected);
//     }

//     #[test]
//     fn handle_key_event() {
//         let mut app = App::default();
//         app.handle_key_event(KeyCode::Right.into()).unwrap();
//         assert_eq!(app.current_index, 1);

//         app.handle_key_event(KeyCode::Left.into()).unwrap();
//         assert_eq!(app.current_index, 0);

//         let mut app = App::default();
//         app.handle_key_event(KeyCode::Char('q').into()).unwrap();
//         assert!(app.exit);
//     }

//     #[test]
//     #[should_panic(expected = "attempt to subtract with overflow")]
//     fn handle_key_event_panic() {
//         let mut app = App::default();
//         let _ = app.handle_key_event(KeyCode::Left.into());
//     }

//     #[test]
//     fn handle_key_event_overflow() {
//         let mut app = App::default();
//         assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
//         assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
//         assert_eq!(
//             app.handle_key_event(KeyCode::Right.into())
//                 .unwrap_err()
//                 .to_string(),
//             "counter overflow"
//         );
//     }
// }
