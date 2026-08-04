use crate::records::compiler::Compiler;
use luaur_ast::records::ast_array::AstArray;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn compile_expr_list_temp(
        &mut self,
        list: &AstArray<*mut AstExpr>,
        target: u8,
        target_count: u8,
        target_top: bool,
    ) {
        LUAU_ASSERT!(!target_top || (target as u32 + target_count as u32) == self.reg_top);

        if list.size == target_count as usize {
            for i in 0..list.size {
                let expr = unsafe { *list.data.add(i) };
                self.compile_expr_temp(expr, target.wrapping_add(i as u8));
            }
        } else if list.size > target_count as usize {
            for i in 0..target_count as usize {
                let expr = unsafe { *list.data.add(i) };
                self.compile_expr_temp(expr, target.wrapping_add(i as u8));
            }

            for i in target_count as usize..list.size {
                let expr = unsafe { *list.data.add(i) };
                self.compile_expr_side(expr);
            }
        } else if list.size > 0 {
            for i in 0..list.size - 1 {
                let expr = unsafe { *list.data.add(i) };
                self.compile_expr_temp(expr, target.wrapping_add(i as u8));
            }

            let last_expr = unsafe { *list.data.add(list.size - 1) };
            self.compile_expr_temp_n(
                last_expr,
                target.wrapping_add((list.size - 1) as u8),
                target_count.wrapping_sub((list.size - 1) as u8),
                target_top,
            );
        } else {
            for i in 0..target_count {
                unsafe {
                    let bytecode = &mut *self.bytecode;
                    bytecode.emit_abc(LuauOpcode::LOP_LOADNIL, target.wrapping_add(i), 0, 0);
                }
            }
        }
    }
}
