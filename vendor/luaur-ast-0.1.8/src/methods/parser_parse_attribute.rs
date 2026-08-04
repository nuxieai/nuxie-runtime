use crate::records::ast_array::AstArray;
use crate::records::ast_attr::{AstAttr, AstAttrType};
use crate::records::ast_expr::AstExpr;
use crate::records::ast_name::AstName;
use crate::records::cst_attr::CstAttr;
use crate::records::cst_node::CstNode;
use crate::records::lexeme::Type;
use crate::records::parser::Parser;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_attribute(&mut self, attributes: &mut TempVector<'_, *mut AstAttr>) {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauCstAttr.get());
        luaur_common::LUAU_ASSERT!(self.lexer.current().r#type == Type::Attribute);

        let empty: AstArray<*mut AstExpr> = AstArray::default();
        let loc = self.lexer.current().location;
        let name = unsafe { self.lexer.current().data.name };
        let name_str = unsafe { core::ffi::CStr::from_ptr(name).to_string_lossy() };
        let ty = self.validate_attribute(loc, &name_str, attributes, &empty);

        self.next_lexeme();

        let node = unsafe {
            (*self.allocator).alloc(
                AstAttr::ast_attr_location_type_item_ast_array_ast_expr_ast_name(
                    loc,
                    ty.unwrap_or(AstAttrType::Unknown),
                    empty,
                    AstName { value: name },
                ),
            )
        };
        attributes.push_back(node);

        if self.options.store_cst_data {
            let cst_node = unsafe { (*self.allocator).alloc(CstAttr::new(true)) };
            self.cst_node_map.try_insert(
                node as *mut crate::records::ast_node::AstNode,
                cst_node as *mut CstNode,
            );
        }
    }
}
