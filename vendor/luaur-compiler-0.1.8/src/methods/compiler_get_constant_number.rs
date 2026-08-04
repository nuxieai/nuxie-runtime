use crate::enums::type_constant_folding::Type;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr::AstExpr;

impl Compiler {
    pub fn get_constant_number(&mut self, node: *mut AstExpr) -> i32 {
        unsafe {
            let c = self.constants.find(&node);

            if let Some(constant) = c {
                if (*constant).r#type == Type::Type_Number {
                    let cid = (*self.bytecode).add_constant_number((*constant).data.value_number);
                    if cid < 0 {
                        CompileError::raise(
                            &(*node).base.location,
                            core::format_args!(
                                "Exceeded constant limit; simplify the code to compile"
                            ),
                        );
                    }
                    return cid;
                }
            }

            -1
        }
    }
}
