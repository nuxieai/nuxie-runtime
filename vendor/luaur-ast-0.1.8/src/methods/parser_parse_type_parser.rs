use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::ast_type::AstType;
use crate::records::parse_node_result::ParseNodeResult;
use crate::records::parse_options::ParseOptions;
use crate::records::parser::Parser;
use luaur_common::macros::luau_timetrace_scope::LUAU_TIMETRACE_SCOPE;

impl Parser {
    #[allow(non_snake_case)]
    pub fn parse_type_c_char_usize_ast_name_table_allocator_parse_options(
        buffer: &str,
        names: &mut AstNameTable,
        allocator: *mut Allocator,
        options: ParseOptions,
    ) -> ParseNodeResult<AstType> {
        LUAU_TIMETRACE_SCOPE!("Parser::parseType", "Parser");

        Parser::run_parse::<AstType, _>(
            buffer,
            buffer.len(),
            names,
            unsafe { &mut *allocator },
            options,
            |parser| parser.parse_type_bool(false),
        )
    }
}
