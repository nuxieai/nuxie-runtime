use crate::functions::pretty_print_pretty_printer::pretty_print_ast_stat_block_cst_node_map;
use crate::functions::pretty_print_pretty_printer_alt_b::pretty_print_ast_stat_block;
use crate::functions::pretty_print_with_types_pretty_printer::pretty_print_with_types_ast_stat_block_cst_node_map;
use crate::functions::pretty_print_with_types_pretty_printer_alt_b::pretty_print_with_types_ast_stat_block;
use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::parse_options::ParseOptions;
use crate::records::parser::Parser;
use crate::records::pretty_print_result::PrettyPrintResult;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn pretty_print_string_view_parse_options_bool_bool(
    source: &str,
    mut options: ParseOptions,
    with_types: bool,
    ignore_parse_errors: bool,
) -> PrettyPrintResult {
    options.store_cst_data = true;

    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let mut parse_result = Parser::parse(source, source.len(), &mut names, &mut allocator, options);

    let has_errors = !parse_result.errors.is_empty();

    if has_errors && !ignore_parse_errors {
        let error = &parse_result.errors[0];
        return PrettyPrintResult {
            code: alloc::string::String::new(),
            error_location: *error.get_location(),
            parse_error: error.what().to_string(),
        };
    }

    LUAU_ASSERT!(!parse_result.root.is_null());
    if parse_result.root.is_null() {
        return PrettyPrintResult {
            code: alloc::string::String::new(),
            error_location: crate::records::location::Location::default(),
            parse_error: alloc::string::String::from(
                "Internal error: Parser yielded empty parse tree",
            ),
        };
    }

    let root = unsafe { &mut *parse_result.root };
    if with_types {
        PrettyPrintResult {
            code: pretty_print_with_types_ast_stat_block_cst_node_map(
                root,
                parse_result.cst_node_map,
            ),
            error_location: crate::records::location::Location::default(),
            parse_error: alloc::string::String::new(),
        }
    } else {
        PrettyPrintResult {
            code: pretty_print_ast_stat_block_cst_node_map(root, parse_result.cst_node_map),
            error_location: crate::records::location::Location::default(),
            parse_error: alloc::string::String::new(),
        }
    }
}
