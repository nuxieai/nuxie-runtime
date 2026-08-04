//! `Parser::report` (`Ast/src/Parser.cpp:5194`).
//!
//! Hand-ported: the base error-reporting method that every `reportXError`
//! funnels into. The C++ `va_list` overload and the variadic `...` wrapper both
//! collapse to a single Rust method taking `core::fmt::Arguments` (callers pass
//! `format_args!(...)`, so `vformat` is already applied at the call site). This
//! is ranked far after its callers because the varargs call edges were not
//! linked in the graph, so it is hand-ported here to unblock them.

use crate::records::location::Location;
use crate::records::parse_error::ParseError;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub(crate) fn report(&mut self, location: Location, message: core::fmt::Arguments<'_>) {
        // To reduce the number of errors reported for incomplete statements, skip
        // multiple errors at the same location (e.g. `local a = (((b + `).
        if let Some(last) = self.parse_errors.last() {
            if location == *last.get_location() {
                return;
            }
        }

        let message = alloc::fmt::format(message);

        let limit = luaur_common::FInt::LuauParseErrorLimit.get();

        // When limited to a single error, behave as if error recovery is disabled.
        if limit == 1 {
            // C++ `throw ParseError(location, message)` — panic with the ParseError
            // OBJECT (not a formatted String) so Parser::parse's catch_unwind can
            // recover it via downcast_ref::<ParseError>().
            std::panic::panic_any(ParseError::new(location, message));
        }

        self.parse_errors.push(ParseError::new(location, message));

        if self.parse_errors.len() >= limit as usize && !self.options.no_error_limit {
            ParseError::raise(location, format_args!("Reached error limit ({})", limit));
        }
    }
}
