//! Generated skeleton item.
//! Node: `cxx:Record:Luau.Compiler:Compiler/src/Compiler.cpp:4883:reg_scope`
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
//!   - type_ref <- record Compiler (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileFunction (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprTempMultRet (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprTempTop (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileInlinedCall (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprCall (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprFunction (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileClassDeclaration (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileCompareJump (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileConditionValue (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprAndOr (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprUnary (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprBinary (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprIfElseAndOr (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprInterpString (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprTable (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprIndexName (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprIndexExpr (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprAuto (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileExprSide (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileLValueIndex (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileLValue (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatRepeat (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatReturn (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatFor (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatForIn (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatAssign (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatCompoundAssign (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStatFunction (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::compileStat (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::RegScope::RegScope (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::RegScope::RegScope (Compiler/src/Compiler.cpp)
//!   - type_ref <- method Compiler::RegScope::~RegScope (Compiler/src/Compiler.cpp)
//! - outgoing:
//!   - type_ref -> record Compiler (Compiler/src/Compiler.cpp)
//!   - translates_to -> rust_item RegScope

use crate::records::compiler::Compiler;

// RAII register-stack guard. Must NOT be `Copy`/`Clone`: the C++ `~RegScope`
// restores `regTop = oldTop` on scope exit, reproduced by the `Drop` below. The
// prior `Copy` derive silently elided that restore, leaving `regTop` un-rewound
// between sibling sub-expressions and tripping `assert!(top <= regTop)` in
// `reg_scope_compiler_i32`.
#[derive(Debug)]
pub struct RegScope {
    pub(crate) self_: *mut Compiler,
    pub(crate) old_top: core::ffi::c_uint,
}

impl Drop for RegScope {
    fn drop(&mut self) {
        // C++ `~RegScope() { self->regTop = oldTop; }`
        unsafe {
            (*self.self_).reg_top = self.old_top;
        }
    }
}
