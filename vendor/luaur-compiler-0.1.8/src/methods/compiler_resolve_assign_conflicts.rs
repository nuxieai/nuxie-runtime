use crate::enums::kind::Kind;
use crate::records::assignment::Assignment;
use crate::records::compiler::Compiler;
use crate::records::visitor::Visitor;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_stat::AstStat;
use luaur_ast::visit::ast_expr_visit;

impl Compiler {
    pub fn resolve_assign_conflicts(
        &mut self,
        stat: *mut AstStat,
        vars: &mut Vec<Assignment>,
        values: &AstArray<*mut AstExpr>,
    ) {
        let mut visitor = self.visitor_visitor();

        for i in 0..vars.len() {
            let li = &vars[i].lvalue;
            if li.kind == Kind::Kind_Local {
                if i < values.size {
                    unsafe {
                        ast_expr_visit(*values.data.add(i), &mut visitor);
                    }
                }
                let reg = li.reg as usize;
                visitor.assigned[reg / 64] |= 1 << (reg % 64);
            }
        }

        for i in 0..vars.len() {
            let li = &vars[i].lvalue;
            if li.kind != Kind::Kind_Local && i < values.size {
                unsafe {
                    ast_expr_visit(*values.data.add(i), &mut visitor);
                }
            }
        }

        for i in vars.len()..values.size {
            unsafe {
                ast_expr_visit(*values.data.add(i), &mut visitor);
            }
        }

        for var in vars.iter() {
            let li = &var.lvalue;
            if (li.kind == Kind::Kind_IndexName
                || li.kind == Kind::Kind_IndexNumber
                || li.kind == Kind::Kind_IndexExpr)
            {
                let reg = li.reg as usize;
                if (visitor.assigned[reg / 64] & (1 << (reg % 64))) != 0 {
                    visitor.conflict[reg / 64] |= 1 << (reg % 64);
                }
            }
            if li.kind == Kind::Kind_IndexExpr {
                let idx = li.index as usize;
                if (visitor.assigned[idx / 64] & (1 << (idx % 64))) != 0 {
                    visitor.conflict[idx / 64] |= 1 << (idx % 64);
                }
            }
        }

        for var in vars.iter_mut() {
            let li = &var.lvalue;
            if li.kind == Kind::Kind_Local {
                let reg = li.reg as usize;
                if (visitor.conflict[reg / 64] & (1 << (reg % 64))) != 0 {
                    var.conflict_reg =
                        self.alloc_reg(stat as *mut luaur_ast::records::ast_node::AstNode, 1);
                }
            }
        }
    }
}
