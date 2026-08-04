use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_call::AstExprCall;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction;
use luaur_common::records::dense_hash_map::DenseHashMap;

use crate::enums::global::Global;
use crate::functions::get_builtin::get_builtin;
use crate::functions::get_builtin_function_id::get_builtin_function_id;
use crate::functions::get_global_state::get_global_state;
use crate::records::builtin::Builtin;
use crate::records::compile_options::CompileOptions;
use crate::records::variable::Variable;

use core::ffi::c_char;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct BuiltinVisitor {
    pub(crate) result: *mut DenseHashMap<*mut AstExprCall, i32>,
    pub(crate) builtin_is_disabled: [bool; 256],

    pub(crate) globals: *const DenseHashMap<AstName, Global>,
    pub(crate) variables: *const DenseHashMap<*mut AstLocal, Variable>,

    pub(crate) options: *const CompileOptions,
    pub(crate) names: *const AstNameTable,
}

impl BuiltinVisitor {
    pub fn builtin_visitor(
        result: &mut DenseHashMap<*mut AstExprCall, i32>,
        globals: &DenseHashMap<AstName, Global>,
        variables: &DenseHashMap<*mut AstLocal, Variable>,
        options: &CompileOptions,
        names: &AstNameTable,
    ) -> Self {
        let mut builtin_is_disabled = [false; 256];

        unsafe {
            let mut ptr = options.disabled_builtins;
            if !ptr.is_null() {
                while *ptr != core::ptr::null() {
                    let s = *ptr;

                    if let Some(dot) = memchr::memchr(
                        b'.',
                        core::slice::from_raw_parts(s as *const u8, c_str_len(s)),
                    ) {
                        let lib_ptr = s;
                        let lib_len = dot;

                        let (library, _library_type) =
                            names.get_with_type(lib_ptr, lib_len as usize);
                        let name = names.get(dot_ptr(dot, s));

                        if !library.value.is_null() && !name.value.is_null() {
                            if get_global_state(globals, name) == Global::Default {
                                let builtin = Builtin {
                                    object: library,
                                    method: name,
                                };
                                let bfid = get_builtin_function_id(&builtin, options);
                                if bfid >= 0 {
                                    builtin_is_disabled[bfid as usize] = true;
                                }
                            }
                        }
                    } else {
                        let name = names.get(s);
                        if !name.value.is_null()
                            && get_global_state(globals, name) == Global::Default
                        {
                            let builtin = Builtin {
                                object: AstName::new(),
                                method: name,
                            };
                            let bfid = get_builtin_function_id(&builtin, options);
                            if bfid >= 0 {
                                builtin_is_disabled[bfid as usize] = true;
                            }
                        }
                    }

                    ptr = ptr.add(1);
                }
            }
        }

        Self {
            result: result as *mut _,
            builtin_is_disabled,
            globals: globals as *const _,
            variables: variables as *const _,
            options: options as *const _,
            names: names as *const _,
        }
    }

    pub fn visit(&mut self, node: *mut AstExprCall) -> bool {
        unsafe {
            let options = &*self.options;
            let globals = &*self.globals;
            let variables = &*self.variables;
            let result = &mut *self.result;

            // C++ `getBuiltin(node->func, ...)` — pass the call's FUNCTION (the
            // `math.max` reference), not the whole call expression. The model
            // passed `node` (the call), so get_builtin never resolved a builtin
            // and nothing was registered for folding.
            let builtin = if (*node).self_ {
                Builtin::default()
            } else {
                get_builtin((*node).func, globals, variables)
            };

            if builtin.empty() {
                return true;
            }

            let mut bfid = get_builtin_function_id(&builtin, options);

            if bfid >= 0 && self.builtin_is_disabled[bfid as usize] {
                bfid = -1;
            }

            // getBuiltinFunctionId optimistically assumes all select() calls are builtin but actually
            // the second argument must be a vararg
            // C++: bfid == LBF_SELECT_VARARG && !(args.size == 2 && args.data[1]->is<AstExprVarargs>())
            if bfid == LuauBuiltinFunction::LBF_SELECT_VARARG as i32 {
                let is_select_arity_2 = (*node).args.len() == 2;
                let second_arg_is_vararg = is_select_arity_2 && {
                    let arg1 = *(*node).args.data.add(1);
                    luaur_ast::rtti::ast_node_is::<
                        luaur_ast::records::ast_expr_varargs::AstExprVarargs,
                    >(arg1 as *mut luaur_ast::records::ast_node::AstNode)
                };

                if !(is_select_arity_2 && second_arg_is_vararg) {
                    bfid = -1;
                }
            }

            if bfid >= 0 {
                // C++ `result[node] = bfid` overwrites.
                *result.get_or_insert(node) = bfid;
            }

            true
        }
    }
}

unsafe fn dot_ptr(dot_index: usize, base: *const c_char) -> *const c_char {
    (base as *const u8).add(dot_index) as *const c_char
}

unsafe fn c_str_len(mut s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s != 0 {
        len += 1;
        s = s.add(1);
    }
    len
}

mod memchr {
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        for (i, b) in haystack.iter().enumerate() {
            if *b == needle {
                return Some(i);
            }
        }
        None
    }
}

// C++ `BuiltinVisitor : AstVisitor` overrides `visit(AstExprCall*)`. The
// generated `visit(&mut self, *mut AstExprCall) -> bool` method existed but was
// NEVER wired to the visitor trait — so `analyzeBuiltins` (which must traverse
// the whole AST) registered nothing and ALL optimization-level-2 builtin
// constant-folding silently no-op'd. This impl dispatches every call node to it.
impl luaur_ast::records::ast_visitor::AstVisitor for BuiltinVisitor {
    fn visit_expr_call(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit(node as *mut AstExprCall)
    }
}
