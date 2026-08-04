use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_visitor::AstVisitor;

#[derive(Debug)]
pub struct FunctionVisitor<'a> {
    pub(crate) functions: &'a mut Vec<*mut AstExprFunction>,
    pub(crate) has_types: bool,
    pub(crate) has_native_function: bool,
}

impl<'a> FunctionVisitor<'a> {
    pub fn new(functions: &'a mut Vec<*mut AstExprFunction>) -> Self {
        functions.reserve(16);
        Self {
            functions,
            has_types: false,
            has_native_function: false,
        }
    }
}

impl<'a> AstVisitor for FunctionVisitor<'a> {
    fn visit_expr_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = node as *mut AstExprFunction;
        let node_ref = unsafe { &*node };

        unsafe {
            luaur_ast::visit::ast_stat_visit(
                node_ref.body as *mut luaur_ast::records::ast_stat::AstStat,
                self,
            );
        }

        for arg_ptr in node_ref.args.iter() {
            let arg = unsafe { &**arg_ptr };
            self.has_types |= !arg.annotation.is_null();
        }

        // this makes sure all functions that are used when compiling this one have been already added to the vector
        luaur_common::LUAU_ASSERT!(self.functions.iter().all(|&f| f != node));
        self.functions.push(node);

        if !self.has_native_function && node_ref.has_native_attribute() {
            self.has_native_function = true;
        }

        false
    }

    fn visit_stat_type_function(&mut self, _node: *mut core::ffi::c_void) -> bool {
        false
    }
}
