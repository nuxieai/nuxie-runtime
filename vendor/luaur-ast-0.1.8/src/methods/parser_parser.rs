use crate::records::allocator::Allocator;
use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;
use crate::records::function::Function;
use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::parse_options::ParseOptions;
use crate::records::parser::Parser;
use crate::records::position::Position;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

// C++ `kParseNameError = "%error-id%"` (ParseResult.h). The port used "(error)".
const K_PARSE_NAME_ERROR: &str = "%error-id%";

impl Parser {
    pub fn new(
        buffer: &str,
        names: &mut AstNameTable,
        allocator: *mut Allocator,
        options: ParseOptions,
    ) -> Self {
        let resume_position = options
            .parse_fragment
            .as_ref()
            .map(|f| f.resume_position)
            .unwrap_or(Position { line: 0, column: 0 });

        let mut parser = Parser {
            options: options.clone(),
            lexer: Lexer::new(
                buffer.as_ptr() as *const core::ffi::c_char,
                buffer.len(),
                names,
                resume_position,
            ),
            allocator,
            comment_locations: alloc::vec::Vec::new(),
            hotcomments: alloc::vec::Vec::new(),
            hotcomment_header: true,
            recursion_counter: 0,
            name_self: AstName::default(),
            name_number: AstName::default(),
            name_error: AstName::default(),
            name_nil: AstName::default(),
            end_mismatch_suspect: None,
            function_stack: alloc::vec::Vec::with_capacity(8),
            type_function_depth: 0,
            local_map: DenseHashMap::new(AstName::default()),
            local_stack: alloc::vec::Vec::with_capacity(16),
            classes_within_module: DenseHashSet::new(AstName::default()),
            parse_errors: alloc::vec::Vec::new(),
            // C++ sizes this `[unsigned(Lexeme::Type::Reserved_END)]`; 256 only
            // covered char tokens and overflowed on reserved-keyword types
            // (ReservedEnd=296 indexed into a len-256 vec).
            match_recovery_stop_on_token: alloc::vec![0; Type::Reserved_END.0 as usize],
            declared_export_bindings: DenseHashMap::new(AstName::default()),
            has_module_return: false,
            scratch_attr: alloc::vec::Vec::new(),
            scratch_stat: alloc::vec::Vec::with_capacity(16),
            scratch_string: alloc::vec::Vec::new(),
            scratch_string_2: alloc::vec::Vec::new(),
            scratch_expr: alloc::vec::Vec::with_capacity(16),
            scratch_expr_aux: alloc::vec::Vec::new(),
            scratch_name: alloc::vec::Vec::new(),
            scratch_pack_name: alloc::vec::Vec::new(),
            scratch_binding: alloc::vec::Vec::with_capacity(16),
            scratch_local: alloc::vec::Vec::with_capacity(16),
            scratch_table_type_props: alloc::vec::Vec::new(),
            scratch_cst_table_type_props: alloc::vec::Vec::new(),
            scratch_type: alloc::vec::Vec::new(),
            scratch_type_or_pack: alloc::vec::Vec::new(),
            scratch_declared_class_props: alloc::vec::Vec::new(),
            scratch_class_declarations: alloc::vec::Vec::new(),
            scratch_item: alloc::vec::Vec::new(),
            scratch_cst_item: alloc::vec::Vec::new(),
            scratch_arg_name: alloc::vec::Vec::new(),
            scratch_generic_types: alloc::vec::Vec::new(),
            scratch_generic_type_packs: alloc::vec::Vec::new(),
            scratch_opt_arg_name: alloc::vec::Vec::new(),
            scratch_position: alloc::vec::Vec::new(),
            scratch_position_2: alloc::vec::Vec::new(),
            scratch_data: alloc::string::String::new(),
            cst_node_map: DenseHashMap::new(core::ptr::null_mut()),
        };

        let mut top = Function {
            vararg: true,
            loop_depth: 0,
        };
        parser.function_stack.push(top);

        parser.name_self = names.get_or_add_c_str(c"self".as_ptr());
        parser.name_number = names.get_or_add_c_str(c"number".as_ptr());
        parser.name_error = names.get_or_add_c_str(c"%error-id%".as_ptr());
        parser.name_nil = names.get_or_add_c_str(c"nil".as_ptr());

        parser.match_recovery_stop_on_token[Type::ReservedEnd.0 as usize] = 0;
        parser.match_recovery_stop_on_token[Type::Eof.0 as usize] = 1;

        parser.lexer.set_skip_comments(true);

        LUAU_ASSERT!(parser.hotcomment_header);
        parser.next_lexeme();

        parser.hotcomment_header = false;

        if let Some(fragment) = &options.parse_fragment {
            parser.local_map = fragment.local_map.clone();
            parser.local_stack = fragment.local_stack.clone();
        }

        parser
    }
}
