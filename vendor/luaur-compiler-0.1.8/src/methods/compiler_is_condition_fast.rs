use crate::enums::type_constant_folding::Type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::AstExprBinary;
use luaur_ast::records::ast_expr_binary::AstExprBinary_Op;
use luaur_ast::records::ast_expr_group::AstExprGroup;
use luaur_ast::records::ast_node::AstNode;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl Compiler {
    pub fn is_condition_fast(&mut self, node: *mut AstExpr) -> bool {
        unsafe {
            if node.is_null() {
                return false;
            }

            let cv = self.constants.find(&node);

            if let Some(constant) = cv {
                if (*constant).r#type != Type::Type_Unknown {
                    return true;
                }
            }

            let binary = luaur_ast::rtti::ast_node_as::<AstExprBinary>(node as *mut AstNode);
            if !binary.is_null() {
                match (*binary).op {
                    AstExprBinary_Op::And
                    | AstExprBinary_Op::Or
                    | AstExprBinary_Op::CompareNe
                    | AstExprBinary_Op::CompareEq
                    | AstExprBinary_Op::CompareLt
                    | AstExprBinary_Op::CompareLe
                    | AstExprBinary_Op::CompareGt
                    | AstExprBinary_Op::CompareGe => return true,
                    _ => return false,
                }
            }

            let group = luaur_ast::rtti::ast_node_as::<AstExprGroup>(node as *mut AstNode);
            if !group.is_null() {
                return self.is_condition_fast((*group).expr);
            }

            false
        }
    }
}
