use crate::functions::sref_compiler::sref_ast_name;
use crate::records::compile_error::CompileError;
use crate::records::compiler::Compiler;
use luaur_bytecode::records::table_shape::TableShape;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn build_export_table_shape(&mut self) {
        LUAU_ASSERT!(luaur_common::FFlag::LuauOptimizeExportTable.get());

        let mut exported_shape = TableShape::default();
        if self.exports.exported_variables.len() >= TableShape::kMaxLength as usize {
            return;
        }

        let exported_variables = self.exports.exported_variables.clone();
        for exported_local in exported_variables {
            let Some(variable) = self.variables.find(&exported_local).copied() else {
                unsafe {
                    CompileError::raise(
                        &(*exported_local).location,
                        format_args!("Local does not have corresponding variable"),
                    );
                }
            };

            let key_cid = unsafe {
                (*self.bytecode).add_constant_string(sref_ast_name((*exported_local).name))
            };
            if key_cid < 0 {
                return;
            }

            let idx = exported_shape.length as usize;
            exported_shape.keys[idx] = key_cid;
            exported_shape.constants[idx] = -1;
            if variable.constant && !variable.written {
                let value_cid = self.get_constant_index(variable.init);
                if value_cid < 0 {
                    return;
                }
                exported_shape.constants[idx] = value_cid;
                exported_shape.hasConstants = true;
            }
            exported_shape.length += 1;
        }

        if exported_shape.length > 0 {
            self.exports.exported_table_cid =
                unsafe { (*self.bytecode).add_constant_table(&exported_shape) };
        }
    }
}
