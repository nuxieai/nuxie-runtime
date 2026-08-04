use crate::records::ast_array::AstArray;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_break::AstStatBreak;
use crate::records::function::Function;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parser_parse_break(&mut self) -> *mut AstStat {
        let start = self.lexer.current().location;

        self.next_lexeme(); // break

        if self.function_stack.last().unwrap().loop_depth == 0 {
            return self.report_stat_error(
                start,
                AstArray::default(),
                AstArray::default(),
                format_args!("break statement must be inside a loop"),
            ) as *mut AstStat;
        }

        unsafe { (*self.allocator).alloc(AstStatBreak::new(start)) as *mut AstStat }
    }
}
