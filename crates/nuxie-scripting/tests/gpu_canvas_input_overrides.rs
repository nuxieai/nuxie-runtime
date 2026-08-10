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
                    return self.enabled and self.amount > 5 and self.title == "updated"
                end,
            }
        end
    "#;
    GpuCanvasBytecodeProgram::load(&compile_source(source).expect("source compiles"))
        .expect("program loads")
}

#[test]
fn retained_program_accepts_typed_authored_input_overrides() {
    let mut program = program();

    assert!(program.set_number_input("amount", 6.0).expect("number"));
    assert!(!program.set_boolean_input("enabled", true).expect("boolean"));
    assert!(
        program
            .set_string_input("title", "updated")
            .expect("string")
    );
    assert!(
        program
            .advance(1.0 / 60.0)
            .expect("advance observes inputs")
    );
}

#[test]
fn retained_program_rejects_unknown_mismatched_and_non_finite_inputs() {
    let mut program = program();

    let unknown = program
        .set_number_input("missing", 1.0)
        .expect_err("unknown input");
    assert!(unknown.to_string().contains("is not defined"), "{unknown}");

    let mismatch = program
        .set_boolean_input("amount", false)
        .expect_err("type mismatch");
    assert!(
        mismatch.to_string().contains("expected boolean"),
        "{mismatch}"
    );

    let non_finite = program
        .set_number_input("amount", f64::NAN)
        .expect_err("non-finite number");
    assert!(
        non_finite.to_string().contains("finite number"),
        "{non_finite}"
    );
}
