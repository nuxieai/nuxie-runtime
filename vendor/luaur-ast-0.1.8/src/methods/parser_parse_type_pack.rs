use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_generic::AstTypePackGeneric;
use crate::records::ast_type_pack_variadic::AstTypePackVariadic;
use crate::records::cst_type_pack_generic::CstTypePackGeneric;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::parser::Parser;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Parser {
    pub fn parse_type_pack(&mut self) -> *mut AstTypePack {
        if self.lexer.current().r#type == Type::Dot3 {
            let start = self.lexer.current().location;
            self.next_lexeme();
            let vararg_ty = self.parse_type_bool(false);
            unsafe {
                (*self.allocator).alloc(AstTypePackVariadic::new(
                    Location::new(start.begin, (*vararg_ty).base.location.end),
                    vararg_ty,
                )) as *mut AstTypePack
            }
        } else if self.lexer.current().r#type == Type::Name
            && self.lexer.lookahead().r#type == Type::Dot3
        {
            let name = self.parse_name("generic name");
            let end = self.lexer.current().location;
            self.expect_and_consume_type(Type::Dot3, "generic type pack annotation");
            let node = unsafe {
                (*self.allocator).alloc(AstTypePackGeneric::new(
                    Location::new(name.location.begin, end.end),
                    name.name,
                ))
            };
            if self.options.store_cst_data {
                let cst_node =
                    unsafe { (*self.allocator).alloc(CstTypePackGeneric::new(end.begin)) };
                self.cst_node_map.try_insert(
                    node as *mut crate::records::ast_node::AstNode,
                    cst_node as *mut crate::records::cst_node::CstNode,
                );
            }
            node as *mut AstTypePack
        } else {
            LUAU_ASSERT!(false);
            core::ptr::null_mut()
        }
    }
}
