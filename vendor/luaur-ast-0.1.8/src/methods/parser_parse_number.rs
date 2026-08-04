use crate::enums::constant_number_parse_result::ConstantNumberParseResult;
use crate::functions::parse_double::parse_double;
use crate::functions::parse_integer_64::parse_integer_64;
use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_constant_integer::AstExprConstantInteger;
use crate::records::ast_expr_constant_number::AstExprConstantNumber;
use crate::records::cst_expr_constant_integer::CstExprConstantInteger;
use crate::records::cst_expr_constant_number::CstExprConstantNumber;
use crate::records::parser::Parser;
use luaur_common::FFlag;

impl Parser {
    pub fn parse_number(&mut self) -> *mut AstExpr {
        // scratchData.assign(lexer.current().data, lexer.current().getLength());
        let (start, data_ptr, lexeme_len) = {
            let current = self.lexer.current();
            (
                current.location,
                unsafe { current.data.data } as *const u8,
                current.get_length() as usize,
            )
        };
        self.scratch_data.clear();
        let bytes = unsafe { core::slice::from_raw_parts(data_ptr, lexeme_len) };
        self.scratch_data
            .push_str(core::str::from_utf8(bytes).unwrap_or(""));

        let mut source_data = AstArray::default();
        if self.options.store_cst_data {
            let sd = self.scratch_data.clone();
            source_data = self.copy_string(&sd);
        }

        // Remove all internal _
        if self.scratch_data.contains('_') {
            self.scratch_data.retain(|c| c != '_');
        }

        if FFlag::LuauIntegerType2.get() && self.scratch_data.ends_with('i') {
            let mut value: i64 = 0;
            let result: ConstantNumberParseResult;

            if self.scratch_data.starts_with("0x") || self.scratch_data.starts_with("0X") {
                result = parse_integer_64(&mut value, &self.scratch_data, 16);
            } else if self.scratch_data.starts_with("0b") || self.scratch_data.starts_with("0B") {
                result = parse_integer_64(&mut value, &self.scratch_data[2..], 2);
            } else {
                result = parse_integer_64(&mut value, &self.scratch_data, 10);
            }

            self.next_lexeme();

            if result == ConstantNumberParseResult::Malformed {
                return self.report_expr_error(
                    start,
                    AstArray::default(),
                    format_args!("Malformed integer"),
                ) as *mut AstExpr;
            }

            if result != ConstantNumberParseResult::Ok {
                return self.report_expr_error(
                    start,
                    AstArray::default(),
                    format_args!("Integer overflow"),
                ) as *mut AstExpr;
            }

            let node = unsafe {
                (*self.allocator).alloc(AstExprConstantInteger {
                    base: AstExpr {
                        base: crate::records::ast_node::AstNode {
                            class_index:
                                <AstExprConstantInteger as crate::rtti::AstNodeClass>::CLASS_INDEX,
                            location: start,
                        },
                    },
                    value,
                    parse_result: result,
                })
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstExprConstantInteger {
                        base: crate::records::cst_node::CstNode {
                            class_index:
                                <CstExprConstantInteger as crate::rtti::CstNodeClass>::CLASS_INDEX,
                        },
                        value: source_data,
                    })
                };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }

            node as *mut AstExpr
        } else {
            let mut value: f64 = 0.0;
            let result = parse_double(&mut value, &self.scratch_data);

            self.next_lexeme();

            if result == ConstantNumberParseResult::Malformed {
                return self.report_expr_error(
                    start,
                    AstArray::default(),
                    format_args!("Malformed number"),
                ) as *mut AstExpr;
            }

            let node = unsafe {
                (*self.allocator).alloc(AstExprConstantNumber {
                    base: AstExpr {
                        base: crate::records::ast_node::AstNode {
                            class_index:
                                <AstExprConstantNumber as crate::rtti::AstNodeClass>::CLASS_INDEX,
                            location: start,
                        },
                    },
                    value,
                    parse_result: result,
                })
            };

            if self.options.store_cst_data {
                let cst_node = unsafe {
                    (*self.allocator).alloc(CstExprConstantNumber {
                        base: crate::records::cst_node::CstNode {
                            class_index:
                                <CstExprConstantNumber as crate::rtti::CstNodeClass>::CLASS_INDEX,
                        },
                        value: source_data,
                    })
                };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }

            node as *mut AstExpr
        }
    }
}
