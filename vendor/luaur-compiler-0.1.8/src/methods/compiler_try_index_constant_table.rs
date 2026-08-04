use crate::enums::table_constant_kind::TableConstantKind;
use crate::enums::type_constant_folding::Type;
use crate::functions::unwrap_expr_of_type::unwrap_expr_of_type;
use crate::records::compiler::Compiler;
use crate::records::constant::Constant;
use crate::records::variable::Variable;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::item_ast::ItemKind;

impl Compiler {
    pub fn try_index_constant_table(&mut self, expr: *mut AstExprIndexName) -> *mut AstExpr {
        unsafe {
            if expr.is_null() {
                return core::ptr::null_mut();
            }

            let table_expr = (*expr).expr;
            let table_local = unwrap_expr_of_type::<AstExprLocal>(table_expr);
            if table_local.is_null() {
                return core::ptr::null_mut();
            }

            let lv = self.variables.find(&(*table_local).local);
            if lv.is_none() {
                return core::ptr::null_mut();
            }
            let lv = *lv.unwrap();
            if lv.written || lv.init.is_null() {
                return core::ptr::null_mut();
            }

            let table_kind = self.table_constants.find(&(*table_local).local);
            if table_kind.is_none() {
                return core::ptr::null_mut();
            }
            let table_kind = *table_kind.unwrap();
            if table_kind != TableConstantKind::ConstantTable {
                return core::ptr::null_mut();
            }

            let table = unwrap_expr_of_type::<AstExprTable>(lv.init);
            if table.is_null() {
                return core::ptr::null_mut();
            }

            let mut match_value: *mut AstExpr = core::ptr::null_mut();

            for item in (*table).items.as_slice() {
                if item.kind == ItemKind::Record || item.kind == ItemKind::General {
                    let key_constant = self.constants.find(&item.key);
                    if key_constant.is_none() {
                        match_value = core::ptr::null_mut();
                    } else {
                        let key_constant = key_constant.unwrap();
                        if key_constant.r#type == Type::Type_String
                            && key_constant.string_length != 0
                        {
                            let key_name = unsafe {
                                let arr = key_constant.get_string();
                                (*self.names).get_or_add(arr.data, arr.size as usize)
                            };

                            if key_name.operator_eq_ast_name(&(*expr).index) {
                                match_value = item.value;
                            }
                        }
                    }
                }
            }

            match_value
        }
    }
}
