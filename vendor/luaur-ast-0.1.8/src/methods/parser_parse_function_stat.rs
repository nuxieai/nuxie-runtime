use crate::enums::type_lexer::Type;
use crate::functions::is_expr_l_value::is_expr_l_value;
use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_name::AstName;
use crate::records::ast_stat_function::AstStatFunction;
use crate::records::cst_stat_function::CstStatFunction;
use crate::records::lexeme::Lexeme;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;

impl Parser {
    pub fn parse_function_stat(
        &mut self,
        attributes: &AstArray<*mut AstAttr>,
    ) -> *mut AstStatFunction {
        let start = if attributes.size > 0 {
            unsafe { (**attributes.data).base.location }
        } else {
            self.lexer.current().location
        };

        let match_function = *self.lexer.current();
        self.next_lexeme();

        let mut hasself = false;
        let mut debugname = AstName::new();
        let expr = self.parse_function_name(&mut hasself, &mut debugname);

        if luaur_common::FFlag::LuauConst2.get() && !is_expr_l_value(expr) {
            let expr = if luaur_common::FFlag::LuauExportValueSyntax.get()
                && luaur_common::FFlag::LuauConst2.get()
            {
                self.report_l_value_error(expr)
            } else {
                let expressions = self.copy_initializer_list_t(&[expr]);
                self.report_expr_error(
                    unsafe { (*expr).base.location },
                    expressions,
                    format_args!("Assigned expression must be a variable or a field"),
                )
            };
            return expr as *mut AstStatFunction;
        }

        self.match_recovery_stop_on_token[Type::ReservedEnd.0 as usize] += 1;

        let (body, _) = self.parse_function_body(
            hasself,
            &match_function,
            &debugname,
            None,
            attributes,
            false,
        );

        self.match_recovery_stop_on_token[Type::ReservedEnd.0 as usize] -= 1;

        let node = unsafe {
            (*self.allocator).alloc(AstStatFunction::new(
                Location::new(start.begin, (*body).base.base.location.end),
                expr,
                body,
            ))
        };

        if self.options.store_cst_data {
            let cst_node = unsafe {
                (*self.allocator).alloc(CstStatFunction::new(match_function.location.begin))
            };
            self.cst_node_map.try_insert(
                node as *mut crate::records::ast_node::AstNode,
                cst_node as *mut crate::records::cst_node::CstNode,
            );
        }

        node as *mut AstStatFunction
    }
}
