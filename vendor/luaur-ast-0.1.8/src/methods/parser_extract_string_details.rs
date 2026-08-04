use crate::enums::quote_style_cst::QuoteStyle;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn extract_string_details(&mut self) -> (QuoteStyle, u32) {
        let mut style: QuoteStyle = QuoteStyle::QuotedDouble;
        let mut block_depth: u32 = 0;

        let current = self.lexer.current();

        match current.r#type {
            Type::QuotedString => {
                style = if current.get_quote_style() == crate::records::lexeme::QuoteStyle::Double {
                    QuoteStyle::QuotedDouble
                } else {
                    QuoteStyle::QuotedSingle
                };
            }
            Type::InterpStringSimple => {
                style = QuoteStyle::QuotedInterp;
            }
            Type::RawString => {
                style = QuoteStyle::QuotedRaw;
                block_depth = current.get_block_depth();
            }
            _ => {
                LUAU_ASSERT!(false);
            }
        }

        (style, block_depth)
    }
}
