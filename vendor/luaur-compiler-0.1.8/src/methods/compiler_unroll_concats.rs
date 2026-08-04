use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_binary::{AstExprBinary, AstExprBinary_Op};

use crate::records::compiler::Compiler;

impl Compiler {
    #[allow(non_snake_case)]
    pub fn unroll_concats(&mut self, args: &mut Vec<*mut AstExpr>) {
        loop {
            if args.is_empty() {
                break;
            }

            let back = *args.last().unwrap();
            if back.is_null() {
                break;
            }

            // C++ `args.back()->as<AstExprBinary>()` — a CHECKED RTTI downcast that
            // returns null when the node is not an AstExprBinary. The model used
            // `as_expr() as *mut AstExprBinary`, a blind reinterpret that treats any
            // node (e.g. the trailing string in `a..b..c`) as a Binary and derefs
            // its garbage `op`/`left`/`right` -> SIGSEGV.
            let be = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprBinary>(
                    back as *mut luaur_ast::records::ast_node::AstNode,
                )
            };
            if be.is_null() {
                break;
            }

            let op = unsafe { (*be).op };
            if op != AstExprBinary_Op::Concat {
                break;
            }

            args.pop();
            args.push(unsafe { (*be).left });
            args.push(unsafe { (*be).right });
        }
    }
}
