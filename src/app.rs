use color_eyre::{eyre::Context, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders},
};
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

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputFieldType {
    Date,
    Flag,
    #[default]
    Payee,
    Narration,
    Account,
    Amount,
    Currency,
}

#[derive(Debug, Default, Clone)]
pub struct InputField<'t> {
    pub input_type: InputFieldType,
    pub textarea: TextArea<'t>,
}

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct App<'t> {
    pub exit: bool,                            // wether we want to exit the program
    pub transactions: Vec<TransactionTui<'t>>, // all the transactions that were parsed
    pub current_index: usize,                  // which transaction is currently shown
    pub currently_selected_field: usize,       // which field of the current transaction is selected
    pub current_mode: InputMode,               // in which editing mode are we in
    pub current_account: usize,                // which account is currently selected
}

impl<'t> App<'t> {
    pub fn new(args: Args) -> Result<Self> {
        // handle inputs
        let beancount = parse_beancount_file(&args.file)?;
        let transactions = filter_transactions(beancount)
            .iter()
            .map(|t| t.try_into().expect("Couldn't parse trnsaction!"))
            .collect();
        let mut ret = Self {
            exit: false,
            transactions,
            current_index: 0,
            currently_selected_field: 2, // payee field
            current_mode: InputMode::Normal,
            current_account: 0,
        };
        ret.update_textareas();
        Ok(ret)
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, &self).expect("Couldn't draw ui!"))?;
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
        let current_transaction = &mut self.transactions[self.current_index];
        let mut current_metadata_field =
            &mut current_transaction.metadata_textareas[self.currently_selected_field];
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
            }
            | Input {
                key: Key::Char('l'),
                ctrl: true,
                ..
            } => self.navigate_field(true)?,
            Input {
                key: Key::Left,
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Char('h'),
                ctrl: true,
                ..
            } => self.navigate_field(false)?,
            text_input => {
                current_metadata_field.input(text_input);
            }
        }
        Ok(())
    }

    fn navigate_field(&mut self, forward: bool) -> Result<()> {
        if forward {
            self.currently_selected_field =
                (self.currently_selected_field + 1) % METAFIELD_ORDER.len();
        } else {
            self.currently_selected_field =
                (self.currently_selected_field + METAFIELD_ORDER.len() - 1) % METAFIELD_ORDER.len();
        }
        self.update_textareas();
        Ok(())
    }

    fn update_textareas(&mut self) {
        let current_transaction = &mut self.transactions[self.current_index];

        for (index, metadata_field) in current_transaction
            .metadata_textareas
            .iter_mut()
            .enumerate()
        {
            if index == self.currently_selected_field {
                // Highlight the selected TextArea
                metadata_field.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)), // Highlight with yellow border
                );
                metadata_field.set_cursor_style(Style::default().reversed());
            } else {
                // Reset style for unselected TextAreas
                metadata_field.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default()), // Default border style
                );
                metadata_field.set_cursor_style(Style::default().bg(Color::Reset));
            }
        }
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

    fn toggle_textarea_active(textarea: &mut TextArea) -> Result<()> {
        textarea.set_cursor_style(textarea.cursor_style().reversed());
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
