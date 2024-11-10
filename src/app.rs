use beancount_parser::Directive;
use color_eyre::{eyre::Context, owo_colors::OwoColorize, Result};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use rust_decimal::Decimal;

use crate::{
    beancount::{filter_transactions, parse_beancount_file, TransactionTui},
    cli::Args,
    terminal,
    utils::format_posting_line,
};

#[derive(Debug, Default)]
enum Fields {
    Date,
    TransactionType,
    #[default]
    Payee,
    Narration,
    Account,
    Amount,
    Currency,
}

enum InputMode {
    Normal,
    Edit,
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    transactions: Vec<Directive<Decimal>>,
    current_index: usize,
    currently_editing: Fields,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui, args: Args) -> Result<()> {
        // handle inputs
        let beancount = parse_beancount_file(&args.file)?;
        self.transactions = filter_transactions(beancount);
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.prev_transaction()?,
            KeyCode::Right => self.next_transaction()?,
            _ => {}
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Beancount importer ".bold());
        let instructions = Line::from(vec![
            " Prev Transaction ".into(),
            "<Left>".blue().bold(),
            " Next Transaction ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::default()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let inner_area = block.inner(area);
        let width = inner_area.width as usize;
        // unwrap() is save here because an error is only raised if try_into() is
        // called on a directive which is not a transaction. But we have already
        // filtered out all non-transactions before.
        let current_transaction: TransactionTui = self.transactions[self.current_index]
            .clone()
            .try_into()
            .unwrap();
        let metadata_string = vec![
            Span::from(current_transaction.date + " ").red(),
            Span::from(current_transaction.flag + " ").magenta(),
            Span::from(format!("\"{}\" ", current_transaction.payee)),
            Span::from(format!("\"{}\" ", current_transaction.narration)),
        ];
        let metadata_line = Line::from(metadata_string).style(Style::new().yellow());
        let mut transaction_text = Text::from(vec![metadata_line]);
        for posting in current_transaction.postings {
            let posting_line = format_posting_line(posting, width);
            transaction_text.push_line(posting_line);
        }

        block.render(area, buf);
        Paragraph::new(transaction_text)
            .left_aligned()
            .render(inner_area, buf);
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
