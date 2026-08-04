use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::lexeme::Type;
use crate::records::parse_node_result::ParseNodeResult;
use crate::records::parse_options::ParseOptions;
use crate::records::parser::Parser;
use luaur_common::macros::luau_timetrace_scope::LUAU_TIMETRACE_SCOPE;

impl Parser {
    pub fn run_parse<Node, F>(
        buffer: &str,
        buffer_size: usize,
        names: &mut AstNameTable,
        allocator: &mut Allocator,
        options: ParseOptions,
        f: F,
    ) -> ParseNodeResult<Node>
    where
        F: FnOnce(&mut Parser) -> *mut Node,
    {
        LUAU_TIMETRACE_SCOPE!("Parser::parse", "Parser");

        // Silence the default panic-hook noise for the parser's exception-
        // emulation unwinds (a caught `ParseError` is a normal syntax/limit
        // error, not a crash).
        crate::functions::install_parse_error_panic_hook::install_parse_error_panic_hook();

        let mut p = Parser::new(buffer, names, allocator as *mut Allocator, options);

        // C++ try-catch is mapped to a result-like handling of ParseError panics if they occur,
        // but the source uses a catch-block for fatal errors. In Luau's Parser, ParseError
        // is often thrown via a panic-like mechanism in Rust or handled via explicit checks.
        // Following the C++ logic:
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let expr = f(&mut p);
            let current_lexeme = p.lexer.current();

            let mut lines = current_lexeme.location.end.line;
            if buffer_size > 0 && buffer.as_bytes()[buffer_size - 1] != b'\n' {
                lines += 1;
            }

            let eof = p.lexer.next();
            let mut root = expr;

            if eof.r#type != Type::Eof {
                root = core::ptr::null_mut();
                p.parse_errors
                    .push(crate::records::parse_error::ParseError::new(
                        eof.location,
                        "Expected end of file".to_string(),
                    ));
            }

            ParseNodeResult {
                root,
                lines: lines as usize,
                hotcomments: std::mem::take(&mut p.hotcomments),
                errors: std::mem::take(&mut p.parse_errors),
                comment_locations: std::mem::take(&mut p.comment_locations),
                cst_node_map: core::mem::replace(
                    &mut p.cst_node_map,
                    luaur_common::records::dense_hash_map::DenseHashMap::new(core::ptr::null_mut()),
                ),
            }
        }));

        match result {
            Ok(res) => res,
            Err(payload) => {
                // If it's a ParseError (the C++ catch (ParseError& err) case)
                if let Some(err) = payload.downcast_ref::<crate::records::parse_error::ParseError>()
                {
                    p.parse_errors.push(err.clone());

                    ParseNodeResult {
                        root: core::ptr::null_mut(),
                        lines: 0,
                        hotcomments: Vec::new(),
                        errors: std::mem::take(&mut p.parse_errors),
                        comment_locations: Vec::new(),
                        cst_node_map: core::mem::replace(
                            &mut p.cst_node_map,
                            luaur_common::records::dense_hash_map::DenseHashMap::new(
                                core::ptr::null_mut(),
                            ),
                        ),
                    }
                } else {
                    // Re-panic if it's not a ParseError
                    std::panic::resume_unwind(payload);
                }
            }
        }
    }
}
