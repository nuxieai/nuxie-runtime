use crate::records::ast_expr::AstExpr;
use crate::records::ast_node::AstNode;
use crate::rtti::AstNodeClass;

impl AstNode {
    #[inline]
    pub fn as_expr(&self) -> *mut AstExpr {
        let is_expr = self.class_index
            == crate::records::ast_expr_binary::AstExprBinary::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_call::AstExprCall::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_constant_bool::AstExprConstantBool::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_constant_integer::AstExprConstantInteger::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_constant_nil::AstExprConstantNil::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_constant_number::AstExprConstantNumber::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_constant_string::AstExprConstantString::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_error::AstExprError::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_function::AstExprFunction::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_global::AstExprGlobal::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_group::AstExprGroup::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_if_else::AstExprIfElse::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_index_expr::AstExprIndexExpr::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_index_name::AstExprIndexName::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_instantiate::AstExprInstantiate::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_interp_string::AstExprInterpString::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_local::AstExprLocal::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_table::AstExprTable::CLASS_INDEX
            || self.class_index
                == crate::records::ast_expr_type_assertion::AstExprTypeAssertion::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_unary::AstExprUnary::CLASS_INDEX
            || self.class_index == crate::records::ast_expr_varargs::AstExprVarargs::CLASS_INDEX;

        if is_expr {
            self as *const AstNode as *mut AstExpr
        } else {
            core::ptr::null_mut()
        }
    }

    #[inline]
    pub fn as_expr_const(&self) -> *const AstExpr {
        self.as_expr() as *const AstExpr
    }
}
