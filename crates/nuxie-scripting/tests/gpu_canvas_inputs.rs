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
                color = 0,
                advance = function(self, _seconds)
                    return type(self.amount) == "number"
                        and self.enabled
                        and self.amount > 5
                        and self.title == "updated"
                        and self.color == 4279312947
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
        .set_integer_input("color", 0xff112233_u32 as i32)
        .expect("C++ integer input uses unsigned Lua number semantics");
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
        .set_integer_input("color", 0xff112233_u32 as i32)
        .expect("integer input");
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
        .set_integer_input("color", 0xff112233_u32 as i32)
        .expect("integer input");
    program
        .set_number_input("added", 7.0)
        .expect("new table field");

    assert!(program.advance(1.0 / 60.0).expect("truncated string input"));
}

#[test]
fn retained_program_executes_the_current_draw_and_composite_contract() {
    let source = r#"
        return function(context)
            local canvas = context:gpuCanvas()
            local shader = context:shader("scene")
            local pipeline = GPUPipeline.new {
                vertex = shader,
                fragment = shader,
                vertexLayout = {},
                colorTargets = { { format = "rgba8unorm" } },
            }
            local sampler = ImageSampler("clamp", "clamp", "nearest")
            canvas:resize(8, 6)
            return {
                draw = function(self, renderer)
                    local pass = canvas:beginRenderPass {
                        color = {
                            {
                                loadOp = "clear",
                                storeOp = "store",
                                clearColor = { 0.25, 0.5, 0.75, 1.0 },
                            },
                        },
                    }
                    pass:setPipeline(pipeline)
                    pass:draw(3)
                    pass:finish()
                    renderer:drawImage(canvas.image, sampler, "srcOver", 1.0)
                end,
            }
        end
    "#;
    let mut program = GpuCanvasBytecodeProgram::load(
        &compile_source(source).expect("current GPU-canvas source compiles"),
    )
    .expect("current GPU-canvas program loads");

    let plan = program
        .draw()
        .expect("direct snapshots execute draw and ignore only the final 2D composite");
    assert_eq!((plan.width, plan.height), (8, 6));
    assert_eq!(plan.vertex_count, 3);
    assert_eq!(plan.clear_color, [0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn retained_program_ignores_the_current_scripted_renderer_surface() {
    let source = r#"
        return function(context)
            local canvas = context:gpuCanvas()
            local shader = context:shader("scene")
            local pipeline = GPUPipeline.new {
                vertex = shader,
                fragment = shader,
                vertexLayout = {},
                colorTargets = { { format = "rgba8unorm" } },
            }
            canvas:resize(5, 7)
            return {
                draw = function(self, renderer)
                    renderer:save()
                    renderer:transform(nil)
                    renderer:clipPath(nil)
                    renderer:drawPath(nil, nil)
                    renderer:drawImage(nil, nil, "srcOver", 1.0)
                    renderer:drawImageMesh(nil, nil, nil, nil, nil, "srcOver", 1.0)

                    local pass = canvas:beginRenderPass {
                        color = {
                            {
                                loadOp = "clear",
                                storeOp = "store",
                                clearColor = { 0.1, 0.2, 0.3, 1.0 },
                            },
                        },
                    }
                    pass:setPipeline(pipeline)
                    pass:draw(6)
                    pass:finish()

                    renderer:restore()
                    renderer:save()
                    renderer:restore()
                end,
            }
        end
    "#;
    let mut program = GpuCanvasBytecodeProgram::load(
        &compile_source(source).expect("mixed renderer source compiles"),
    )
    .expect("mixed renderer program loads");

    let plan = program
        .draw()
        .expect("snapshot renderer ignores 2D calls and returns the GPU plan");
    assert_eq!((plan.width, plan.height), (5, 7));
    assert_eq!(plan.vertex_count, 6);
    assert_eq!(plan.clear_color, [0.1, 0.2, 0.3, 1.0]);
}

#[test]
fn retained_program_samples_supplied_images_and_reports_missing_names() {
    use nuxie_scripting::gpu_canvas::GpuCanvasSnapshotImage;

    let source = r#"
        return function(context)
            local canvas = context:gpuCanvas()
            local shader = context:shader("scene")
            local painting = context:image("asset_painting")
            local missing = context:image("asset_missing")
            local pipeline = GPUPipeline.new {
                vertex = shader,
                fragment = shader,
                vertexLayout = {},
                colorTargets = { { format = "rgba8unorm" } },
            }
            local layout = GPUBindGroupLayout.new { groupIndex = 0, shader = shader }
            local group = GPUBindGroup.new {
                layout = layout,
                textures = { { slot = 0, view = painting:view() } },
            }
            canvas:resize(painting.width, painting.height)
            return {
                missingIsNil = missing == nil,
                draw = function(self, renderer)
                    local pass = canvas:beginRenderPass {
                        color = { { loadOp = "clear", storeOp = "store" } },
                    }
                    pass:setPipeline(pipeline)
                    pass:setBindGroup(0, group)
                    pass:draw(3)
                    pass:finish()
                    renderer:drawImage(painting, nil, "srcOver", 1.0)
                end,
            }
        end
    "#;
    let rgba = vec![
        255, 0, 0, 255, 0, 255, 0, 255, //
        0, 0, 255, 255, 255, 255, 255, 255,
    ];
    let mut program = GpuCanvasBytecodeProgram::load_with_images(
        &compile_source(source).expect("image source compiles"),
        vec![GpuCanvasSnapshotImage {
            name: "asset_painting".into(),
            width: 2,
            height: 2,
            rgba: rgba.clone(),
        }],
    )
    .expect("image program loads");

    let plan = program.draw().expect("image program draws");
    assert_eq!((plan.width, plan.height), (2, 2));
    let bindings: Vec<_> = plan
        .pipelines
        .iter()
        .flat_map(|pipeline| &pipeline.texture_bindings)
        .chain(&plan.texture_bindings)
        .collect();
    let binding = bindings.first().expect("the image view is bound");
    assert_eq!((binding.width, binding.height), (2, 2));
    assert_eq!(binding.format, "rgba8unorm");
    assert_eq!(binding.uploads.len(), 1);
    assert_eq!(binding.uploads[0].bytes, rgba);
    assert_eq!(binding.uploads[0].bytes_per_row, 8);
    assert_eq!(program.missing_images(), vec!["asset_missing".to_string()]);
}

#[test]
fn retained_program_rejects_images_whose_pixels_do_not_match_their_size() {
    use nuxie_scripting::gpu_canvas::GpuCanvasSnapshotImage;

    let error = GpuCanvasBytecodeProgram::load_with_images(
        &compile_source("return function(context) context:gpuCanvas() return {} end")
            .expect("source compiles"),
        vec![GpuCanvasSnapshotImage {
            name: "short".into(),
            width: 2,
            height: 2,
            rgba: vec![0; 4],
        }],
    )
    .expect_err("short pixel buffers are rejected");
    assert!(error.to_string().contains("must be 2x2 RGBA8 pixels"));
}
