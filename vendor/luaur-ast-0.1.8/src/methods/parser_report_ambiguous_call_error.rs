use crate::records::parser::Parser;
use luaur_common::macros::luau_noinline::LUAU_NOINLINE;

impl Parser {
    LUAU_NOINLINE! {
        pub fn report_ambiguous_call_error(&mut self) {
            self.report(
                self.lexer.current().location,
                format_args!(
                    "Ambiguous syntax: this looks like an argument list for a function call, but could also be a start of new statement; use ';' to separate statements"
                ),
            );
        }
    }
}
