use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use crate::records::constant_visitor::ConstantVisitor;
use luaur_ast::records::ast_local::AstLocal;
use luaur_common::FFlag;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> ConstantVisitor<'a> {
    pub fn record_value(&mut self, local: *mut AstLocal, value: &Constant) {
        unsafe {
            // note: we rely on trackValues to have been run before us
            let v = self.variables.find_mut(&local);
            LUAU_ASSERT!(v.is_some());
            let v = v.unwrap();

            if !v.written {
                if FFlag::LuauCompileFoldOptimize.get() && FFlag::LuauCompilePropagateTableProps2.get() {
                    if value.r#type == Type::Type_Table {
                        v.constant = false;
                        *self.table_locals.get_or_insert(local) = value.clone();
                    } else {
                        v.constant = value.r#type != Type::Type_Unknown;
                        let locals_ptr = self.locals as *mut _;
                        self.record_constant(&mut *locals_ptr, local, value);
                    }
                } else {
                    v.constant = if FFlag::LuauCompilePropagateTableProps2.get() {
                        value.r#type != Type::Type_Unknown && value.r#type != Type::Type_Table
                    } else {
                        value.r#type != Type::Type_Unknown
                    };
                    let locals_ptr = self.locals as *mut _;
                    self.record_constant(&mut *locals_ptr, local, value);
                }
            }
        }
    }
}
