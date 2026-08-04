use crate::enums::type_constant_folding::Type;
use crate::functions::sref_compiler_alt_c::sref_ast_array_c_char;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::location::Location;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn get_constant_index(&mut self, node: *mut AstExpr) -> i32 {
        unsafe {
            let c = self.constants.find(&node);
            if c.is_none() || (*c.unwrap()).r#type == Type::Type_Unknown {
                return -1;
            }

            let constant = c.unwrap();
            let mut cid = -1;

            match (*constant).r#type {
                Type::Type_Nil => {
                    cid = (*self.bytecode).add_constant_nil();
                }
                Type::Type_Boolean => {
                    cid = (*self.bytecode).add_constant_boolean((*constant).data.value_boolean);
                }
                Type::Type_Number => {
                    cid = (*self.bytecode).add_constant_number((*constant).data.value_number);
                }
                Type::Type_Integer => {
                    cid = (*self.bytecode).add_constant_integer((*constant).data.value_integer64);
                }
                Type::Type_Vector => {
                    cid = (*self.bytecode).add_constant_vector(
                        (*constant).data.value_vector[0],
                        (*constant).data.value_vector[1],
                        (*constant).data.value_vector[2],
                        (*constant).data.value_vector[3],
                    );
                }
                Type::Type_String => {
                    let string_data = (*constant).get_string();
                    cid = (*self.bytecode).add_constant_string(sref_ast_array_c_char(string_data));
                }
                _ => {
                    LUAU_ASSERT!(false);
                    return -1;
                }
            }

            if cid < 0 {
                CompileError::raise(
                    &(*node).base.location,
                    core::format_args!("Exceeded constant limit; simplify the code to compile"),
                );
            }

            cid
        }
    }
}
