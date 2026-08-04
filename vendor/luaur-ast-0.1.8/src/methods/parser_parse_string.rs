use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_string::AstExprConstantString;
use crate::records::cst_expr_constant_string::CstExprConstantString;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use core::ffi::c_char;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn parse_string(&mut self) -> *mut AstExpr {
        let location: Location = self.lexer.current().location;

        let style = match self.lexer.current().r#type {
            Type::QuotedString | Type::InterpStringSimple => AstExprConstantString::QuotedSimple,
            Type::RawString => AstExprConstantString::QuotedRaw,
            _ => {
                LUAU_ASSERT!(false);
                AstExprConstantString::QuotedSimple
            }
        };

        let mut full_style = crate::enums::quote_style_cst::QuoteStyle::QuotedDouble;
        let mut block_depth: u32 = 0;

        if self.options.store_cst_data {
            let (fs, bd) = self.extract_string_details();
            full_style = fs;
            block_depth = bd;
        }

        let mut original_string = AstArray::<c_char> {
            data: core::ptr::null_mut(),
            size: 0,
        };

        if let Some(value) = self.parse_char_array(if self.options.store_cst_data {
            Some(&mut original_string)
        } else {
            None
        }) {
            let node = unsafe {
                (*self.allocator).alloc(AstExprConstantString {
                    base: AstExpr {
                        base: crate::records::ast_node::AstNode {
                            class_index:
                                <AstExprConstantString as crate::rtti::AstNodeClass>::CLASS_INDEX,
                            location,
                        },
                    },
                    value,
                    quote_style: style,
                })
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstExprConstantString {
                        base: crate::records::cst_node::CstNode {
                            class_index:
                                <CstExprConstantString as crate::rtti::CstNodeClass>::CLASS_INDEX,
                        },
                        source_string: original_string,
                        quote_style: full_style,
                        block_depth,
                    })
                };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }

            node as *mut AstExpr
        } else {
            self.report_expr_error(
                location,
                AstArray {
                    data: core::ptr::null_mut(),
                    size: 0,
                },
                format_args!("String literal contains malformed escape sequence"),
            ) as *mut AstExpr
        }
    }
}
