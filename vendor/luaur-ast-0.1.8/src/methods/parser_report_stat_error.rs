use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_error::AstStatError;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn report_stat_error(
        &mut self,
        location: Location,
        expressions: AstArray<*mut AstExpr>,
        statements: AstArray<*mut AstStat>,
        format: core::fmt::Arguments<'_>,
    ) -> *mut AstStatError {
        self.report(location, format);

        let message_index = (self.parse_errors.len() as u32).saturating_sub(1);

        unsafe {
            let allocator = &mut *self.allocator;
            allocator.alloc(AstStatError::new(
                location,
                expressions,
                statements,
                message_index,
            ))
        }
    }
}
