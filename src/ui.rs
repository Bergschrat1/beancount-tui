use color_eyre::eyre::{OptionExt, Result};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) -> Result<()> {
    let title = Line::from(
        format!(
            "Beancount importer ({}/{})",
            app.current_index + 1,
            app.transactions.len()
        )
        .bold(),
    );
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
    let [metadata_area, postings_area] = vertical_layout.areas(inner_area);

    // draw_transaction(frame, app, transaction_area);
    // draw_edit(frame, app, edit_area);
    draw_metadata_fields(frame, app, metadata_area)?;
    draw_postings(frame, app, postings_area)?;
    Ok(())
}

fn draw_metadata_fields(frame: &mut Frame, app: &App, area: Rect) -> Result<()> {
    let horizontal_layout = Layout::horizontal([
        Constraint::Min(10),
        Constraint::Length(5),
        Constraint::Min(10),
        Constraint::Min(10),
    ]);
    let [date_area, flag_area, payee_area, narration_area] = horizontal_layout.areas(area);
    let current_transaction = &app.transactions[app.current_index];
    let date_textarea = current_transaction
        .metadata_textareas
        .get(0)
        .ok_or_eyre("No date field initialized!")?;
    let flag_textarea = current_transaction
        .metadata_textareas
        .get(1)
        .ok_or_eyre("No flag field initialized!")?;
    let payee_textarea = current_transaction
        .metadata_textareas
        .get(2)
        .ok_or_eyre("No payee field initialized!")?;
    let narration_textarea = current_transaction
        .metadata_textareas
        .get(3)
        .ok_or_eyre("No narration field initialized!")?;
    frame.render_widget(date_textarea, date_area);
    frame.render_widget(flag_textarea, flag_area);
    frame.render_widget(payee_textarea, payee_area);
    frame.render_widget(narration_textarea, narration_area);
    Ok(())
}

fn draw_postings(frame: &mut Frame, app: &App, area: Rect) -> Result<()> {
    let current_transaction = &app.transactions[app.current_index];
    let postings = &current_transaction.postings_textareas;

    let layout = Layout::vertical(
        postings
            .iter()
            .map(|_| Constraint::Length(3)) // Each posting gets 3 lines
            .collect::<Vec<_>>(),
    );

    let areas = layout.split(area);

    for (i, posting) in postings.iter().enumerate() {
        let posting_area = areas[i];
        let horizontal_layout = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ]);
        let [account_area, amount_area, currency_area] = horizontal_layout.areas(posting_area);

        frame.render_widget(&posting.account_textarea, account_area);
        frame.render_widget(&posting.amount_textarea, amount_area);
        frame.render_widget(&posting.currency_textarea, currency_area);
    }

    Ok(())
}
