use std::ops::Sub;

use color_eyre::{eyre::Context, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders},
};
use tui_textarea::{Input, Key, TextArea};

use crate::{
    beancount::{filter_transactions, parse_beancount_file, PostingField, TransactionTui},
    cli::Args,
    terminal, ui,
};

const METAFIELD_ORDER: [InputFieldType; 4] = [
    InputFieldType::Date,
    InputFieldType::Flag,
    InputFieldType::Payee,
    InputFieldType::Narration,
];

const POSTING_FIELD_ORDER: [PostingField; 3] = [
    PostingField::Account,
    PostingField::Amount,
    PostingField::Currency,
];

#[derive(Debug, Clone)]
pub struct Popup {
    pub active: bool,
    pub prompt: String,
}

impl Popup {
    pub fn show(&mut self, prompt: &str) {
        self.active = true;
        self.prompt = prompt.to_string();
    }
    pub fn hide(&mut self) {
        self.active = false;
    }
}

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

#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct App<'t> {
    pub exit: bool,                               // wether we want to exit the program
    pub transactions: Vec<TransactionTui<'t>>,    // all the transactions that were parsed
    pub current_index: usize,                     // which transaction is currently shown
    pub currently_selected_metadata_field: usize, // which field of the current transaction is selected
    pub currently_selected_posting: usize,        // the posting that is currently selected
    pub currently_selected_posting_field: PostingField, // the posting that is currently selected
    pub current_mode: InputMode,                  // in which editing mode are we in
    pub current_account: usize,                   // which account is currently selected
    pub focus_on_postings: bool, // wether we are currently focused on a posting field or a metadata field
    pub popup: Popup,
}

impl<'t> App<'t> {
    pub fn new(args: Args) -> Result<Self> {
        // handle inputs
        let beancount = parse_beancount_file(&args.file)?;
        let transactions: Vec<TransactionTui<'t>> = filter_transactions(beancount)
            .iter()
            .map(|t| t.try_into().expect("Couldn't parse trnsaction!"))
            .collect();
        let mut ret = Self {
            exit: false,
            transactions,
            current_index: 0,
            currently_selected_metadata_field: 2, // payee field
            currently_selected_posting: 0,
            currently_selected_posting_field: PostingField::Account,
            current_mode: InputMode::Normal,
            current_account: 0,
            focus_on_postings: false,
            popup: Popup {
                active: false,
                prompt: "".to_string(),
            },
        };
        ret.update_textareas();
        Ok(ret)
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<Vec<TransactionTui<'t>>> {
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, self).expect("Couldn't draw ui!"))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(self.transactions.clone())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if self.popup.active {
                    self.handle_popup_key_event(key_event)
                        .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
                } else {
                    self.handle_key_event(key_event)
                        .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
                }
            }
            _ => Ok(()),
        }
    }

    fn handle_popup_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.into() {
            Input { key: Key::Esc, .. } => self.popup.hide(),
            Input {
                key: Key::Enter, ..
            } => self.exit(),
            _ => (),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let current_transaction = &mut self.transactions[self.current_index];
        let current_field = {
            if self.focus_on_postings {
                current_transaction.postings_textareas[self.currently_selected_posting]
                    .get_field_mut(&self.currently_selected_posting_field)
            } else {
                &mut current_transaction.metadata_textareas[self.currently_selected_metadata_field]
            }
        };
        match key_event.into() {
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                self.confirm_close();
            }
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
            // Focus right
            Input {
                key: Key::Char('l'),
                ctrl: true,
                ..
            } => {
                if self.focus_on_postings {
                    self.navigate_posting_field(true)?;
                } else {
                    self.navigate_metadata_field(true)?;
                }
            }
            // Focus left
            Input {
                key: Key::Char('h'),
                ctrl: true,
                ..
            } => {
                if self.focus_on_postings {
                    self.navigate_posting_field(false)?;
                } else {
                    self.navigate_metadata_field(false)?;
                }
            }
            // Focus Down
            Input {
                key: Key::Char('j'),
                ctrl: true,
                ..
            } => {
                if self.focus_on_postings {
                    self.navigate_posting(true)?;
                } else {
                    self.focus_on_postings = true;
                    self.currently_selected_posting = 0;
                    self.update_textareas();
                }
            }
            // Focus Up
            Input {
                key: Key::Char('k'),
                ctrl: true,
                ..
            } => {
                if self.focus_on_postings {
                    self.navigate_posting(false)?;
                } else {
                    self.focus_on_postings = true;
                    self.currently_selected_posting =
                        current_transaction.postings_textareas.len() - 1;
                    self.update_textareas();
                }
            }
            // add new posting
            Input {
                key: Key::Char('o'),
                ctrl: true,
                ..
            } => {
                self.add_posting();
            }
            text_input => {
                current_field.input(text_input);
            }
        }
        Ok(())
    }

    fn navigate_metadata_field(&mut self, forward: bool) -> Result<()> {
        if forward {
            self.currently_selected_metadata_field =
                (self.currently_selected_metadata_field + 1) % METAFIELD_ORDER.len();
        } else {
            self.currently_selected_metadata_field =
                (self.currently_selected_metadata_field + METAFIELD_ORDER.len() - 1)
                    % METAFIELD_ORDER.len();
        }
        self.update_textareas();
        Ok(())
    }

    fn navigate_posting(&mut self, forward: bool) -> Result<()> {
        let current_transaction = &mut self.transactions[self.current_index];
        let n_postings = current_transaction.postings_textareas.len();
        if forward {
            let next_posting = self.currently_selected_posting + 1;
            if next_posting >= n_postings {
                self.focus_on_postings = false
            } else {
                self.currently_selected_posting = next_posting
            }
        } else {
            let prev_posting = self.currently_selected_posting.checked_sub(1);
            if prev_posting.is_none() {
                self.focus_on_postings = false
            } else {
                self.currently_selected_posting = prev_posting.unwrap()
            }
        }
        self.update_textareas();
        Ok(())
    }
    fn navigate_posting_field(&mut self, forward: bool) -> Result<()> {
        let current_transaction = &mut self.transactions[self.current_index];
        let current_posting =
            &mut current_transaction.postings_textareas[self.currently_selected_posting];
        self.currently_selected_posting_field =
            current_posting.next_field(&self.currently_selected_posting_field, forward);
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
            let block = metadata_field
                .block()
                .expect("Textarea should have a block");
            if index == self.currently_selected_metadata_field && !self.focus_on_postings {
                // Highlight the selected TextArea
                // FIXME this currently overwrites the title of the block
                metadata_field.set_block(
                    block
                        .clone()
                        .border_style(Style::default().fg(Color::Yellow)), // Highlight with yellow border
                );
                metadata_field.set_cursor_style(Style::default().reversed());
            } else {
                // Reset style for unselected TextAreas
                metadata_field.set_block(
                    block.clone().border_style(Style::default()), // Default border style
                );
                metadata_field.set_cursor_style(Style::default().bg(Color::Reset));
            }
        }
        for (index, posting) in current_transaction
            .postings_textareas
            .iter_mut()
            .enumerate()
        {
            for posting_field in POSTING_FIELD_ORDER {
                if index == self.currently_selected_posting
                    && posting_field == self.currently_selected_posting_field
                    && self.focus_on_postings
                {
                    let current_posting_field = posting.get_field_mut(&posting_field);
                    let block = current_posting_field
                        .block()
                        .expect("Textarea should have a block");
                    // Highlight the selected TextArea
                    current_posting_field.set_block(
                        block
                            .clone()
                            .border_style(Style::default().fg(Color::Yellow)), // Highlight with yellow border
                    );
                    current_posting_field.set_cursor_style(Style::default().reversed());
                } else {
                    let current_posting_field = posting.get_field_mut(&posting_field);
                    let block = current_posting_field
                        .block()
                        .expect("Textarea should have a block");
                    // Reset style for unselected TextAreas
                    current_posting_field.set_block(
                        block.clone().border_style(Style::default()), // Default border style
                    );
                    current_posting_field.set_cursor_style(Style::default().bg(Color::Reset));
                }
            }
        }
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

    fn toggle_textarea_active(textarea: &mut TextArea) -> Result<()> {
        textarea.set_cursor_style(textarea.cursor_style().reversed());
        Ok(())
    }

    fn confirm_close(&mut self) {
        self.popup
            .show("Do you want to close the application and print the transaction to stdout?")
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn add_posting(&mut self) {
        let current_transaction = &mut self.transactions[self.current_index];
        current_transaction.add_posting();
        self.update_textareas();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use event::{KeyCode, KeyEventState, KeyModifiers};
    use ratatui::style::Style;

    use super::*;

    #[test]
    fn test_app_initialization() {
        let args = Args {
            file: PathBuf::from("data/test.beancount"),
        };
        let app = App::new(args).expect("Failed to initialize app");

        assert!(!app.exit);
        assert_eq!(app.current_index, 0);
        assert_eq!(app.currently_selected_metadata_field, 2);
        assert_eq!(app.currently_selected_posting, 0);
        assert_eq!(app.currently_selected_posting_field, PostingField::Account);
        assert_eq!(app.current_mode, InputMode::Normal);
        assert_eq!(app.current_account, 0);
        assert!(!app.popup.active);
    }

    #[test]
    fn test_handle_key_event_navigate_transaction() {
        let args = Args {
            file: PathBuf::from("data/test.beancount"),
        };
        let mut app = App::new(args).expect("Failed to initialize app");
        let initial_index = app.current_index;

        let key_event_next = KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_next)
            .expect("Failed to handle key event");
        assert_eq!(app.current_index, initial_index + 1);

        let key_event_prev = KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_prev)
            .expect("Failed to handle key event");
        assert_eq!(app.current_index, initial_index);
    }

    #[test]
    fn test_toggle_popup() {
        let args = Args {
            file: PathBuf::from("data/test.beancount"),
        };
        let mut app = App::new(args).expect("Failed to initialize app");

        app.popup.show("Test Prompt");
        assert!(app.popup.active);
        assert_eq!(app.popup.prompt, "Test Prompt");

        app.popup.hide();
        assert!(!app.popup.active);
    }

    #[test]
    fn test_navigation_between_fields() {
        let args = Args {
            file: PathBuf::from("data/test.beancount"),
        };
        let mut app = App::new(args).expect("Failed to initialize app");
        let initial_field = app.currently_selected_metadata_field;

        let key_event_next = KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_next)
            .expect("Failed to handle key event");
        assert_ne!(app.currently_selected_metadata_field, initial_field);

        let key_event_prev = KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_prev)
            .expect("Failed to handle key event");
        assert_eq!(app.currently_selected_metadata_field, initial_field);

        let key_event_down = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_down)
            .expect("Failed to handle key event");
        assert!(app.focus_on_postings);

        let key_event_up = KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_up)
            .expect("Failed to handle key event");
        assert!(!app.focus_on_postings);
    }

    #[test]
    fn test_add_new_posting() {
        let args = Args {
            file: PathBuf::from("data/test.beancount"),
        };
        let mut app = App::new(args).expect("Failed to initialize app");
        let initial_postings_count = app.transactions[app.current_index].postings_textareas.len();

        let key_event_add = KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_add)
            .expect("Failed to handle key event");
        assert_eq!(
            app.transactions[app.current_index].postings_textareas.len(),
            initial_postings_count + 1
        );
    }

    #[test]
    fn test_edit_textfields() {
        let args = Args {
            file: PathBuf::from("data/test.beancount"),
        };
        let mut app = App::new(args).expect("Failed to initialize app");
        let initial_text = "text".to_string();

        let current_field = &mut app.transactions[app.current_index].metadata_textareas
            [app.currently_selected_metadata_field];
        current_field.insert_str(&initial_text);
        let key_event_input = KeyEvent {
            code: KeyCode::Char('N'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        app.handle_key_event(key_event_input)
            .expect("Failed to handle key event");

        let current_field = &mut app.transactions[app.current_index].metadata_textareas
            [app.currently_selected_metadata_field];
        assert_ne!(current_field.lines().join(" "), initial_text);
        assert!(current_field.lines().join(" ").contains('N'));
    }
}
