use beancount_parser::Date;

pub fn format_date(date: &Date) -> String {
    format!("{}-{:02}-{:02}", date.year, date.month, date.day)
}

// pub fn format_posting_line<'p>(posting: PostingTui, line_width: usize) -> Line<'p> {
//     let account = Span::from(["    ".to_string(), posting.account].join("")).blue();
//     let amount = Span::from(
//         posting
//             .amount
//             .map(|d| d.to_string())
//             .unwrap_or("".to_string()),
//     )
//     .green();
//     let currency =
//         Span::from(" ".to_string() + &posting.currency.unwrap_or("".to_string())).green();
//     let spaces = Span::from(" ".repeat(
//         line_width - account.content.len() - amount.content.len() - currency.content.len() - 1,
//     ));
//     let posting_line = Line::from(vec![account, spaces, amount, currency]);
//     posting_line
// }
