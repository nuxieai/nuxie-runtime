use crate::records::assignment::Assignment;
use crate::records::compiler::Compiler;
use crate::records::reg_scope::RegScope;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_stat_assign::AstStatAssign;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_stat_assign(&mut self, stat: *mut AstStatAssign) {
        unsafe {
            let stat_ref = &*stat;
            let mut rs = self.reg_scope_compiler();

            if stat_ref.vars.size == 1 && stat_ref.values.size == 1 {
                let var = self.compile_l_value(*stat_ref.vars.data, &mut rs);
                if var.kind == crate::enums::kind::Kind::Kind_Local {
                    self.compile_expr(*stat_ref.values.data, var.reg, false);
                } else {
                    let reg = self.compile_expr_auto(*stat_ref.values.data, &mut rs);
                    self.set_debug_line_ast_node(
                        *stat_ref.vars.data as *mut luaur_ast::records::ast_node::AstNode,
                    );
                    self.compile_assign(&var, reg, *stat_ref.vars.data);
                }
                return;
            }

            let mut vars = Vec::with_capacity(stat_ref.vars.size);
            for i in 0..stat_ref.vars.size {
                vars.push(Assignment {
                    lvalue: self.compile_l_value(*stat_ref.vars.data.add(i), &mut rs),
                    conflict_reg: Assignment::kInvalidReg,
                    value_reg: Assignment::kInvalidReg,
                });
            }

            self.resolve_assign_conflicts(stat as *mut _, &mut vars, &stat_ref.values);

            for i in 0..stat_ref.vars.size.min(stat_ref.values.size) {
                let value = *stat_ref.values.data.add(i);
                if i + 1 == stat_ref.values.size && stat_ref.vars.size > stat_ref.values.size {
                    let rest = (stat_ref.vars.size - stat_ref.values.size + 1) as u32;
                    let temp = self.alloc_reg(stat as *mut _, rest);
                    self.compile_expr_temp_n(value, temp, rest as u8, true);
                    for j in i..stat_ref.vars.size {
                        vars[j].value_reg = temp + (j - i) as u8;
                    }
                } else {
                    let var = &mut vars[i];
                    if var.lvalue.kind == crate::enums::kind::Kind::Kind_Local {
                        var.value_reg = if var.conflict_reg == Assignment::kInvalidReg {
                            var.lvalue.reg
                        } else {
                            var.conflict_reg
                        };
                        self.compile_expr(value, var.value_reg, false);
                    } else {
                        var.value_reg = self.compile_expr_auto(value, &mut rs);
                    }
                }
            }

            for i in stat_ref.vars.size..stat_ref.values.size {
                self.compile_expr_side(*stat_ref.values.data.add(i));
            }

            for (i, var) in vars.iter().enumerate() {
                LUAU_ASSERT!(var.value_reg != Assignment::kInvalidReg);
                if var.lvalue.kind != crate::enums::kind::Kind::Kind_Local {
                    self.set_debug_line_location(&var.lvalue.location);
                    let target_expr = if i < stat_ref.vars.size {
                        *stat_ref.vars.data.add(i)
                    } else {
                        core::ptr::null_mut()
                    };
                    self.compile_assign(&var.lvalue, var.value_reg, target_expr);
                }
            }

            for var in vars {
                if var.lvalue.kind == crate::enums::kind::Kind::Kind_Local
                    && var.value_reg != var.lvalue.reg
                {
                    (*self.bytecode).emit_abc(
                        LuauOpcode::LOP_MOVE,
                        var.lvalue.reg,
                        var.value_reg,
                        0,
                    );
                }
            }
        }
    }
}
