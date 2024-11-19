use std::borrow::Borrow;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use crate::{
    app::{App, InputFieldType},
    beancount::TransactionTui,
    utils::format_posting_line,
};

pub fn draw(frame: &mut Frame, app: &App) {
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
    frame.render_widget(&block, frame.area());
    let inner_area = block.inner(frame.area());
    let vertical_layout = Layout::vertical([Constraint::Length(3), Constraint::Min(10)]);
    let [metadata_area, account_area] = vertical_layout.areas(inner_area);

    // draw_transaction(frame, app, transaction_area);
    // draw_edit(frame, app, edit_area);
    draw_metadata_fields(frame, app, metadata_area);
}

fn draw_metadata_fields(frame: &mut Frame, app: &App, area: Rect) {
    let horizontal_layout = Layout::horizontal([
        Constraint::Min(10),
        Constraint::Length(5),
        Constraint::Min(10),
        Constraint::Min(10),
    ]);
    let [date_area, flag_area, payee_area, narration_area] = horizontal_layout.areas(area);
    let date_textarea = app.metadata_fields.get(&InputFieldType::Date).unwrap();
    let flag_textarea = app.metadata_fields.get(&InputFieldType::Flag).unwrap();
    let payee_textarea = app.metadata_fields.get(&InputFieldType::Payee).unwrap();
    let narration_textarea = app.metadata_fields.get(&InputFieldType::Narration).unwrap();
    frame.render_widget(date_textarea, date_area);
    frame.render_widget(flag_textarea, flag_area);
    frame.render_widget(payee_textarea, payee_area);
    frame.render_widget(narration_textarea, narration_area);
}

fn draw_transaction(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width as usize;
    // unwrap() is save here because an error is only raised if try_into() is
    // called on a directive which is not a transaction. But we have already
    // filtered out all non-transactions before.
    let current_transaction: TransactionTui = app.transactions[app.current_index]
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

    let transaction_widget = Paragraph::new(transaction_text).left_aligned();
    frame.render_widget(transaction_widget, area)
}

fn draw_edit(frame: &mut Frame, app: &App, area: Rect) {
    let mut textarea = TextArea::new(vec!["cool".to_string(), "stuff".to_string()]);
    textarea.set_block(Block::default().borders(Borders::ALL).title("My textbox"));
    frame.render_widget(&textarea, area)
}
