use crate::enums::global::Global;
use crate::enums::table_constant_kind::TableConstantKind;
use crate::records::builtin_ast_types::BuiltinAstTypes;
use crate::records::capture::Capture;
use crate::records::constant::Constant;
use crate::records::function::Function;
use crate::records::inline_frame::InlineFrame;
use crate::records::local::Local;
use crate::records::loop_jump::LoopJump;
use crate::records::r#loop::Loop;
use crate::records::table_shape::TableShape;
use crate::records::variable::Variable;
use alloc::string::String;
use alloc::vec::Vec;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;
use luaur_common::records::dense_hash_map::DenseHashMap;

#[derive(Debug)]
pub struct Compiler {
    pub bytecode: *mut BytecodeBuilder,
    pub options: crate::records::compile_options::CompileOptions,
    pub functions: DenseHashMap<*mut AstExprFunction, Function>,
    pub locals: DenseHashMap<*mut AstLocal, Local>,
    pub globals: DenseHashMap<AstName, Global>,
    pub variables: DenseHashMap<*mut AstLocal, Variable>,
    pub constants: DenseHashMap<*mut AstExpr, Constant>,
    pub locstants: DenseHashMap<*mut AstLocal, Constant>,
    pub table_constants: DenseHashMap<*mut AstLocal, TableConstantKind>,
    pub table_shapes: DenseHashMap<*mut AstExprTable, TableShape>,
    pub builtins: DenseHashMap<*mut AstExprCall, i32>,
    pub userdata_types: DenseHashMap<AstName, u8>,
    pub function_types: DenseHashMap<*mut AstExprFunction, String>,
    pub local_types: DenseHashMap<*mut AstLocal, LuauBytecodeType>,
    pub expr_types: DenseHashMap<*mut AstExpr, LuauBytecodeType>,
    pub inline_builtins: DenseHashMap<*mut AstExprCall, i32>,
    pub inline_builtins_backup: DenseHashMap<*mut AstExprCall, i32>,
    pub expr_changes: Vec<crate::records::expr_constant_change::ExprConstantChange>,
    pub local_changes: Vec<crate::records::local_constant_change::LocalConstantChange>,
    pub builtin_types: BuiltinAstTypes,
    pub names: *mut AstNameTable,
    pub export_table_local: luaur_ast::records::ast_local::AstLocal,
    pub builtins_fold: *const DenseHashMap<*mut AstExprCall, i32>,
    pub builtins_fold_library_k: bool,
    pub reg_top: u32,
    pub stack_size: u32,
    pub arg_count: usize,
    pub has_loops: bool,
    pub current_function: *mut AstExprFunction,
    pub block_depth: usize,
    pub getfenv_used: bool,
    pub setfenv_used: bool,
    pub local_stack: Vec<*mut AstLocal>,
    pub upvals: Vec<*mut AstLocal>,
    pub loop_jumps: Vec<LoopJump>,
    pub loops: Vec<Loop>,
    pub inline_frames: Vec<InlineFrame>,
    pub captures: Vec<Capture>,
    pub exported_locals: Vec<*mut AstLocal>,
    pub exported_classes: Vec<(AstName, u8)>,
}
