use crate::records::ast_type_pack::AstTypePack;
use crate::records::ast_type_pack_explicit::AstTypePackExplicit;
use crate::records::ast_type_pack_generic::AstTypePackGeneric;
use crate::records::ast_type_pack_variadic::AstTypePackVariadic;
use crate::records::cst_type_pack_explicit::CstTypePackExplicit;
use crate::records::cst_type_pack_generic::CstTypePackGeneric;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_type_pack_annotation(
        &mut self,
        annotation: &mut AstTypePack,
        for_var_arg: bool,
        unconditionally_parenthesize: bool,
        for_function_return: bool,
    ) {
        self.advance(&annotation.base.location.begin);

        let variadic_tp =
            unsafe { crate::rtti::ast_node_as::<AstTypePackVariadic>(&mut annotation.base) };
        if let Some(variadic_tp) = unsafe { variadic_tp.as_mut() } {
            if !for_var_arg {
                self.writer.symbol("...");
            }
            unsafe {
                self.visualize_type_annotation(&mut *variadic_tp.variadic_type);
            }
            return;
        }

        let generic_tp =
            unsafe { crate::rtti::ast_node_as::<AstTypePackGeneric>(&mut annotation.base) };
        if let Some(generic_tp) = unsafe { generic_tp.as_mut() } {
            let name_str = unsafe {
                core::ffi::CStr::from_ptr(generic_tp.generic_name.value).to_string_lossy()
            };
            self.writer.symbol(&name_str);

            if let Some(cst_node) = unsafe {
                self.lookup_cst_node::<CstTypePackGeneric>(&mut annotation.base as *mut _)
                    .as_mut()
            } {
                self.advance(&cst_node.ellipsis_position);
            }

            self.writer.symbol("...");
            return;
        }

        let explicit_tp =
            unsafe { crate::rtti::ast_node_as::<AstTypePackExplicit>(&mut annotation.base) };
        if let Some(explicit_tp) = unsafe { explicit_tp.as_mut() } {
            luaur_common::macros::luau_assert::LUAU_ASSERT!(!for_var_arg);

            let cst_node = unsafe {
                self.lookup_cst_node::<CstTypePackExplicit>(&mut annotation.base as *mut _)
            };

            if let Some(cst_node) = unsafe { cst_node.as_mut() } {
                self.visualize_type_list(
                    &explicit_tp.type_list,
                    false,
                    cst_node.open_parentheses_position,
                    cst_node.close_parentheses_position,
                    cst_node.comma_positions,
                );
                return;
            }

            if for_function_return {
                let pack_size = explicit_tp.type_list.types.size
                    + if !explicit_tp.type_list.tail_type.is_null() {
                        1
                    } else {
                        0
                    };

                self.visualize_type_list(
                    &explicit_tp.type_list,
                    pack_size != 1,
                    crate::records::position::Position::missing(),
                    crate::records::position::Position::missing(),
                    crate::records::ast_array::AstArray::default(),
                );
                return;
            }

            self.visualize_type_list(
                &explicit_tp.type_list,
                unconditionally_parenthesize,
                crate::records::position::Position::missing(),
                crate::records::position::Position::missing(),
                crate::records::ast_array::AstArray::default(),
            );
            return;
        }

        luaur_common::macros::luau_assert::LUAU_ASSERT!(false);
    }
}
