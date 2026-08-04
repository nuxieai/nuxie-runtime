use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_call::AstExprCall;
use crate::records::ast_expr_index_name::AstExprIndexName;
use crate::records::ast_type_or_pack::AstTypeOrPack;
use crate::records::cst_expr_call::CstExprCall;
use crate::records::cst_type_instantiation::CstTypeInstantiation;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_method_call(&mut self, start: Position, mut expr: *mut AstExpr) -> *mut AstExpr {
        let op_position = self.lexer.current().location.begin;
        self.next_lexeme();

        let index: Name = self.parse_index_name("method name", &op_position);

        let func = unsafe {
            (*self.allocator).alloc(AstExprIndexName::new(
                Location::new(start, index.location.end),
                expr,
                index.name,
                index.location,
                op_position,
                ':' as i8,
            ))
        };

        let mut type_arguments: crate::records::ast_array::AstArray<AstTypeOrPack> =
            crate::records::ast_array::AstArray {
                data: core::ptr::null_mut(),
                size: 0,
            };

        let mut cst_type_arguments: *mut CstTypeInstantiation = if self.options.store_cst_data {
            unsafe { (*self.allocator).alloc(CstTypeInstantiation::default()) }
        } else {
            core::ptr::null_mut()
        };

        if self.lexer.current().r#type == Type('<' as i32)
            && self.lexer.lookahead().r#type == Type('<' as i32)
        {
            type_arguments = self.parse_type_instantiation_expr(cst_type_arguments, None);
        }

        expr = self.parse_function_args(func as *mut AstExpr, true);

        if self.options.store_cst_data {
            if let Some(cst_node_ptr) = self
                .cst_node_map
                .find(&(expr as *mut crate::records::ast_node::AstNode))
            {
                let cst_node = unsafe { crate::rtti::cst_node_as::<CstExprCall>(*cst_node_ptr) };
                if !cst_node.is_null() {
                    unsafe {
                        (*cst_node).explicit_types = cst_type_arguments;
                    }
                } else {
                    luaur_common::LUAU_ASSERT!(false);
                }
            }
        }

        if !expr.is_null() && type_arguments.size > 0 {
            let call = unsafe {
                crate::rtti::ast_node_as::<AstExprCall>(
                    expr as *mut crate::records::ast_node::AstNode,
                )
            };
            if !call.is_null() {
                unsafe {
                    (*call).type_arguments = type_arguments;
                }
            }
        }

        expr
    }
}
