#![cfg(feature = "luau")]

use nuxie_scripting::vm::ScriptVm;

fn rive_vm() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}

#[test]
fn path_exposes_raw_verbs_as_one_based_commands() {
    let vm = rive_vm();

    let values: (
        i64,
        String,
        i64,
        String,
        i64,
        String,
        i64,
        String,
        i64,
        String,
        i64,
    ) = vm
        .eval(
            r#"
            local path = Path.new()
            path:moveTo(Vector(1, 2))
            path:lineTo(Vector(3, 4))
            path:quadTo(Vector(5, 6), Vector(7, 8))
            path:cubicTo(Vector(9, 10), Vector(11, 12), Vector(13, 14))
            path:close()
            return #path,
                path[1].type, #path[1],
                path[2].type, #path[2],
                path[3].type, #path[3],
                path[4].type, #path[4],
                path[5].type, #path[5]
            "#,
        )
        .unwrap();

    assert_eq!(
        values,
        (
            5,
            "moveTo".to_owned(),
            1,
            "lineTo".to_owned(),
            1,
            "quadTo".to_owned(),
            2,
            "cubicTo".to_owned(),
            3,
            "close".to_owned(),
            0,
        )
    );
}

#[test]
fn path_commands_preserve_raw_quad_and_cubic_point_order() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local path = Path.new()
            path:moveTo(Vector(1, 2))
            path:quadTo(Vector(3, 4), Vector(5, 6))
            path:cubicTo(Vector(7, 8), Vector(9, 10), Vector(11, 12))
            local quad = path[2]
            local cubic = path[3]
            return quad[1].x, quad[1].y, quad[2].x, quad[2].y,
                cubic[1].x, cubic[1].y, cubic[2].x, cubic[2].y,
                cubic[3].x, cubic[3].y
            "#,
        )
        .unwrap();

    assert_eq!(
        values,
        (3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0)
    );
}

#[test]
fn path_indexing_matches_cpp_out_of_range_behavior_and_keeps_methods() {
    let vm = rive_vm();

    let values: (String, i64, bool, bool, bool, i64) = vm
        .eval(
            r#"
            local path = Path.new()
            path:moveTo(Vector(1, 2))
            local absent = path[100]
            local command = path[1]
            local unknownPropertyOk = pcall(function()
                return command.points
            end)
            path:reset()
            return absent.type, #absent, command[0] == nil,
                command[2] == nil, unknownPropertyOk, #path
            "#,
        )
        .unwrap();

    assert_eq!(values, ("none".to_owned(), 0, true, true, false, 0));
}
