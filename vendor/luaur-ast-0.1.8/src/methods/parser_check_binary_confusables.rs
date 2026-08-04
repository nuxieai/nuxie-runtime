use crate::records::ast_expr_binary::AstExprBinaryOp;
use crate::records::binary_op_priority::BinaryOpPriority;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    pub(crate) fn check_binary_confusables(
        &mut self,
        binary_priority: &[BinaryOpPriority],
        limit: u32,
    ) -> Option<AstExprBinaryOp> {
        let curr = *self.lexer.current();

        // early-out: need to check if this is a possible confusable quickly
        if curr.r#type != Type('&' as i32)
            && curr.r#type != Type('|' as i32)
            && curr.r#type != Type('!' as i32)
        {
            return None;
        }

        // slow path: possible confusable
        let start = curr.location;
        let next = self.lexer.lookahead();

        if curr.r#type == Type('&' as i32)
            && next.r#type == Type('&' as i32)
            && curr.location.end == next.location.begin
            && binary_priority[AstExprBinaryOp::And as usize].left as u32 > limit
        {
            self.next_lexeme();
            self.report(
                Location {
                    begin: start.begin,
                    end: next.location.end,
                },
                format_args!("Unexpected '&&'; did you mean 'and'?"),
            );
            return Some(AstExprBinaryOp::And);
        } else if curr.r#type == Type('|' as i32)
            && next.r#type == Type('|' as i32)
            && curr.location.end == next.location.begin
            && binary_priority[AstExprBinaryOp::Or as usize].left as u32 > limit
        {
            self.next_lexeme();
            self.report(
                Location {
                    begin: start.begin,
                    end: next.location.end,
                },
                format_args!("Unexpected '||'; did you mean 'or'?"),
            );
            return Some(AstExprBinaryOp::Or);
        } else if curr.r#type == Type('!' as i32)
            && next.r#type == Type('=' as i32)
            && curr.location.end == next.location.begin
            && binary_priority[AstExprBinaryOp::CompareNe as usize].left as u32 > limit
        {
            self.next_lexeme();
            self.report(
                Location {
                    begin: start.begin,
                    end: next.location.end,
                },
                format_args!("Unexpected '!='; did you mean '~='?"),
            );
            return Some(AstExprBinaryOp::CompareNe);
        }

        None
    }
}

pub fn parser_check_binary_confusables(
    this: &mut Parser,
    binary_priority: &[BinaryOpPriority],
    limit: u32,
) -> Option<AstExprBinaryOp> {
    this.check_binary_confusables(binary_priority, limit)
}
