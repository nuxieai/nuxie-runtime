use crate::records::allocator::Allocator;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_instantiate::AstExprInstantiate;
use crate::records::cst_expr_explicit_type_instantiation::CstExprExplicitTypeInstantiation;
use crate::records::cst_type_instantiation::CstTypeInstantiation;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use luaur_common::FFlag;

impl Parser {
    pub fn parse_explicit_type_instantiation_expr(
        &mut self,
        start: Position,
        based_on_expr: &mut AstExpr,
    ) -> *mut AstExpr {
        let mut cst_node: *mut CstExprExplicitTypeInstantiation = core::ptr::null_mut();
        if self.options.store_cst_data {
            cst_node = unsafe {
                (*self.allocator).alloc(CstExprExplicitTypeInstantiation::new(
                    CstTypeInstantiation::default(),
                ))
            };
        }

        let mut end_location = Location::default();
        let types_or_packs = self.parse_type_instantiation_expr(
            if !cst_node.is_null() {
                unsafe { &mut (*cst_node).instantiation }
            } else {
                core::ptr::null_mut()
            },
            Some(&mut end_location),
        );

        let expr = unsafe {
            (*self.allocator).alloc(AstExprInstantiate::new(
                Location::new(start, end_location.end),
                based_on_expr as *mut AstExpr,
                types_or_packs,
            ))
        };

        if self.options.store_cst_data {
            self.cst_node_map.try_insert(
                expr as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        expr as *mut AstExpr
    }
}
