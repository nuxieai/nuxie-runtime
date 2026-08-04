use crate::records::ast_expr::AstExpr;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_expr_list(
        &mut self,
        result: &mut TempVector<'_, *mut AstExpr>,
        mut comma_positions: Option<&mut TempVector<'_, Position>>,
    ) {
        result.push_back(self.parse_expr_i32(0));

        while self.lexer.current().r#type == Type(',' as i32) {
            if let Some(ref mut positions) = comma_positions {
                positions.push_back(self.lexer.current().location.begin);
            }

            self.next_lexeme();

            if self.lexer.current().r#type == Type(')' as i32) {
                self.report(
                    self.lexer.current().location,
                    format_args!("Expected expression after ',' but got ')' instead"),
                );
                break;
            }

            result.push_back(self.parse_expr_i32(0));
        }
    }
}
