use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app::App, beancount::TransactionTui, utils::format_posting_line};

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
    draw_transaction(frame, app, inner_area)
}
pub fn draw_transaction(frame: &mut Frame, app: &App, area: Rect) {
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
