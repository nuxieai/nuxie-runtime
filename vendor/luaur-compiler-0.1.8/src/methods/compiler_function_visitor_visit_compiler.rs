use crate::records::function_visitor::FunctionVisitor;
use luaur_ast::records::ast_expr_function::AstExprFunction;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn visit_ast_expr_function(this: &mut FunctionVisitor<'_>, node: *mut AstExprFunction) -> bool {
    unsafe {
        if node.is_null() {
            return false;
        }

        let node_ref = &*node;

        // node->body->visit(this);
        if !node_ref.body.is_null() {
            luaur_ast::visit::ast_stat_block_visit(&*node_ref.body, this);
        }

        // for (AstLocal* arg : node->args)
        //     hasTypes |= arg->annotation != nullptr;
        for i in 0..node_ref.args.size {
            let arg_ptr = *node_ref.args.data.add(i);
            if !arg_ptr.is_null() {
                let arg = &*arg_ptr;
                if !arg.annotation.is_null() {
                    this.has_types = true;
                }
            }
        }

        // LUAU_ASSERT(functions.end() == std::find(functions.begin(), functions.end(), node));
        let functions = &*this.functions;
        let mut found = false;
        for &f in functions.iter() {
            if f == node {
                found = true;
                break;
            }
        }
        LUAU_ASSERT!(!found);

        // functions.push_back(node);
        this.functions.push(node);

        // if (!hasNativeFunction && node->hasNativeAttribute())
        //     hasNativeFunction = true;
        if !this.has_native_function && node_ref.has_native_attribute() {
            this.has_native_function = true;
        }
    }

    false
}

impl<'a> FunctionVisitor<'a> {
    pub fn visit_ast_expr_function(&mut self, node: *mut AstExprFunction) -> bool {
        visit_ast_expr_function(self, node)
    }
}
