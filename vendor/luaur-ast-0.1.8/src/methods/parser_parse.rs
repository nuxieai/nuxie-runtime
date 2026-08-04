use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::lexeme::Type;
use crate::records::parse_error::ParseError;
use crate::records::parse_options::ParseOptions;
use crate::records::parse_result::ParseResult;
use crate::records::parser::Parser;
use luaur_common::macros::luau_timetrace_scope::LUAU_TIMETRACE_SCOPE;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl Parser {
    pub fn parse(
        buffer: &str,
        buffer_size: usize,
        names: &mut AstNameTable,
        allocator: &mut Allocator,
        options: ParseOptions,
    ) -> ParseResult {
        LUAU_TIMETRACE_SCOPE!("Parser::parse", "Parser");

        // Silence the default panic-hook noise for the parser's exception-
        // emulation unwinds (a caught `ParseError` is a normal syntax/limit
        // error, not a crash).
        crate::functions::install_parse_error_panic_hook::install_parse_error_panic_hook();

        let mut p = Parser::new(buffer, names, allocator as *mut Allocator, options);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let root = p.parse_chunk();
            let current_lexeme = p.lexer.current();
            let mut line_count = current_lexeme.location.end.line;
            if buffer_size > 0 && buffer.as_bytes()[buffer_size - 1] != b'\n' {
                line_count += 1;
            }
            let lines = line_count as usize;

            ParseResult {
                root,
                lines,
                hotcomments: std::mem::take(&mut p.hotcomments),
                errors: std::mem::take(&mut p.parse_errors),
                comment_locations: std::mem::take(&mut p.comment_locations),
                cst_node_map: core::mem::replace(
                    &mut p.cst_node_map,
                    DenseHashMap::new(core::ptr::null_mut()),
                ),
            }
        }));

        match result {
            Ok(res) => res,
            Err(payload) => {
                if let Some(err) = payload.downcast_ref::<ParseError>() {
                    p.parse_errors.push(err.clone());

                    ParseResult {
                        root: core::ptr::null_mut(),
                        lines: 0,
                        hotcomments: Vec::new(),
                        errors: std::mem::take(&mut p.parse_errors),
                        comment_locations: Vec::new(),
                        cst_node_map: core::mem::replace(
                            &mut p.cst_node_map,
                            DenseHashMap::new(core::ptr::null_mut()),
                        ),
                    }
                } else {
                    std::panic::resume_unwind(payload);
                }
            }
        }
    }
}
