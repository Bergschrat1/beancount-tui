use color_eyre::eyre::{OptionExt, Result};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

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

    if app.popup.active {
        let popup_area = centered_rect(30, 20, frame.area());
        draw_popup(frame, app, popup_area)?;
    }
    Ok(())
}

fn draw_popup(frame: &mut Frame, app: &App, area: Rect) -> Result<()> {
    frame.render_widget(Clear, area);
    let popup_block = Block::default()
        .title(Line::from("Confirm").centered())
        .title_bottom(Line::from("<Enter>: Confirm, <Esc>: Decline").centered())
        .borders(Borders::ALL);
    // .style(Style::default().bg(Color::DarkGray));

    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let lines = app.popup.prompt.lines().count();
    let vertical_padding = (area.height.saturating_sub(lines as u16) / 2).max(1); // Ensure at least 1 line padding

    // Add vertical padding manually to center the text
    let padded_text = format!(
        "{}{}{}",
        "\n".repeat(vertical_padding as usize),
        app.popup.prompt,
        "\n".repeat(vertical_padding as usize)
    );

    let exit_paragraph = Paragraph::new(padded_text)
        .alignment(Alignment::Center)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(exit_paragraph, area);
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
        .first()
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
