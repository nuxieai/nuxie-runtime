use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_expr_function::AstExprFunction;
use crate::records::ast_local::AstLocal;
use crate::records::ast_name::AstName;
use crate::records::lexeme::Lexeme;
use crate::records::name::Name;
use crate::records::parser::Parser;

use crate::records::ast_generic_type::AstGenericType;
use crate::records::ast_generic_type_pack::AstGenericTypePack;
use crate::records::ast_node::AstNode;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::ast_type::AstType;
use crate::records::ast_type_pack::AstTypePack;
use crate::records::binding::Binding;
use crate::records::cst_expr_function::CstExprFunction;
use crate::records::cst_node::CstNode;
use crate::records::function::Function;
use crate::records::lexeme::Type;
use crate::records::location::Location;
use crate::records::match_lexeme::MatchLexeme;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_function_body(
        &mut self,
        hasself: bool,
        match_function: &Lexeme,
        debugname: &AstName,
        local_name: Option<&Name>,
        attributes: &AstArray<*mut AstAttr>,
        is_const: bool,
    ) -> (*mut AstExprFunction, *mut AstLocal) {
        let mut start = match_function.location;
        if attributes.size > 0 {
            start = unsafe { (**attributes.data).base.location };
        }

        let cst_node = if self.options.store_cst_data {
            unsafe { (*self.allocator).alloc(CstExprFunction::new()) }
        } else {
            core::ptr::null_mut()
        };

        let (generics, generic_packs) = if !cst_node.is_null() {
            let mut local_comma_positions = TempVector::new(&mut self.scratch_position);
            let res = self.parse_generic_type_list(
                false,
                unsafe { Some(&mut (*cst_node).open_generics_position) },
                Some(&mut local_comma_positions),
                unsafe { Some(&mut (*cst_node).close_generics_position) },
            );
            unsafe {
                (*cst_node).generics_comma_positions =
                    self.copy_temp_vector_t(&local_comma_positions);
            }
            res
        } else {
            self.parse_generic_type_list(false, None, None, None)
        };

        let match_paren = MatchLexeme::new(self.lexer.current());
        self.expect_and_consume_char('(', "function");

        self.match_recovery_stop_on_token[')' as usize] += 1;

        let mut args = TempVector::new(&mut self.scratch_binding);

        let mut vararg = false;
        let mut vararg_location = Location::default();
        let mut vararg_annotation: *mut AstTypePack = core::ptr::null_mut();

        if self.lexer.current().r#type != Type(')' as i32) {
            if !cst_node.is_null() {
                let res = self.parse_binding_list(
                    &mut args,
                    true,
                    unsafe { &mut (*cst_node).args_comma_positions },
                    core::ptr::null_mut(),
                    unsafe { &mut (*cst_node).vararg_annotation_colon_position },
                    false,
                );
                vararg = res.0;
                vararg_location = res.1;
                vararg_annotation = res.2;
            } else {
                let res = self.parse_binding_list(
                    &mut args,
                    true,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    false,
                );
                vararg = res.0;
                vararg_location = res.1;
                vararg_annotation = res.2;
            }
        }

        let mut arg_location: Option<Location> = None;
        if match_paren.type_ == Type('(' as i32) && self.lexer.current().r#type == Type(')' as i32)
        {
            arg_location = Some(Location::new(
                match_paren.position,
                self.lexer.current().location.end,
            ));
        }

        self.expect_match_and_consume(')', &match_paren, true);

        self.match_recovery_stop_on_token[')' as usize] -= 1;

        let typelist = self.parse_optional_return_type(if !cst_node.is_null() {
            unsafe { Some(&mut (*cst_node).return_specifier_position) }
        } else {
            None
        });
        let mut fun_local: *mut AstLocal = core::ptr::null_mut();
        if let Some(local_name) = local_name {
            let binding = if luaur_common::FFlag::LuauConst2.get() {
                Binding::new(
                    *local_name,
                    core::ptr::null_mut(),
                    Position::new(0, 0),
                    is_const,
                )
            } else {
                Binding::new(
                    *local_name,
                    core::ptr::null_mut(),
                    Position::missing(),
                    false,
                )
            };
            fun_local = self.push_local(&binding);
        }

        let locals_begin = self.save_locals();

        let mut fun = Function::new();
        fun.vararg = vararg;

        self.function_stack.push(fun);

        let (self_, vars) = self.prepare_function_arguments(&start, hasself, &args);

        let body = self.parse_block();

        self.function_stack.pop();

        self.restore_locals(locals_begin);

        let end = self.lexer.current().location;

        let has_end =
            self.expect_match_end_and_consume(Type::ReservedEnd, &MatchLexeme::new(match_function));
        unsafe {
            (*body).has_end = has_end;
        }

        let node = unsafe {
            (*self.allocator).alloc(AstExprFunction::new(
                Location::new(start.begin, end.end),
                *attributes,
                generics,
                generic_packs,
                self_,
                vars,
                vararg,
                vararg_location,
                body,
                self.function_stack.len(),
                *debugname,
                typelist,
                vararg_annotation,
                arg_location,
            ))
        };

        if self.options.store_cst_data {
            unsafe {
                (*cst_node).function_keyword_position = match_function.location.begin;
                (*cst_node).args_annotation_colon_positions =
                    self.extract_annotation_colon_positions(&args);
            }
            self.cst_node_map
                .try_insert(node as *mut AstNode, cst_node as *mut CstNode);
        }

        (node, fun_local)
    }
}
