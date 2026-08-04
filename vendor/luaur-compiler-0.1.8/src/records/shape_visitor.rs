//! Source: `Compiler/src/TableShape.cpp:27-149`

use crate::records::hasher::Hasher;
use crate::records::table_shape::TableShape;
use luaur_ast::records::ast_expr::AstExpr;
use luaur_ast::records::ast_expr_constant_number::AstExprConstantNumber;
use luaur_ast::records::ast_expr_index_expr::AstExprIndexExpr;
use luaur_ast::records::ast_expr_index_name::AstExprIndexName;
use luaur_ast::records::ast_expr_local::AstExprLocal;
use luaur_ast::records::ast_expr_table::AstExprTable;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_name::AstName;
use luaur_ast::records::ast_stat_assign::AstStatAssign;
use luaur_ast::records::ast_stat_for::AstStatFor;
use luaur_ast::records::ast_stat_function::AstStatFunction;
use luaur_ast::records::ast_stat_local::AstStatLocal;
use luaur_ast::records::ast_visitor::AstVisitor;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct ShapeVisitor<'a> {
    pub(crate) shapes: &'a mut DenseHashMap<*mut AstExprTable, TableShape>,
    pub(crate) tables: DenseHashMap<*mut AstLocal, *mut AstExprTable>,
    pub(crate) fields: DenseHashSet<(*mut AstExprTable, AstName), Hasher>,
    pub(crate) loops: DenseHashMap<*mut AstLocal, core::ffi::c_uint>,
}

impl<'a> ShapeVisitor<'a> {
    pub fn new(shapes: &'a mut DenseHashMap<*mut AstExprTable, TableShape>) -> Self {
        ShapeVisitor {
            shapes,
            tables: DenseHashMap::new(core::ptr::null_mut()),
            fields: DenseHashSet::new((core::ptr::null_mut(), AstName::new())),
            loops: DenseHashMap::new(core::ptr::null_mut()),
        }
    }

    fn assign_field_name(&mut self, expr: *mut AstExpr, index: AstName) {
        let lv = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprLocal>(
                expr as *mut luaur_ast::records::ast_node::AstNode,
            )
        };
        if lv.is_null() {
            return;
        }

        let table_opt = self.tables.find(&unsafe { (*lv).local });
        if let Some(&table) = table_opt {
            let field = (table, index);

            if !self.fields.contains(&field) {
                self.fields.insert(field);
                // C++ `shapes[*table].hashSize += 1` — operator[] inserts a default
                // shape on miss. `find_mut` does NOT insert, so the FIRST field of
                // every table found no shape and never counted -> predictions 0.
                self.shapes.get_or_insert(table).hash_size += 1;
            }
        }
    }

    fn assign_field_expr(&mut self, expr: *mut AstExpr, index: *mut AstExpr) {
        let lv = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprLocal>(
                expr as *mut luaur_ast::records::ast_node::AstNode,
            )
        };
        if lv.is_null() {
            return;
        }

        let table_opt = self.tables.find(&unsafe { (*lv).local });
        let table = match table_opt {
            Some(t) => *t,
            None => return,
        };

        let number = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprConstantNumber>(
                index as *mut luaur_ast::records::ast_node::AstNode,
            )
        };
        if !number.is_null() {
            // C++ `shapes[*table]` inserts-on-miss; `find_mut` did not, so array
            // predictions never started.
            let shape = self.shapes.get_or_insert(table);
            if unsafe { (*number).value } == (shape.array_size as f64 + 1.0) {
                shape.array_size += 1;
            }
        } else {
            let iter = unsafe {
                luaur_ast::rtti::ast_node_as::<AstExprLocal>(
                    index as *mut luaur_ast::records::ast_node::AstNode,
                )
            };
            if !iter.is_null() {
                if let Some(&bound) = self.loops.find(&unsafe { (*iter).local }) {
                    let shape = self.shapes.get_or_insert(table);
                    if shape.array_size == 0 {
                        shape.array_size = bound;
                    }
                }
            }
        }
    }

    fn assign(&mut self, var: *mut AstExpr) {
        let index_name = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprIndexName>(
                var as *mut luaur_ast::records::ast_node::AstNode,
            )
        };
        if !index_name.is_null() {
            self.assign_field_name(unsafe { (*index_name).expr }, unsafe {
                (*index_name).index
            });
            return;
        }

        let index_expr = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprIndexExpr>(
                var as *mut luaur_ast::records::ast_node::AstNode,
            )
        };
        if !index_expr.is_null() {
            self.assign_field_expr(unsafe { (*index_expr).expr }, unsafe {
                (*index_expr).index
            });
        }
    }
}

impl<'a> AstVisitor for ShapeVisitor<'a> {
    fn visit_stat_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = unsafe { &mut *(node as *mut AstStatLocal) };

        if node.vars.size == 1 && node.values.size == 1 {
            let value = unsafe { *node.values.data.add(0) };
            // C++ uses getTableHint, which unwraps `setmetatable(table_literal, ...)` to the
            // inner table literal. Casting the initializer straight to AstExprTable missed
            // that form, so a table behind setmetatable was never tracked and its predicted
            // shape stayed (0,0) -> NEWTABLE with size 0.
            let table = crate::functions::get_table_hint::get_table_hint(value);
            if !table.is_null() && unsafe { (*table).items.size } == 0 {
                let var = unsafe { *node.vars.data.add(0) };
                self.tables.try_insert(var, table);
            }
        }

        true
    }

    fn visit_stat_assign(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = unsafe { &mut *(node as *mut AstStatAssign) };

        for i in 0..node.vars.size as usize {
            let var = unsafe { *node.vars.data.add(i) };
            self.assign(var);
        }

        for i in 0..node.values.size as usize {
            let value = unsafe { *node.values.data.add(i) };
            unsafe { luaur_ast::visit::ast_expr_visit(value, self as &mut dyn AstVisitor) };
        }

        false
    }

    fn visit_stat_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = unsafe { &mut *(node as *mut AstStatFunction) };

        self.assign(node.name);

        unsafe {
            luaur_ast::visit::ast_expr_visit(
                node.func as *mut luaur_ast::records::ast_expr::AstExpr,
                self as &mut dyn AstVisitor,
            )
        };

        false
    }

    fn visit_stat_for(&mut self, node: *mut core::ffi::c_void) -> bool {
        let node = unsafe { &mut *(node as *mut AstStatFor) };

        let from = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprConstantNumber>(
                node.from as *mut luaur_ast::records::ast_node::AstNode,
            )
        };
        let to = unsafe {
            luaur_ast::rtti::ast_node_as::<AstExprConstantNumber>(
                node.to as *mut luaur_ast::records::ast_node::AstNode,
            )
        };

        if !from.is_null() && !to.is_null() {
            let from_val = unsafe { (*from).value };
            let to_val = unsafe { (*to).value };

            if from_val == 1.0 && to_val >= 1.0 && to_val <= 16.0 && node.step.is_null() {
                self.loops.try_insert(node.var, to_val as core::ffi::c_uint);
            }
        }

        true
    }
}
