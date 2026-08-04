use crate::records::allocator::Allocator;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name_table::AstNameTable;
use crate::records::parse_node_result::ParseNodeResult;
use crate::records::parse_options::ParseOptions;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_expr_c_char_usize_ast_name_table_allocator_parse_options(
        buffer: &str,
        buffer_size: usize,
        names: &mut AstNameTable,
        allocator: &mut Allocator,
        options: ParseOptions,
    ) -> ParseNodeResult<AstExpr> {
        Parser::run_parse(buffer, buffer_size, names, allocator, options, |parser| {
            parser.parse_expr_i32(0)
        })
    }
}
