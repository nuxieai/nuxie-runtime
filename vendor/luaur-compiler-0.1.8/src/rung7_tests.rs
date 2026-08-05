use crate::enums::type_constant_folding::Type;
use crate::functions::cnum::cnum;
use crate::functions::compile_or_throw_compiler_alt_b::compile_or_throw_bytecode_builder_string_compile_options_parse_options;
use crate::functions::fold_binary::fold_binary;
use crate::functions::fold_builtin::fold_builtin;
use crate::records::compile_options::CompileOptions;
use crate::records::constant::{Constant, ConstantData};
use luaur_ast::records::allocator::Allocator;
use luaur_ast::records::ast_expr_binary::AstExprBinaryOp;
use luaur_ast::records::ast_name_table::AstNameTable;
use luaur_ast::records::parse_options::ParseOptions;
use luaur_bytecode::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction::LBF_VECTOR;

#[test]
fn vector_builtin_folding_respects_component_precision() {
    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let args = [cnum(1.0 / 3.0), cnum(2.0), cnum(3.0)];

    let vectorf = fold_builtin(
        &mut names,
        LBF_VECTOR as i32,
        args.as_ptr(),
        args.len(),
        false,
    );
    let vectord = fold_builtin(
        &mut names,
        LBF_VECTOR as i32,
        args.as_ptr(),
        args.len(),
        true,
    );

    assert_eq!(vectorf.r#type, Type::Type_Vectorf);
    assert_eq!(vectord.r#type, Type::Type_Vectord);
    assert_eq!(unsafe { vectorf.data.value_vectorf[0] }, (1.0 / 3.0) as f32);
    assert_eq!(unsafe { vectord.data.value_vectord[0] }, 1.0 / 3.0);
    assert_eq!(CompileOptions::default().vector_precision, 0);
}

#[test]
fn double_vector_arithmetic_preserves_double_components() {
    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let lhs = Constant {
        r#type: Type::Type_Vectord,
        string_length: 0,
        data: ConstantData {
            value_vectord: [1.0 / 3.0, 4.0, 9.0, 0.0],
        },
    };
    let rhs = Constant {
        r#type: Type::Type_Vectord,
        string_length: 0,
        data: ConstantData {
            value_vectord: [3.0, 2.0, 3.0, 0.0],
        },
    };
    let mut result = Constant::default();

    fold_binary(&mut result, AstExprBinaryOp::Mul, &lhs, &rhs, &mut names);

    assert_eq!(result.r#type, Type::Type_Vectord);
    assert_eq!(unsafe { result.data.value_vectord }, [1.0, 8.0, 27.0, 0.0]);
}

#[test]
fn class_declarations_are_hoisted_for_forward_references() {
    let old = luaur_common::FFlag::DebugLuauUserDefinedClasses.get();
    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(true);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let source =
            String::from("local ref = Point\nclass Point\n    public x\nend\nreturn ref, Point");
        let mut bytecode = BytecodeBuilder::new(None);
        compile_or_throw_bytecode_builder_string_compile_options_parse_options(
            &mut bytecode,
            &source,
            &CompileOptions::default(),
            &ParseOptions::default(),
        );
        bytecode.get_bytecode().clone()
    }));

    luaur_common::FFlag::DebugLuauUserDefinedClasses.set(old);
    let bytecode = result.expect("class hoisting compilation should succeed");
    assert!(!bytecode.starts_with('\0'));
}
