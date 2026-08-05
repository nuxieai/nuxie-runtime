use crate::functions::get_builtin_function_id::get_builtin_function_id;
use crate::functions::get_builtin_info::get_builtin_info;
use crate::records::builtin::Builtin;
use crate::records::builtin_info::BuiltinInfo;
use crate::records::compile_options::CompileOptions;
use luaur_ast::records::ast_name::AstName;
use luaur_common::enums::luau_builtin_function::LuauBuiltinFunction::*;

fn builtin(object: &'static core::ffi::CStr, method: &'static core::ffi::CStr) -> Builtin {
    Builtin {
        object: AstName {
            value: object.as_ptr(),
        },
        method: AstName {
            value: method.as_ptr(),
        },
    }
}

#[test]
fn rive_builtin_names_have_pinned_ids() {
    let options = CompileOptions::default();
    let cases = [
        (c"math", c"fround", LBF_RIVE_FROUND as i32),
        (c"Vector", c"xy", LBF_VECTOR as i32),
        (c"Vector", c"distance", LBF_RIVE_VECTOR_DISTANCE as i32),
        (
            c"Vector",
            c"distanceSquared",
            LBF_RIVE_VECTOR_DISTANCE_SQUARED as i32,
        ),
        (c"Vector", c"origin", LBF_RIVE_VECTOR_ORIGIN as i32),
        (
            c"Vector",
            c"lengthSquared",
            LBF_RIVE_VECTOR_LENGTH_SQUARED as i32,
        ),
        (c"Vector", c"dot", LBF_RIVE_VECTOR_DOT as i32),
        (c"Vector", c"length", LBF_RIVE_VECTOR_MAGNITUDE as i32),
        (c"Vector", c"normalized", LBF_RIVE_VECTOR_NORMALIZE as i32),
        (c"Vector", c"lerp", LBF_RIVE_VECTOR_LERP as i32),
        (c"Vector", c"cross", LBF_RIVE_VECTOR2_CROSS as i32),
        (
            c"Vector",
            c"scaleAndAdd",
            LBF_RIVE_VECTOR_SCALE_AND_ADD as i32,
        ),
        (
            c"Vector",
            c"scaleAndSub",
            LBF_RIVE_VECTOR_SCALE_AND_SUB as i32,
        ),
    ];

    for (object, method, expected) in cases {
        assert_eq!(
            get_builtin_function_id(&builtin(object, method), &options),
            expected,
            "{}.{}",
            object.to_string_lossy(),
            method.to_string_lossy()
        );
    }

    assert_eq!(
        get_builtin_function_id(&builtin(c"vector", c"origin"), &options),
        -1
    );
    assert_eq!(
        get_builtin_function_id(&builtin(c"Vector", c"Distance"), &options),
        -1
    );
}

#[test]
fn rive_builtin_layout_and_arities_are_exact() {
    assert_eq!(LBF_RIVE_FROUND as u8, 243);
    assert_eq!(LBF_RIVE_VECTOR_DISTANCE as u8, 245);
    assert_eq!(LBF_RIVE_VECTOR_SCALE_AND_SUB as u8, 255);

    let cases = [
        (LBF_RIVE_FROUND, 1),
        (LBF_RIVE_VECTOR_DISTANCE, 2),
        (LBF_RIVE_VECTOR_DISTANCE_SQUARED, 2),
        (LBF_RIVE_VECTOR_ORIGIN, 0),
        (LBF_RIVE_VECTOR_LENGTH_SQUARED, 1),
        (LBF_RIVE_VECTOR_DOT, 2),
        (LBF_RIVE_VECTOR_MAGNITUDE, 1),
        (LBF_RIVE_VECTOR_NORMALIZE, 1),
        (LBF_RIVE_VECTOR_LERP, 3),
        (LBF_RIVE_VECTOR2_CROSS, 2),
        (LBF_RIVE_VECTOR_SCALE_AND_ADD, 3),
        (LBF_RIVE_VECTOR_SCALE_AND_SUB, 3),
    ];

    for (id, params) in cases {
        let info = get_builtin_info(id as i32);
        assert_eq!(info.params, params);
        assert_eq!(info.results, 1);
        assert_eq!(info.flags, BuiltinInfo::Flag_NoneSafe);
    }
}
