pub trait AstVisitor {
    fn visit_node(&mut self, _node: *mut core::ffi::c_void) -> bool {
        true
    }

    fn visit_attr(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_node(node)
    }

    fn visit_generic_type(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_node(node)
    }

    fn visit_generic_type_pack(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_node(node)
    }

    fn visit_expr(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_node(node)
    }

    fn visit_expr_group(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_constant_nil(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_constant_bool(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_constant_number(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_constant_integer(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_constant_string(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_global(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_varargs(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_call(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_index_name(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_index_expr(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_table(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_unary(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_binary(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_type_assertion(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_if_else(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_interp_string(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_instantiate(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_expr_error(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_expr(node)
    }

    fn visit_stat(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_node(node)
    }

    fn visit_stat_block(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_if(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_while(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_repeat(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_break(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_continue(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_return(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_expr(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_for(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_for_in(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_compound_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_local_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_type_alias(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_type_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_declare_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_declare_global(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_class(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_declare_extern_type(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_stat_error(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_stat(node)
    }

    fn visit_type(&mut self, _node: *mut core::ffi::c_void) -> bool {
        false
    }

    fn visit_type_reference(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_table(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_typeof(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_optional(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_union(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_intersection(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_singleton_bool(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_singleton_string(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_group(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_error(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type(node)
    }

    fn visit_type_pack(&mut self, _node: *mut core::ffi::c_void) -> bool {
        false
    }

    fn visit_type_pack_explicit(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type_pack(node)
    }

    fn visit_type_pack_variadic(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type_pack(node)
    }

    fn visit_type_pack_generic(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_type_pack(node)
    }
}
