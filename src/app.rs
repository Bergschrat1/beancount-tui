use beancount_parser::Directive;
use color_eyre::{eyre::Context, Result};
use crossterm::event::KeyModifiers;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rust_decimal::Decimal;
use tui_textarea::TextArea;

use crate::{
    beancount::{filter_transactions, parse_beancount_file},
    cli::Args,
    terminal, ui,
};

#[derive(Debug)]
pub enum InputFieldType {
    Date,
    TransactionType,
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
    pub currently_selected_field: InputFieldType,
    pub current_mode: InputMode,
    pub metadata_fields: Vec<InputField<'t>>,
    pub account_fields: Vec<[InputField<'t>; 3]>,
    // TODO field to hold the InputFields
}

impl<'t> App<'t> {
    pub fn new(args: Args) -> Result<Self> {
        // handle inputs
        let beancount = parse_beancount_file(&args.file)?;
        let transactions = filter_transactions(beancount);
        Ok(Self {
            exit: false,
            transactions,
            current_index: 0,
            currently_selected_field: InputFieldType::Payee,
            current_mode: InputMode::Normal,
            metadata_fields: vec![
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::Date,
                },
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::TransactionType,
                },
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::Payee,
                },
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::Narration,
                },
            ],
            account_fields: vec![[
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::Account,
                },
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::Amount,
                },
                InputField {
                    textarea: TextArea::new(vec!["".to_string()]),
                    field_type: InputFieldType::Currency,
                },
            ]],
        })
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
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => self.exit(),
            // next transaction
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.next_transaction()?,
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => self.next_transaction()?,
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => self.next_transaction()?,
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => self.next_transaction()?,
            // previous transaction
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => self.prev_transaction()?,
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => self.prev_transaction()?,
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => self.prev_transaction()?,
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => self.prev_transaction()?,
            _ => {
                // TODO edit current TextArea AND current transaction
            }
        }
        Ok(())
    }

    fn next_transaction(&mut self) -> Result<()> {
        if self.current_index < self.transactions.len() - 1 {
            self.current_index = self.current_index.saturating_add(1);
        }
        Ok(())
    }

    fn prev_transaction(&mut self) -> Result<()> {
        self.current_index = self.current_index.saturating_sub(1);
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Style;

    use super::*;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_eq!(buf, expected);
    }

    #[test]
    fn handle_key_event() {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        assert_eq!(app.current_index, 1);

        app.handle_key_event(KeyCode::Left.into()).unwrap();
        assert_eq!(app.current_index, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert!(app.exit);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn handle_key_event_panic() {
        let mut app = App::default();
        let _ = app.handle_key_event(KeyCode::Left.into());
    }

    #[test]
    fn handle_key_event_overflow() {
        let mut app = App::default();
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert_eq!(
            app.handle_key_event(KeyCode::Right.into())
                .unwrap_err()
                .to_string(),
            "counter overflow"
        );
    }
}
