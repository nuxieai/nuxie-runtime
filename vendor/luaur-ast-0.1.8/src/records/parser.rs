use crate::enums::ast_table_access::AstTableAccess;
use crate::enums::separator::Separator;
use crate::records::allocator::Allocator;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_declared_extern_type_property::AstDeclaredExternTypeProperty;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::ast_name_table::AstNameTable;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_stat_error::AstStatError;
use crate::records::ast_type::AstType;
use crate::records::ast_type_error::AstTypeError;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::binding::Binding;
use crate::records::comment::Comment;
use crate::records::cst_type_instantiation::CstTypeInstantiation;
use crate::records::function::Function;
use crate::records::hot_comment::HotComment;
use crate::records::lexeme::Lexeme;
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::name::Name;
use crate::records::parse_error::ParseError;
use crate::records::parse_options::ParseOptions;
use crate::records::parse_result::ParseResult;
use crate::records::position::Position;
use crate::records::table_indexer_result::TableIndexerResult;
use crate::type_aliases::ast_argument_name::AstArgumentName;
use crate::type_aliases::ast_class_member::AstClassMember;
use crate::type_aliases::cst_node_map::CstNodeMap;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Parser {
    pub(crate) options: ParseOptions,
    pub(crate) lexer: Lexer,
    pub(crate) allocator: *mut Allocator,
    pub(crate) comment_locations: Vec<Comment>,
    pub(crate) hotcomments: Vec<HotComment>,
    pub(crate) hotcomment_header: bool,
    pub(crate) recursion_counter: u32,
    pub(crate) name_self: AstName,
    pub(crate) name_number: AstName,
    pub(crate) name_error: AstName,
    pub(crate) name_nil: AstName,
    pub(crate) end_mismatch_suspect: Option<MatchLexeme>,
    pub(crate) function_stack: Vec<Function>,
    pub(crate) type_function_depth: usize,
    pub(crate) local_map: DenseHashMap<AstName, *mut AstLocal>,
    pub(crate) local_stack: Vec<*mut AstLocal>,
    pub(crate) classes_within_module: DenseHashSet<AstName>,
    pub(crate) parse_errors: Vec<ParseError>,
    pub(crate) match_recovery_stop_on_token: Vec<u32>,
    pub(crate) declared_export_bindings: DenseHashMap<AstName, Location>,
    pub(crate) has_module_return: bool,
    pub(crate) scratch_attr: Vec<*mut AstAttr>,
    pub(crate) scratch_stat: Vec<*mut AstStat>,
    pub(crate) scratch_string: Vec<AstArray<core::ffi::c_char>>,
    pub(crate) scratch_string_2: Vec<AstArray<core::ffi::c_char>>,
    pub(crate) scratch_expr: Vec<*mut AstExpr>,
    pub(crate) scratch_expr_aux: Vec<*mut AstExpr>,
    pub(crate) scratch_name: Vec<AstName>,
    pub(crate) scratch_pack_name: Vec<AstName>,
    pub(crate) scratch_binding: Vec<Binding>,
    pub(crate) scratch_local: Vec<*mut AstLocal>,
    pub(crate) scratch_table_type_props: Vec<crate::records::ast_table_prop::AstTableProp>,
    pub(crate) scratch_cst_table_type_props: Vec<crate::records::cst_type_table::Item>,
    pub(crate) scratch_type: Vec<*mut AstType>,
    pub(crate) scratch_type_or_pack: Vec<AstTypeOrPack>,
    pub(crate) scratch_declared_class_props: Vec<AstDeclaredExternTypeProperty>,
    pub(crate) scratch_class_declarations: Vec<AstClassMember>,
    pub(crate) scratch_item: Vec<crate::records::ast_expr_table::Item>,
    pub(crate) scratch_cst_item: Vec<crate::records::cst_expr_table::Item>,
    pub(crate) scratch_arg_name: Vec<AstArgumentName>,
    pub(crate) scratch_generic_types: Vec<*mut AstGenericType>,
    pub(crate) scratch_generic_type_packs: Vec<*mut AstGenericTypePack>,
    pub(crate) scratch_opt_arg_name: Vec<Option<AstArgumentName>>,
    pub(crate) scratch_position: Vec<Position>,
    pub(crate) scratch_position_2: Vec<Position>,
    pub(crate) scratch_data: String,
    pub(crate) cst_node_map: CstNodeMap,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Local {
    pub(crate) local: *mut AstLocal,
    pub(crate) offset: u32,
}

impl Default for Local {
    fn default() -> Self {
        Self {
            local: core::ptr::null_mut(),
            offset: 0,
        }
    }
}
