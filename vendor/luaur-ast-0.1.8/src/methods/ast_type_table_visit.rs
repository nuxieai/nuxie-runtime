use crate::records::ast_table_prop::AstTableProp;
use crate::records::ast_type_table::AstTypeTable;
use crate::records::ast_visitor::AstVisitor;
use crate::visit::AstVisitable;

impl AstVisitable for AstTypeTable {
    fn visit(&self, visitor: &mut dyn AstVisitor) {
        if visitor.visit_type_table(self as *const Self as *mut core::ffi::c_void) {
            for prop in self.props.iter() {
                unsafe {
                    crate::visit::ast_type_visit(prop.r#type, visitor);
                }
            }

            if !self.indexer.is_null() {
                unsafe {
                    let indexer = &*self.indexer;
                    crate::visit::ast_type_visit(indexer.index_type, visitor);
                    crate::visit::ast_type_visit(indexer.result_type, visitor);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn ast_type_table_visit(this: &AstTypeTable, visitor: &mut dyn AstVisitor) {
    this.visit(visitor);
}
