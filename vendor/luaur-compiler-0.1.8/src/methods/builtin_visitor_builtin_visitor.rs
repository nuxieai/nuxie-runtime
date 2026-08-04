use crate::enums::global::Global;
use crate::functions::get_builtin_function_id::get_builtin_function_id;
use crate::functions::get_global_state::get_global_state;
use crate::records::builtin::Builtin;
use crate::records::builtin_visitor::BuiltinVisitor;
use crate::records::compile_options::CompileOptions;
use crate::records::variable::Variable;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_common::records::dense_hash_map::DenseHashMap;

use core::ffi::CStr;
use core::ptr;

fn find_dot(ptr: *const core::ffi::c_char) -> *const core::ffi::c_char {
    unsafe {
        if ptr.is_null() {
            return ptr::null();
        }
        let mut p = ptr;
        loop {
            let c = *p;
            if c == 0 {
                return ptr::null();
            }
            if c == b'.' as core::ffi::c_char {
                return p;
            }
            p = p.add(1);
        }
    }
}

fn c_str_len(ptr: *const core::ffi::c_char) -> usize {
    unsafe {
        if ptr.is_null() {
            return 0;
        }
        let bytes = CStr::from_ptr(ptr).to_bytes();
        bytes.len()
    }
}

impl BuiltinVisitor {
    pub fn new(
        result: &mut DenseHashMap<*mut AstExprCall, i32>,
        globals: &DenseHashMap<AstName, Global>,
        variables: &DenseHashMap<*mut AstLocal, Variable>,
        options: &CompileOptions,
        names: &AstNameTable,
    ) -> Self {
        let mut builtin_is_disabled = [false; 256];

        let disabled_builtins = options.disabled_builtins;
        if !disabled_builtins.is_null() {
            unsafe {
                let mut ptr = disabled_builtins;
                while !(*ptr).is_null() {
                    let current = *ptr;
                    let dot = find_dot(current);

                    if !dot.is_null() {
                        let library_len = (dot as usize) - (current as usize);
                        let (library, _) = names.get_with_type(current, library_len);
                        let name_ptr = dot.add(1);
                        let name = names.get(name_ptr);

                        if !library.value.is_null()
                            && !name.value.is_null()
                            && get_global_state(globals, name) == Global::Default
                        {
                            let builtin = Builtin {
                                object: library,
                                method: name,
                            };

                            let bfid = get_builtin_function_id(&builtin, options);
                            if bfid >= 0 && (bfid as usize) < 256 {
                                builtin_is_disabled[bfid as usize] = true;
                            }
                        }
                    } else {
                        let name = names.get(current);
                        if !name.value.is_null()
                            && get_global_state(globals, name) == Global::Default
                        {
                            let builtin = Builtin {
                                object: AstName::new(),
                                method: name,
                            };

                            let bfid = get_builtin_function_id(&builtin, options);
                            if bfid >= 0 && (bfid as usize) < 256 {
                                builtin_is_disabled[bfid as usize] = true;
                            }
                        }
                    }
                    ptr = ptr.add(1);
                }
            }
        }

        let mut visitor = Self::builtin_visitor(result, globals, variables, options, names);
        visitor.builtin_is_disabled = builtin_is_disabled;
        visitor
    }
}

#[allow(non_snake_case)]
pub fn builtin_visitor_builtin_visitor(
    result: &mut DenseHashMap<*mut AstExprCall, i32>,
    globals: &DenseHashMap<AstName, Global>,
    variables: &DenseHashMap<*mut AstLocal, Variable>,
    options: &CompileOptions,
    names: &AstNameTable,
) -> BuiltinVisitor {
    BuiltinVisitor::new(result, globals, variables, options, names)
}
