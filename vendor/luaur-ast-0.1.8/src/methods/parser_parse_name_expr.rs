use crate::records::allocator::Allocator;
use crate::records::ast_array::AstArray;
use crate::records::ast_expr::AstExpr;
use crate::records::ast_expr_error::AstExprError;
use crate::records::ast_expr_global::AstExprGlobal;
use crate::records::ast_expr_local::AstExprLocal;
use crate::records::ast_local::AstLocal;
use crate::records::ast_node::AstNode;
use crate::records::name::Name;
use crate::records::parser::Parser;
use crate::rtti::AstNodeClass;

impl Parser {
    pub fn parse_name_expr(&mut self, context: &str) -> *mut AstExpr {
        let name: Option<Name> = self.parse_name_opt(context);

        if name.is_none() {
            let location = self.lexer.current().location;
            let expressions = self.copy_initializer_list_t::<*mut AstExpr>(&[]);
            let message_index = (self.parse_errors.len() as u32).saturating_sub(1);

            return unsafe {
                Allocator::alloc(
                    &mut *self.allocator,
                    AstExprError {
                        base: AstExpr {
                            base: AstNode {
                                class_index: <AstExprError as AstNodeClass>::CLASS_INDEX,
                                location,
                            },
                        },
                        expressions,
                        message_index,
                    },
                ) as *mut AstExpr
            };
        }

        let name = name.unwrap();
        let value = self.local_map.find(&name.name);

        if let Some(local) = value.copied().filter(|p| !p.is_null()) {
            if unsafe { (*local).function_depth < self.type_function_depth } {
                return self.report_expr_error(
                    self.lexer.current().location,
                    AstArray::default(),
                    format_args!("Type function cannot reference outer local '{}'", unsafe {
                        core::ffi::CStr::from_ptr((*local).name.value).to_string_lossy()
                    }),
                ) as *mut AstExpr;
            }

            let upvalue =
                unsafe { (*local).function_depth != self.function_stack.len().saturating_sub(1) };

            return unsafe {
                Allocator::alloc(
                    &mut *self.allocator,
                    AstExprLocal {
                        base: AstExpr {
                            base: AstNode {
                                class_index: <AstExprLocal as AstNodeClass>::CLASS_INDEX,
                                location: name.location,
                            },
                        },
                        local,
                        upvalue,
                    },
                ) as *mut AstExpr
            };
        }

        unsafe {
            Allocator::alloc(
                &mut *self.allocator,
                AstExprGlobal {
                    base: AstExpr {
                        base: AstNode {
                            class_index: <AstExprGlobal as AstNodeClass>::CLASS_INDEX,
                            location: name.location,
                        },
                    },
                    name: name.name,
                },
            ) as *mut AstExpr
        }
    }
}
