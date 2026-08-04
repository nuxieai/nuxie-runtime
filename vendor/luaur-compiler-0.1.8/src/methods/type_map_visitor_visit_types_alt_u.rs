use crate::records::type_map_visitor::TypeMapVisitor;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_node::AstNode;
use luaur_ast::records::ast_type::AstType;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction;

pub fn visit_ast_expr_call(this: &mut TypeMapVisitor<'_>, node: *mut AstExprCall) -> bool {
    unsafe {
        if node.is_null() {
            return true;
        }

        let node_ref = &*node;
        let builtin_calls_entry = this.builtin_calls.find(&node);

        if let Some(&bfid) = builtin_calls_entry {
            let builtin = core::mem::transmute::<u8, LuauBuiltinFunction>(bfid as u8);
            match builtin {
                LuauBuiltinFunction::LBF_NONE
                | LuauBuiltinFunction::LBF_ASSERT
                | LuauBuiltinFunction::LBF_RAWSET
                | LuauBuiltinFunction::LBF_RAWGET
                | LuauBuiltinFunction::LBF_TABLE_INSERT
                | LuauBuiltinFunction::LBF_TABLE_UNPACK
                | LuauBuiltinFunction::LBF_SELECT_VARARG
                | LuauBuiltinFunction::LBF_GETMETATABLE
                | LuauBuiltinFunction::LBF_SETMETATABLE
                | LuauBuiltinFunction::LBF_BUFFER_WRITEU8
                | LuauBuiltinFunction::LBF_BUFFER_WRITEU16
                | LuauBuiltinFunction::LBF_BUFFER_WRITEU32
                | LuauBuiltinFunction::LBF_BUFFER_WRITEF32
                | LuauBuiltinFunction::LBF_BUFFER_WRITEF64
                | LuauBuiltinFunction::LBF_BUFFER_WRITEINTEGER => {
                    // break;
                }

                LuauBuiltinFunction::LBF_MATH_ABS
                | LuauBuiltinFunction::LBF_MATH_ACOS
                | LuauBuiltinFunction::LBF_MATH_ASIN
                | LuauBuiltinFunction::LBF_MATH_ATAN2
                | LuauBuiltinFunction::LBF_MATH_ATAN
                | LuauBuiltinFunction::LBF_MATH_CEIL
                | LuauBuiltinFunction::LBF_MATH_COSH
                | LuauBuiltinFunction::LBF_MATH_COS
                | LuauBuiltinFunction::LBF_MATH_DEG
                | LuauBuiltinFunction::LBF_MATH_EXP
                | LuauBuiltinFunction::LBF_MATH_FLOOR
                | LuauBuiltinFunction::LBF_MATH_FMOD
                | LuauBuiltinFunction::LBF_MATH_FREXP
                | LuauBuiltinFunction::LBF_MATH_LDEXP
                | LuauBuiltinFunction::LBF_MATH_LOG10
                | LuauBuiltinFunction::LBF_MATH_LOG
                | LuauBuiltinFunction::LBF_MATH_MAX
                | LuauBuiltinFunction::LBF_MATH_MIN
                | LuauBuiltinFunction::LBF_MATH_MODF
                | LuauBuiltinFunction::LBF_MATH_POW
                | LuauBuiltinFunction::LBF_MATH_RAD
                | LuauBuiltinFunction::LBF_MATH_SINH
                | LuauBuiltinFunction::LBF_MATH_SIN
                | LuauBuiltinFunction::LBF_MATH_SQRT
                | LuauBuiltinFunction::LBF_MATH_TANH
                | LuauBuiltinFunction::LBF_MATH_TAN
                | LuauBuiltinFunction::LBF_BIT32_ARSHIFT
                | LuauBuiltinFunction::LBF_BIT32_BAND
                | LuauBuiltinFunction::LBF_BIT32_BNOT
                | LuauBuiltinFunction::LBF_BIT32_BOR
                | LuauBuiltinFunction::LBF_BIT32_BXOR
                | LuauBuiltinFunction::LBF_BIT32_BTEST
                | LuauBuiltinFunction::LBF_BIT32_EXTRACT
                | LuauBuiltinFunction::LBF_BIT32_LROTATE
                | LuauBuiltinFunction::LBF_BIT32_LSHIFT
                | LuauBuiltinFunction::LBF_BIT32_REPLACE
                | LuauBuiltinFunction::LBF_BIT32_RROTATE
                | LuauBuiltinFunction::LBF_BIT32_RSHIFT
                | LuauBuiltinFunction::LBF_STRING_BYTE
                | LuauBuiltinFunction::LBF_STRING_LEN
                | LuauBuiltinFunction::LBF_MATH_CLAMP
                | LuauBuiltinFunction::LBF_MATH_SIGN
                | LuauBuiltinFunction::LBF_MATH_ROUND
                | LuauBuiltinFunction::LBF_BIT32_COUNTLZ
                | LuauBuiltinFunction::LBF_BIT32_COUNTRZ
                | LuauBuiltinFunction::LBF_RAWLEN
                | LuauBuiltinFunction::LBF_BIT32_EXTRACTK
                | LuauBuiltinFunction::LBF_TONUMBER
                | LuauBuiltinFunction::LBF_BIT32_BYTESWAP
                | LuauBuiltinFunction::LBF_BUFFER_READI8
                | LuauBuiltinFunction::LBF_BUFFER_READU8
                | LuauBuiltinFunction::LBF_BUFFER_READI16
                | LuauBuiltinFunction::LBF_BUFFER_READU16
                | LuauBuiltinFunction::LBF_BUFFER_READI32
                | LuauBuiltinFunction::LBF_BUFFER_READU32
                | LuauBuiltinFunction::LBF_BUFFER_READF32
                | LuauBuiltinFunction::LBF_BUFFER_READF64
                | LuauBuiltinFunction::LBF_VECTOR_MAGNITUDE
                | LuauBuiltinFunction::LBF_VECTOR_DOT
                | LuauBuiltinFunction::LBF_MATH_LERP => {
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.number_type as *const _ as *const AstType,
                    );
                }

                LuauBuiltinFunction::LBF_TYPE
                | LuauBuiltinFunction::LBF_STRING_CHAR
                | LuauBuiltinFunction::LBF_TYPEOF
                | LuauBuiltinFunction::LBF_STRING_SUB
                | LuauBuiltinFunction::LBF_TOSTRING => {
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.string_type as *const _ as *const AstType,
                    );
                }

                LuauBuiltinFunction::LBF_MATH_ISNAN
                | LuauBuiltinFunction::LBF_MATH_ISINF
                | LuauBuiltinFunction::LBF_MATH_ISFINITE
                | LuauBuiltinFunction::LBF_RAWEQUAL => {
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.boolean_type as *const _ as *const AstType,
                    );
                }

                LuauBuiltinFunction::LBF_VECTOR
                | LuauBuiltinFunction::LBF_VECTOR_NORMALIZE
                | LuauBuiltinFunction::LBF_VECTOR_CROSS
                | LuauBuiltinFunction::LBF_VECTOR_FLOOR
                | LuauBuiltinFunction::LBF_VECTOR_CEIL
                | LuauBuiltinFunction::LBF_VECTOR_ABS
                | LuauBuiltinFunction::LBF_VECTOR_SIGN
                | LuauBuiltinFunction::LBF_VECTOR_CLAMP
                | LuauBuiltinFunction::LBF_VECTOR_MIN
                | LuauBuiltinFunction::LBF_VECTOR_MAX
                | LuauBuiltinFunction::LBF_VECTOR_LERP => {
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.vector_type as *const _ as *const AstType,
                    );
                }

                LuauBuiltinFunction::LBF_INTEGER_ADD
                | LuauBuiltinFunction::LBF_INTEGER_SUB
                | LuauBuiltinFunction::LBF_INTEGER_MOD
                | LuauBuiltinFunction::LBF_INTEGER_MUL
                | LuauBuiltinFunction::LBF_INTEGER_DIV
                | LuauBuiltinFunction::LBF_INTEGER_IDIV
                | LuauBuiltinFunction::LBF_INTEGER_UDIV
                | LuauBuiltinFunction::LBF_INTEGER_REM
                | LuauBuiltinFunction::LBF_INTEGER_UREM
                | LuauBuiltinFunction::LBF_INTEGER_MAX
                | LuauBuiltinFunction::LBF_INTEGER_MIN
                | LuauBuiltinFunction::LBF_INTEGER_BAND
                | LuauBuiltinFunction::LBF_INTEGER_BOR
                | LuauBuiltinFunction::LBF_INTEGER_BNOT
                | LuauBuiltinFunction::LBF_INTEGER_BXOR
                | LuauBuiltinFunction::LBF_INTEGER_LSHIFT
                | LuauBuiltinFunction::LBF_INTEGER_RSHIFT
                | LuauBuiltinFunction::LBF_INTEGER_ARSHIFT
                | LuauBuiltinFunction::LBF_INTEGER_LROTATE
                | LuauBuiltinFunction::LBF_INTEGER_RROTATE
                | LuauBuiltinFunction::LBF_INTEGER_EXTRACT
                | LuauBuiltinFunction::LBF_INTEGER_COUNTLZ
                | LuauBuiltinFunction::LBF_INTEGER_COUNTRZ
                | LuauBuiltinFunction::LBF_INTEGER_BSWAP
                | LuauBuiltinFunction::LBF_INTEGER_CLAMP
                | LuauBuiltinFunction::LBF_INTEGER_NEG
                | LuauBuiltinFunction::LBF_INTEGER_CREATE
                | LuauBuiltinFunction::LBF_BUFFER_READINTEGER => {
                    if !luaur_common::FFlag::LuauIntegerFastcalls.get() {
                        return true;
                    }
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.integer_type as *const _ as *const AstType,
                    );
                }

                LuauBuiltinFunction::LBF_INTEGER_TONUMBER => {
                    if !luaur_common::FFlag::LuauIntegerFastcalls.get() {
                        return true;
                    }
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.number_type as *const _ as *const AstType,
                    );
                }

                LuauBuiltinFunction::LBF_INTEGER_LT
                | LuauBuiltinFunction::LBF_INTEGER_LE
                | LuauBuiltinFunction::LBF_INTEGER_GT
                | LuauBuiltinFunction::LBF_INTEGER_GE
                | LuauBuiltinFunction::LBF_INTEGER_ULT
                | LuauBuiltinFunction::LBF_INTEGER_ULE
                | LuauBuiltinFunction::LBF_INTEGER_UGT
                | LuauBuiltinFunction::LBF_INTEGER_UGE
                | LuauBuiltinFunction::LBF_INTEGER_BTEST => {
                    if !luaur_common::FFlag::LuauIntegerFastcalls.get() {
                        return true;
                    }
                    this.record_resolved_type_ast_expr_ast_type(
                        node as *mut AstExpr,
                        &this.builtin_types.boolean_type as *const _ as *const AstType,
                    );
                }
            }
        } else {
            if !node_ref.func.is_null() {
                let local_ptr =
                    luaur_ast::rtti::ast_node_as::<AstExprLocal>(node_ref.func as *mut AstNode);
                if !local_ptr.is_null() {
                    let local = (*local_ptr).local;
                    if let Some(&type_ptr) = this.function_return_types.find(&local) {
                        this.record_resolved_type_ast_expr_ast_type(node as *mut AstExpr, type_ptr);
                    }
                }
            }
        }
    }

    true
}

impl TypeMapVisitor<'_> {
    pub fn visit_ast_expr_call(&mut self, node: *mut AstExprCall) -> bool {
        visit_ast_expr_call(self, node)
    }
}
