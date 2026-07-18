//! Editor-facing GPU-canvas execution contract.
//!
//! These tests deliberately exercise Luau source through the same pure-Rust
//! VM crate used by runtime ScriptAssets. The returned draw plan is the typed
//! handoff to the browser renderer; JavaScript never interprets script state.
#![cfg(feature = "luau")]

use nuxie_scripting::gpu_canvas::{
    GpuCanvasProgram, MAX_CPU_BUFFER_BYTES, MAX_GPU_CANVAS_DIMENSION,
    MAX_GPU_CANVAS_DRAW_INVOCATIONS, MAX_UNIFORM_BUFFER_BYTES,
};

const ANIMATED_SCRIPT: &str = r#"
return function(context)
    local canvas = context:gpuCanvas()
    local elapsed = 0
    local scratch = buffer.create(16)
    local pipeline = nil
    local bindGroup = nil
    local ubo = nil

    local function prepare()
        if pipeline then return end
        canvas:resize(96, 64)
        local shader = context:shader("scene")
        pipeline = GPUPipeline.new {
            vertex = shader,
            fragment = shader,
            vertexLayout = {},
            colorTargets = { { format = "rgba8unorm" } },
        }
        buffer.writef32(scratch, 0, elapsed)
        ubo = GPUBuffer.new { size = 16, usage = "uniform", data = scratch }
        local layout = GPUBindGroupLayout.new { groupIndex = 0, shader = shader }
        bindGroup = GPUBindGroup.new {
            layout = layout,
            ubos = { { slot = 0, buffer = ubo } },
        }
    end

    return {
        advance = function(self, seconds)
            elapsed += seconds
            return true
        end,
        drawCanvas = function(self)
            prepare()
            buffer.writef32(scratch, 0, elapsed)
            ubo:write(scratch, 0)
            local pass = canvas:beginRenderPass {
                color = { {
                    loadOp = "clear",
                    storeOp = "store",
                    clearColor = { 0.1, 0.2, 0.3, 1.0 },
                } },
            }
            pass:setPipeline(pipeline)
            pass:setBindGroup(0, bindGroup)
            pass:draw(3)
            pass:finish()
        end,
    }
end
"#;

#[test]
fn executes_luau_and_returns_renderer_owned_draw_plan() {
    let mut program = GpuCanvasProgram::compile(ANIMATED_SCRIPT).expect("script compiles");

    program.advance(0.5).expect("advance succeeds");
    let first = program.draw().expect("draw plan exists");
    assert_eq!((first.width, first.height), (96, 64));
    assert_eq!(first.vertex_count, 3);
    assert_eq!(first.clear_color, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(first.uniform_buffers.len(), 1);
    assert_eq!(
        f32::from_le_bytes(first.uniform_buffers[0].bytes[0..4].try_into().unwrap()),
        0.5
    );

    program.advance(1.0).expect("second advance succeeds");
    let second = program.draw().expect("second draw plan exists");
    assert_eq!(
        f32::from_le_bytes(second.uniform_buffers[0].bytes[0..4].try_into().unwrap()),
        1.5
    );
}

#[test]
fn preserves_vertex_layout_and_buffer_bytes_for_mesh_draws() {
    let script = r#"
return function(context)
    local canvas = context:gpuCanvas()
    return {
        drawCanvas = function(self)
            canvas:resize(32, 32)
            local shader = context:shader("scene")
            local pipeline = GPUPipeline.new {
                vertex = shader,
                fragment = shader,
                vertexLayout = { {
                    stride = 12,
                    attributes = { { format = "float32x3", slot = 0, offset = 0 } },
                } },
                colorTargets = { { format = "rgba8unorm" } },
            }
            local data = buffer.create(36)
            buffer.writef32(data, 0, -1)
            buffer.writef32(data, 16, 1)
            local vertices = GPUBuffer.new { size = 36, usage = "vertex", data = data }
            local pass = canvas:beginRenderPass { color = { {
                loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 }
            } } }
            pass:setPipeline(pipeline)
            pass:setVertexBuffer(0, vertices)
            pass:draw(3)
            pass:finish()
        end,
    }
end
"#;

    let mut program = GpuCanvasProgram::compile(script).expect("mesh script compiles");
    let draw = program.draw().expect("mesh draw plan exists");
    assert_eq!(draw.vertex_layouts[0].stride, 12);
    assert_eq!(draw.vertex_layouts[0].attributes[0].shader_location, 0);
    assert_eq!(draw.vertex_layouts[0].attributes[0].format, "float32x3");
    assert_eq!(draw.vertex_buffers[0].slot, 0);
    assert_eq!(draw.vertex_buffers[0].bytes.len(), 36);
}

#[test]
fn syntax_and_unsupported_gpu_contracts_fail_closed() {
    let syntax = GpuCanvasProgram::compile("return function( this is not luau").unwrap_err();
    assert!(syntax.to_string().contains("syntax"), "{syntax}");

    let mut missing_draw = GpuCanvasProgram::compile("return function() return {} end")
        .expect("shape compiles before execution");
    let error = missing_draw.draw().unwrap_err();
    assert!(error.to_string().contains("drawCanvas"), "{error}");

    let mut infinite_draw = GpuCanvasProgram::compile(
        "return function() return { drawCanvas = function() while true do end end } end",
    )
    .expect("shape compiles before execution");
    let error = infinite_draw.draw().unwrap_err();
    assert!(error.to_string().contains("safepoints"), "{error}");

    let unsupported_pipeline =
        minimal_draw_script("-- draw is unreachable because the unsupported pipeline field fails")
            .replace(
                "colorTargets = { { format = \"rgba8unorm\" } },",
                "colorTargets = { { format = \"rgba8unorm\" } }, depthTest = true,",
            );
    let mut program =
        GpuCanvasProgram::compile(&unsupported_pipeline).expect("shape compiles before draw");
    let error = program.draw().unwrap_err();
    assert!(error.to_string().contains("unsupported"), "{error}");
}

#[test]
fn rejects_script_owned_allocations_and_canvas_dimensions_above_product_limits() {
    let oversized_cpu = format!(
        "return function() local data = buffer.create({}) return {{}} end",
        MAX_CPU_BUFFER_BYTES + 1
    );
    let error = GpuCanvasProgram::compile(&oversized_cpu).unwrap_err();
    assert!(error.to_string().contains("buffer.create size"), "{error}");

    let oversized_uniform = format!(
        r#"
return function()
    local data = buffer.create({size})
    GPUBuffer.new {{ size = {size}, usage = "uniform", data = data }}
    return {{}}
end
"#,
        size = MAX_UNIFORM_BUFFER_BYTES + 4
    );
    let error = GpuCanvasProgram::compile(&oversized_uniform).unwrap_err();
    assert!(error.to_string().contains("Uniform size"), "{error}");

    let oversized_canvas = format!(
        r#"
return function(context)
    local canvas = context:gpuCanvas()
    return {{ drawCanvas = function() canvas:resize({}, 1) end }}
end
"#,
        MAX_GPU_CANVAS_DIMENSION + 1
    );
    let mut program = GpuCanvasProgram::compile(&oversized_canvas).expect("shape compiles");
    let error = program.draw().unwrap_err();
    assert!(error.to_string().contains("dimensions"), "{error}");
}

#[test]
fn rejects_excessive_draw_counts_and_duplicate_pass_slots() {
    let excessive_draw = minimal_draw_script(&format!(
        "pass:draw({}, 2)",
        MAX_GPU_CANVAS_DRAW_INVOCATIONS
    ));
    let mut program = GpuCanvasProgram::compile(&excessive_draw).expect("shape compiles");
    let error = program.draw().unwrap_err();
    assert!(error.to_string().contains("invocations"), "{error}");

    let duplicate_group = draw_with_uniform_script(
        "pass:setBindGroup(0, bindGroup)\n            pass:setBindGroup(0, bindGroup)",
        false,
    );
    let mut program = GpuCanvasProgram::compile(&duplicate_group).expect("shape compiles");
    let error = program.draw().unwrap_err();
    assert!(error.to_string().contains("already bound"), "{error}");

    let duplicate_binding = draw_with_uniform_script("", true);
    let mut program = GpuCanvasProgram::compile(&duplicate_binding).expect("shape compiles");
    let error = program.draw().unwrap_err();
    assert!(error.to_string().contains("duplicated"), "{error}");
}

fn minimal_draw_script(draw: &str) -> String {
    format!(
        r#"
return function(context)
    local canvas = context:gpuCanvas()
    return {{
        drawCanvas = function()
            canvas:resize(32, 32)
            local shader = context:shader("scene")
            local pipeline = GPUPipeline.new {{
                vertex = shader,
                fragment = shader,
                vertexLayout = {{}},
                colorTargets = {{ {{ format = "rgba8unorm" }} }},
            }}
            local pass = canvas:beginRenderPass {{ color = {{ {{
                loadOp = "clear", storeOp = "store", clearColor = {{ 0, 0, 0, 1 }}
            }} }} }}
            pass:setPipeline(pipeline)
            {draw}
            pass:finish()
        end,
    }}
end
"#
    )
}

fn draw_with_uniform_script(pass_bindings: &str, duplicate_binding: bool) -> String {
    let ubos = if duplicate_binding {
        "{ { slot = 0, buffer = ubo }, { slot = 0, buffer = ubo } }"
    } else {
        "{ { slot = 0, buffer = ubo } }"
    };
    format!(
        r#"
return function(context)
    local canvas = context:gpuCanvas()
    return {{
        drawCanvas = function()
            canvas:resize(32, 32)
            local shader = context:shader("scene")
            local pipeline = GPUPipeline.new {{
                vertex = shader,
                fragment = shader,
                vertexLayout = {{}},
                colorTargets = {{ {{ format = "rgba8unorm" }} }},
            }}
            local data = buffer.create(16)
            local ubo = GPUBuffer.new {{ size = 16, usage = "uniform", data = data }}
            local layout = GPUBindGroupLayout.new {{ groupIndex = 0, shader = shader }}
            local bindGroup = GPUBindGroup.new {{ layout = layout, ubos = {ubos} }}
            local pass = canvas:beginRenderPass {{ color = {{ {{
                loadOp = "clear", storeOp = "store", clearColor = {{ 0, 0, 0, 1 }}
            }} }} }}
            pass:setPipeline(pipeline)
            {pass_bindings}
            pass:draw(3)
            pass:finish()
        end,
    }}
end
"#
    )
}
