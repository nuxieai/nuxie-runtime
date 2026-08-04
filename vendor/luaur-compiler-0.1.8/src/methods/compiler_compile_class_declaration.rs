use crate::records::compiler::Compiler;
use alloc::vec::Vec;
use luaur_ast::records::ast_stat_class::AstStatClass;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::variant::Variant2;

impl Compiler {
    pub fn compile_class_declaration(&mut self, decl: *mut AstStatClass) {
        let dest = self.alloc_reg(decl as *mut _, 1);
        self.push_local(unsafe { (*decl).name }, dest, !0u32);
        // C++ `RegScope _(this)` after pushLocal: reclaims the transient `temp` register on
        // scope exit so it doesn't leak past the declaration (the port had dropped this).
        let _rs = self.reg_scope_compiler();
        unsafe {
            (*self.bytecode).emit_ad(LuauOpcode::LOP_LOADKX, dest, 0);
            let aux_offset = (*self.bytecode).emit_label();
            (*self.bytecode).emit_aux(0xDEADBEEF);
            // C++ builds `shape.className`/`propertyNames`/`methodNames` directly; the port
            // collected names into throwaway vecs and stored an empty default shape, so every
            // class constant dumped `(props: 0, methods: 0)`.
            let mut shape = luaur_bytecode::records::class_shape::ClassShape::default();
            let class_name = (*(*decl).name).name;
            let class_name_cid = (*self.bytecode)
                .add_constant_string(crate::functions::sref_compiler::sref_ast_name(class_name));
            self.check_constant(class_name_cid, &(*(*decl).name).location);
            shape.className = class_name_cid;
            let temp = self.alloc_reg(decl as *mut _, 1);
            for member in (*decl).members.as_slice() {
                match member {
                    Variant2::V0(prop) => {
                        let cid = (*self.bytecode).add_constant_string(
                            crate::functions::sref_compiler::sref_ast_name(prop.name),
                        );
                        self.check_constant(cid, &prop.name_location);
                        shape.propertyNames.push(cid);
                    }
                    Variant2::V1(method) => {
                        self.compile_expr_function(method.function, temp);
                        let cid = (*self.bytecode).add_constant_string(
                            crate::functions::sref_compiler::sref_ast_name(method.function_name),
                        );
                        self.check_constant(cid, &(*method.function).base.base.location);
                        shape.methodNames.push(cid);
                        (*self.bytecode).emit_abc(LuauOpcode::LOP_NEWCLASSMEMBER, dest, 0, temp);
                        (*self.bytecode).emit_aux(cid as u32);
                    }
                }
            }
            let class_const = (*self.bytecode).add_class_shape(shape);
            self.check_constant(class_const, &(*decl).base.base.location);
            (*self.bytecode).patch_aux(aux_offset, class_const);
            if luaur_common::FFlag::LuauExportValueSyntax.get() && (*decl).exported {
                self.exported_classes.push((class_name, dest));
            }
        }
    }
}
