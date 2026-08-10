#![cfg(feature = "luau")]

mod support;

use nuxie_scripting::gpu_canvas::GpuCanvasBytecodeProgram;
use support::compile_source;

fn program() -> GpuCanvasBytecodeProgram {
    let source = r#"
        return function(context)
            local _canvas = context:gpuCanvas()
            return {
                amount = 1.0,
                enabled = true,
                title = "original",
                advance = function(self, _seconds)
                    return type(self.amount) == "number"
                        and self.enabled
                        and self.amount > 5
                        and self.title == "updated"
                        and self.added == 7
                end,
            }
        end
    "#;
    GpuCanvasBytecodeProgram::load(&compile_source(source).expect("source compiles"))
        .expect("program loads")
}

#[test]
fn retained_program_applies_scalar_script_inputs() {
    let mut program = program();

    program.set_number_input("amount", 6.0).expect("number");
    program.set_boolean_input("enabled", true).expect("boolean");
    program
        .set_string_input("title", "updated")
        .expect("string");
    program
        .set_number_input("added", 7.0)
        .expect("new table field");

    assert!(
        program
            .advance(1.0 / 60.0)
            .expect("advance observes inputs")
    );
}

#[test]
fn retained_program_matches_cpp_untyped_table_assignment() {
    let mut program = program();

    program
        .set_boolean_input("amount", false)
        .expect("C++ permits replacing a field with another scalar type");
    assert!(!program.advance(1.0 / 60.0).expect("wrong type is visible"));

    program
        .set_number_input("amount", 6.0)
        .expect("numeric input restores the field");
    program
        .set_string_input("title", "updated")
        .expect("string input");
    program
        .set_number_input("added", 7.0)
        .expect("new table field");
    assert!(program.advance(1.0 / 60.0).expect("restored inputs"));
}

#[test]
fn retained_program_matches_cpp_c_string_boundaries() {
    let mut program = program();

    program
        .set_string_input("title\0ignored", "updated\0ignored")
        .expect("C++ truncates names and string values at embedded NULs");
    program.set_number_input("amount", 6.0).expect("number");
    program
        .set_number_input("added", 7.0)
        .expect("new table field");

    assert!(program.advance(1.0 / 60.0).expect("truncated string input"));
}
