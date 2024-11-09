use beancount_parser::Date;

pub fn format_date(date: &Date) -> String {
    format!("{}-{}-{}", date.year, date.month, date.day)
}
