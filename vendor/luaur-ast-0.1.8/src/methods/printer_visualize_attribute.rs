use crate::records::ast_attr::AstAttr;
use crate::records::cst_attr::CstAttr;
use crate::records::cst_parametrized_attr::CstParametrizedAttr;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_attribute(&mut self, attribute: &mut AstAttr) {
        self.advance(&attribute.base.location.begin);
        let name_val = attribute.name.value;
        let name_str = unsafe { core::ffi::CStr::from_ptr(name_val).to_string_lossy() };
        let ast_node = &mut attribute.base as *mut crate::records::ast_node::AstNode;
        let cst_node = self.lookup_cst_node::<CstAttr>(ast_node);
        if !cst_node.is_null() {
            if unsafe { (*cst_node).has_at } {
                self.writer.symbol("@");
            }
            self.writer.identifier(&name_str);
            return;
        }

        let cst_param_node = self.lookup_cst_node::<CstParametrizedAttr>(ast_node);
        if !cst_param_node.is_null() {
            self.writer.identifier(&name_str);
            self.maybe_advance_and_write(
                unsafe { &(*cst_param_node).open_paren_position },
                "(",
                false,
            );
            let comma_position_size = unsafe { (*cst_param_node).args_comma_positions.size };
            for i in 0..attribute.args.size {
                self.visualize_ast_expr(unsafe { *attribute.args.data.add(i) });
                if i < comma_position_size {
                    self.maybe_advance_and_write(
                        unsafe { &*(*cst_param_node).args_comma_positions.data.add(i) },
                        ",",
                        false,
                    );
                }
            }
            self.maybe_advance_and_write(
                unsafe { &(*cst_param_node).close_paren_position },
                ")",
                false,
            );
            return;
        }

        self.writer.symbol("@");
        self.writer.identifier(&name_str);
    }
}
