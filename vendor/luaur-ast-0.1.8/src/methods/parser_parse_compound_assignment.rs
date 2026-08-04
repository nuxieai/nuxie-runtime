use crate::functions::is_expr_l_value::is_expr_l_value;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_binary::AstExprBinaryOp;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_compound_assign::AstStatCompoundAssign;
use crate::records::cst_stat_compound_assign::CstStatCompoundAssign;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub fn parse_compound_assignment(
        &mut self,
        mut initial: *mut AstExpr,
        op: AstExprBinaryOp,
    ) -> *mut AstStat {
        // C++ REASSIGNS `initial` to an error node and FALLS THROUGH to
        // nextLexeme()+parseExpr() — it does NOT return here. The port returned
        // early, so when the LHS is not an lvalue the operator (`+=` etc.) was
        // never consumed; parseBlockNoScope's statement loop then re-parsed the
        // same token forever, allocating until the process OOMs (machine crash).
        if !is_expr_l_value(initial) {
            initial = if luaur_common::FFlag::LuauConst2.get() {
                self.report_l_value_error(initial)
            } else {
                let expressions = self.copy_initializer_list_t(&[initial]);
                self.report_expr_error(
                    unsafe { (*initial).base.location },
                    expressions,
                    format_args!("Assigned expression must be a variable or a field"),
                )
            } as *mut AstExpr;
        }

        let op_position = self.lexer.current().location.begin;
        self.next_lexeme();

        let value = self.parse_expr_i32(0);

        let node = unsafe {
            (*self.allocator).alloc(AstStatCompoundAssign::new(
                Location::new(unsafe { (*initial).base.location.begin }, unsafe {
                    (*value).base.location.end
                }),
                op,
                initial,
                value,
            ))
        };

        if self.options.store_cst_data {
            let cst_node =
                unsafe { (*self.allocator).alloc(CstStatCompoundAssign::new(op_position)) };
            self.cst_node_map.try_insert(
                node as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        node as *mut AstStat
    }
}
