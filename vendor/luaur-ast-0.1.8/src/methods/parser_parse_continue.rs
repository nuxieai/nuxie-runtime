use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_continue::AstStatContinue;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub fn parser_parse_continue(&mut self, start: &Location) -> *mut AstStat {
        if self.function_stack.last().unwrap().loop_depth == 0 {
            return self.report_stat_error(
                *start,
                crate::records::ast_array::AstArray::default(),
                crate::records::ast_array::AstArray::default(),
                format_args!("continue statement must be inside a loop"),
            ) as *mut AstStat;
        }

        // note: the token is already parsed for us!
        let continue_stat = AstStatContinue::new(*start);
        unsafe { (*self.allocator).alloc(continue_stat) as *mut AstStat }
    }
}
