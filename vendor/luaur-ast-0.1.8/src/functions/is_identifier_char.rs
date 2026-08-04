use crate::functions::is_digit_pretty_printer::is_digit;
use crate::functions::is_identifier_start_char::is_identifier_start_char;

#[allow(non_snake_case)]
pub fn is_identifier_char(c: char) -> bool {
    is_identifier_start_char(c) || is_digit(c)
}
