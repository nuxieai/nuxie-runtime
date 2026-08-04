use crate::records::ast_stat_if::AstStatIf;
use crate::records::printer::Printer;

impl<'a> Printer<'a> {
    pub fn visualize_else_if(&mut self, elseif: &mut AstStatIf) {
        unsafe { self.visualize_ast_expr(&mut *elseif.condition) };

        if let Some(ref loc) = elseif.then_location {
            self.advance(&loc.begin);
        }

        self.writer.keyword("then");

        unsafe { self.visualize_block_ast_stat_block(&mut *elseif.thenbody) };

        if elseif.elsebody.is_null() {
            self.advance(unsafe { &(*elseif.thenbody).base.base.location.end });
            self.writer.keyword("end");
        } else if let Some(elseifelseif) = unsafe {
            crate::rtti::ast_node_as::<AstStatIf>(
                elseif.elsebody as *mut crate::records::ast_node::AstNode,
            )
            .as_mut()
        } {
            if let Some(ref loc) = elseif.else_location {
                self.advance(&loc.begin);
            }
            self.writer.keyword("elseif");
            unsafe { self.visualize_else_if(elseifelseif) };
        } else {
            if let Some(ref loc) = elseif.else_location {
                self.advance(&loc.begin);
            }
            self.writer.keyword("else");

            unsafe { self.visualize_block_ast_stat(&mut *elseif.elsebody) };
            self.advance(unsafe { &(*elseif.elsebody).base.location.end });
            self.writer.keyword("end");
        }
    }
}
