use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_attributed_function(&mut self, start: Location) -> *mut AstExpr {
        let mut cst_attr_lists = TempVector::<*mut CstAttrList>::new(
            &mut self.scratch_cst_attr_list,
        );
        let cst_attr_lists_ptr = &mut cst_attr_lists as *mut _;
        let attributes: AstArray<*mut AstAttr> = self.parse_attributes(cst_attr_lists_ptr);

        if self.lexer.current().r#type != Type::ReservedFunction {
            return self.report_expr_error(
                start,
                AstArray::default(),
                format_args!(
                    "Expected 'function' declaration after attribute, but got {} instead",
                    self.lexer.current().to_string()
                ),
            ) as *mut AstExpr;
        }

        let match_function = *self.lexer.current();
        self.next_lexeme();

        self.parse_function_body(
            false,
            &match_function,
            &AstName::new(),
            None,
            &attributes,
            false,
            cst_attr_lists_ptr,
        )
        .0 as *mut AstExpr
    }
}
