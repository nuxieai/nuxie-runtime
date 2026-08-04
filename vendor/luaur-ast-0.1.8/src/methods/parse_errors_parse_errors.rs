use crate::records::parse_error::ParseError;
use crate::records::parse_errors::ParseErrors;
use luaur_common::LUAU_ASSERT;

impl ParseErrors {
    pub fn new(errors: Vec<ParseError>) -> Self {
        LUAU_ASSERT!(!errors.is_empty());

        let message = if errors.len() == 1 {
            errors[0].what().to_string()
        } else {
            alloc::format!("{} parse errors", errors.len())
        };

        Self { errors, message }
    }
}

#[allow(non_snake_case)]
pub fn parse_errors_parse_errors(errors: Vec<ParseError>) -> ParseErrors {
    ParseErrors::new(errors)
}
