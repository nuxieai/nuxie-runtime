use crate::functions::is_expr_l_value::is_expr_l_value;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_return::AstStatReturn;
use crate::records::cst_stat_return::CstStatReturn;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_return(&mut self) -> *mut AstStat {
        let start = self.lexer.current().location;
        self.next_lexeme();

        let mut list = TempVector::new(&mut self.scratch_expr);
        let mut comma_positions = TempVector::new(&mut self.scratch_position);

        if !self.block_follow(self.lexer.current())
            && self.lexer.current().r#type != Type::Semicolon
        {
            self.parse_expr_list(
                &mut list,
                if self.options.store_cst_data {
                    Some(&mut comma_positions)
                } else {
                    None
                },
            );
        }

        let end = if list.empty() {
            start
        } else {
            unsafe { (**list.back()).base.location }
        };

        let list_array = self.copy_temp_vector_t(&list);
        let node = unsafe {
            (*self.allocator).alloc(AstStatReturn::new(
                Location::new(start.begin, end.end),
                list_array,
            ))
        };

        if self.options.store_cst_data {
            let comma_positions_array = self.copy_temp_vector_t(&comma_positions);
            let cst_node =
                unsafe { (*self.allocator).alloc(CstStatReturn::new(comma_positions_array)) };
            self.cst_node_map.try_insert(
                node as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        if luaur_common::FFlag::LuauExportValueSyntax.get()
            && luaur_common::FFlag::LuauConst2.get()
            && self.function_stack.len() == 1
        {
            if !self.declared_export_bindings.is_empty() {
                let expressions = self.copy_initializer_list_t(&[node as *mut AstExpr]);
                return self.report_stat_error(
                    unsafe { (*node).base.base.location },
                    expressions,
                    crate::records::ast_array::AstArray { data: std::ptr::null_mut(), size: 0 },
                    format_args!("Exporting values is not compatible with top-level return (export/return conflict)"),
                ) as *mut AstStat;
            }

            self.has_module_return = true;
        }

        node as *mut AstStat
    }
}
