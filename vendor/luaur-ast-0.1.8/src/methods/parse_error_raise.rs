use crate::records::location::Location;
use crate::records::parse_error::ParseError;

impl ParseError {
    #[allow(non_snake_case)]
    pub fn raise(location: Location, format: core::fmt::Arguments<'_>) -> ! {
        let message = alloc::fmt::format(format);

        // C++ `throw ParseError(...)`. The panic payload MUST be the ParseError
        // object itself: `Parser::parse` catches the unwind and recovers the error
        // via `downcast_ref::<ParseError>()`. Panicking with a formatted String
        // (panic!("{}", ...)) made that downcast fail, so the panic escaped the
        // parse boundary uncaught (every recursion-/error-limit test crashed).
        let err =
            crate::methods::parse_error_parse_error::parse_error_parse_error(location, message);
        std::panic::panic_any(err);
    }
}

#[allow(non_snake_case)]
pub fn parse_error_raise(location: Location, format: core::fmt::Arguments<'_>) -> ! {
    ParseError::raise(location, format)
}
