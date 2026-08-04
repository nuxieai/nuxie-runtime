use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr_function::AstExprFunction;

impl AstExprFunction {
    pub fn has_native_attribute(&self) -> bool {
        for i in 0..self.attributes.size {
            let attribute_ptr = unsafe { *self.attributes.data.add(i) };
            if !attribute_ptr.is_null() {
                let attribute = unsafe { &*attribute_ptr };
                if attribute.r#type == crate::records::ast_attr::AstAttrType::Native {
                    return true;
                }
            }
        }
        false
    }
}

#[allow(non_snake_case)]
pub fn ast_expr_function_has_native_attribute(this: &AstExprFunction) -> bool {
    this.has_native_attribute()
}
