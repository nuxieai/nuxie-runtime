//! Generated skeleton item.
//! Node: `cxx:Record:Luau.Compiler:Compiler/src/Compiler.cpp:4833:const_upvalue_visitor`
//! Source: `Compiler/src/Compiler.cpp`
//! Graph edges:
//! - declared_by: source_file Compiler/src/Compiler.cpp
//! - source_includes:
//!   - includes -> source_file Compiler/include/Luau/Compiler.h
//!   - includes -> source_file Ast/include/Luau/Ast.h
//!   - includes -> source_file Bytecode/include/Luau/BytecodeBuilder.h
//!   - includes -> source_file Common/include/Luau/Common.h
//!   - includes -> source_file Ast/include/Luau/Parser.h
//!   - includes -> source_file Common/include/Luau/InsertionOrderedMap.h
//!   - includes -> source_file Common/include/Luau/StringUtils.h
//!   - includes -> source_file Common/include/Luau/TimeTrace.h
//!   - includes -> source_file CodeGen/src/IrTranslateBuiltins.h
//!   - includes -> source_file Compiler/src/ConstantFolding.h
//!   - includes -> source_file Compiler/src/CostModel.h
//!   - includes -> source_file Compiler/src/TableShape.h
//!   - includes -> source_file Analysis/include/Luau/AutocompleteTypes.h
//!   - includes -> source_file Analysis/include/Luau/AstUtils.h
//!   - includes -> source_file Compiler/src/ValueTracking.h
//! - incoming:
//!   - declares <- source_file Compiler/src/Compiler.cpp
//!   - type_ref <- method Compiler::gatherConstUpvals (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::ConstUpvalueVisitor::ConstUpvalueVisitor (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::ConstUpvalueVisitor::visit (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::ConstUpvalueVisitor::visit (Compiler/src/Compiler.cpp)
//! - outgoing:
//!   - type_ref -> method Compiler::ConstUpvalueVisitor::ConstUpvalueVisitor (Compiler/src/Compiler.cpp)
//!   - type_ref -> record AstVisitor (Ast/include/Luau/Ast.h)
//!   - type_ref -> record Compiler (Compiler/src/Compiler.cpp)
//!   - type_ref -> record AstLocal (Ast/include/Luau/Ast.h)
//!   - translates_to -> rust_item ConstUpvalueVisitor

use crate::records::compiler::Compiler;
use alloc::vec::Vec;
use luaur_ast::records::ast_local::AstLocal;
use luaur_ast::records::ast_visitor::AstVisitor;

#[derive(Debug, Clone)]
pub struct ConstUpvalueVisitor {
    pub(crate) self_: *mut Compiler,
    pub(crate) upvals: Vec<*mut AstLocal>,
}

impl AstVisitor for ConstUpvalueVisitor {
    fn visit_expr_local(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_local(node.cast())
    }

    fn visit_expr_function(&mut self, node: *mut core::ffi::c_void) -> bool {
        self.visit_ast_expr_function(node.cast())
    }
}
