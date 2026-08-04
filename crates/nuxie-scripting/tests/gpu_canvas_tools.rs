//! Editor-facing GPU-canvas execution contract.
//!
//! These tests deliberately exercise Luau source through the same pure-Rust
//! VM crate used by runtime ScriptAssets. The returned draw plan is the typed
//! handoff to the browser renderer; JavaScript never interprets script state.
#![cfg(feature = "luau")]

use nuxie_scripting::gpu_canvas::{
    GpuCanvasAttachmentView, GpuCanvasProgram, GpuCanvasUniformBuffer, GpuCanvasVertexAttribute,
    GpuCanvasVertexBuffer, GpuCanvasVertexLayout, MAX_CPU_BUFFER_BYTES, MAX_GPU_CANVAS_DIMENSION,
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

const LUA_GPU_FULL_SURFACE: &str = include_str!("fixtures/lua-gpu-full-surface.luau");
// Authored oracle: crates/nuxie-scripting/tests/fixtures/lua-gpu-semantic-combinations.luau
const LUA_GPU_SEMANTIC_COMBINATIONS: &str =
    include_str!("fixtures/lua-gpu-semantic-combinations.luau");
const LUA_GPU_CEILINGS: &str = include_str!("fixtures/lua-gpu-ceilings.luau");

#[test]
fn authored_lua_gpu_ceiling_fixture_retains_per_pass_pipeline_and_resources() {
    let mut program =
        GpuCanvasProgram::compile(LUA_GPU_CEILINGS).expect("GPU ceiling fixture compiles");

    let plan = program
        .draw()
        .expect("one submission may use distinct pipelines and resources");
    assert_eq!(plan.pipelines.len(), 2);
    assert_eq!(plan.render_passes.len(), 2);
    assert_eq!(plan.render_passes[0].draws[0].pipeline_index, 0);
    assert_eq!(plan.render_passes[1].draws[0].pipeline_index, 1);
    assert_eq!(plan.pipelines[0].pipeline_state.cull_mode, "none");
    assert_eq!(plan.pipelines[1].pipeline_state.cull_mode, "back");
    assert_eq!(
        f32::from_le_bytes(
            plan.pipelines[0].uniform_buffers[0].bytes[0..4]
                .try_into()
                .unwrap()
        ),
        1.0
    );
    assert_eq!(
        f32::from_le_bytes(
            plan.pipelines[1].uniform_buffers[0].bytes[0..4]
                .try_into()
                .unwrap()
        ),
        2.0
    );
}

#[test]
fn authored_lua_gpu_ceiling_fixture_retains_explicit_empty_finish() {
    let mut program =
        GpuCanvasProgram::compile(LUA_GPU_CEILINGS).expect("GPU ceiling fixture compiles");
    program.advance(0.0).expect("select empty finished pass");

    let plan = program
        .draw()
        .expect("finish closes and retains a pass without draws or a pipeline");
    assert!(plan.pipelines.is_empty());
    assert_eq!(plan.render_passes.len(), 1);
    assert!(plan.render_passes[0].draws.is_empty());
    assert_eq!(plan.render_passes[0].color_attachments[0].store_op, "store");
}

#[test]
fn authored_lua_gpu_ceiling_fixture_keeps_external_identity_across_submissions() {
    let mut program =
        GpuCanvasProgram::compile(LUA_GPU_CEILINGS).expect("GPU ceiling fixture compiles");
    program
        .advance(0.0)
        .expect("skip multi-pipeline submission");
    program
        .advance(0.0)
        .expect("select external attachment seed");

    let seed = program
        .draw()
        .expect("external attachment seed is retained");
    let seeded_resource = match &seed.render_passes[0].color_attachments[0].view {
        GpuCanvasAttachmentView::Texture(texture) => texture.resource_id,
        GpuCanvasAttachmentView::Canvas => panic!("seed must target the external texture"),
    };

    program
        .advance(0.0)
        .expect("select external texture sample");
    let sample = program.draw().expect("external texture sample is retained");
    assert_eq!(sample.pipelines.len(), 1);
    assert_eq!(sample.pipelines[0].texture_bindings.len(), 1);
    assert_eq!(
        sample.pipelines[0].texture_bindings[0].resource_id,
        seeded_resource
    );
}

#[test]
fn authored_lua_gpu_ceiling_fixture_rejects_an_orphan_before_a_finished_pass() {
    let mut program =
        GpuCanvasProgram::compile(LUA_GPU_CEILINGS).expect("GPU ceiling fixture compiles");
    for step in 0..4 {
        program
            .advance(0.0)
            .unwrap_or_else(|error| panic!("advance {step} selects orphan lifecycle: {error}"));
    }

    let error = program
        .draw()
        .expect_err("a later finished pass must not hide an earlier orphan");
    assert!(error.to_string().contains("left open"), "{error}");
}

#[test]
fn authored_lua_gpu_semantic_combinations_retain_exact_pass_structure() {
    let mut program = GpuCanvasProgram::compile(LUA_GPU_SEMANTIC_COMBINATIONS)
        .expect("semantic combinations fixture compiles");

    let repeated = program.draw().expect("repeated passes produce a plan");
    assert_eq!(repeated.render_passes.len(), 2);
    assert_eq!(repeated.render_passes[0].draws.len(), 2);
    assert_eq!(repeated.render_passes[1].draws.len(), 1);
    assert_eq!(
        repeated.render_passes[0].draws[1].pass_state.viewport,
        Some([0.0, 0.0, 8.0, 8.0])
    );
    assert_eq!(
        repeated.render_passes[1].color_attachments[0].load_op,
        "load"
    );

    program.advance(0.0).expect("select MSAA resolves");
    let resolves = program.draw().expect("external resolves produce a plan");
    let colors = &resolves.render_passes[0].color_attachments;
    assert_eq!(colors.len(), 2);
    assert!(matches!(
        colors[0].view,
        GpuCanvasAttachmentView::Texture(_)
    ));
    assert!(matches!(
        colors[0].resolve_target,
        Some(GpuCanvasAttachmentView::Canvas)
    ));
    let source_id = match &colors[1].view {
        GpuCanvasAttachmentView::Texture(texture) => texture.resource_id,
        GpuCanvasAttachmentView::Canvas => panic!("second source must be external"),
    };
    let resolve_id = match colors[1].resolve_target.as_ref().unwrap() {
        GpuCanvasAttachmentView::Texture(texture) => texture.resource_id,
        GpuCanvasAttachmentView::Canvas => panic!("second resolve must be external"),
    };
    assert_ne!(source_id, resolve_id);

    program.advance(0.0).expect("select four targets");
    let four = program.draw().expect("four targets produce a plan");
    assert_eq!(four.pipeline_state.color_targets.len(), 4);
    assert_eq!(four.render_passes[0].color_attachments.len(), 4);

    program.advance(0.0).expect("select three targets");
    let three = program.draw().expect("three targets produce a plan");
    assert_eq!(three.pipeline_state.color_targets.len(), 3);
    assert_eq!(three.render_passes[0].color_attachments.len(), 3);
    assert_eq!(three.pipeline_state.color_targets[1].format, "rg32float");

    program.advance(0.0).expect("select depth only");
    let depth = program.draw().expect("depth-only pipeline produces a plan");
    assert!(depth.fragment_entry.is_none());
    assert!(depth.pipeline_state.color_targets.is_empty());
    assert!(depth.render_passes[0].color_attachments.is_empty());
    assert!(depth.render_passes[0].depth_stencil_attachment.is_some());
}

#[test]
fn authored_lua_gpu_full_surface_reaches_the_wgpu_plan() {
    let mut program =
        GpuCanvasProgram::compile(LUA_GPU_FULL_SURFACE).expect("full-surface fixture compiles");
    let plan = program
        .draw()
        .expect("full-surface fixture produces a plan");

    assert_eq!((plan.width, plan.height), (16, 16));
    assert_eq!(plan.clear_color, [0.125, 0.25, 0.5, 1.0]);
    assert_eq!(plan.vertex_buffers.len(), 1);
    assert_eq!(plan.uniform_buffers.len(), 1);
    assert_eq!(plan.uniform_buffers[0].bytes.len(), 256);
    assert_eq!(plan.vertex_layouts[0].step_mode, "vertex");
    assert_eq!(plan.index_buffer.as_ref().unwrap().format, "uint16");
    assert_eq!(plan.index_buffer.as_ref().unwrap().bytes.len(), 6);
    let indexed = plan
        .indexed_draw
        .as_ref()
        .expect("indexed draw is retained");
    assert_eq!((indexed.index_count, indexed.instance_count), (3, 1));
    assert_eq!(plan.texture_bindings.len(), 1);
    assert_eq!(plan.texture_bindings[0].uploads.len(), 1);
    assert_eq!(plan.texture_bindings[0].uploads[0].bytes_per_row, 8);
    assert_eq!(plan.sampler_bindings.len(), 1);
    assert_eq!(plan.sampler_bindings[0].address_mode_u, "repeat");
    assert_eq!(plan.sampler_bindings[0].compare.as_deref(), Some("always"));
    assert_eq!(plan.pipeline_state.color_targets[0].write_mask, "rgba");
    assert_eq!(plan.pipeline_state.topology, "triangle-list");
    assert_eq!(
        plan.pipeline_state
            .depth_stencil
            .as_ref()
            .unwrap()
            .depth_compare,
        "less-equal"
    );
    assert_eq!(plan.pass_state.viewport, Some([0.0, 0.0, 16.0, 16.0]));
    assert_eq!(plan.pass_state.scissor_rect, Some([0, 0, 16, 16]));
    assert_eq!(plan.pass_state.stencil_reference, 7);
    assert_eq!(plan.pass_state.blend_color, [1.0, 0.5, 0.25, 1.0]);
}

#[test]
fn gpu_canvas_transport_types_remain_public_at_the_scripting_seam() {
    let attribute = GpuCanvasVertexAttribute {
        shader_location: 2,
        offset: 8,
        format: "float32x2".to_owned(),
    };
    let layout = GpuCanvasVertexLayout {
        stride: 16,
        step_mode: "vertex".into(),
        attributes: vec![attribute.clone()],
    };
    let vertices = GpuCanvasVertexBuffer {
        slot: 1,
        bytes: vec![1, 2, 3, 4],
    };
    let uniform = GpuCanvasUniformBuffer {
        group: 0,
        binding: 3,
        bytes: vec![5, 6, 7, 8],
    };

    assert_eq!(layout.attributes, vec![attribute]);
    assert_eq!(vertices.slot, 1);
    assert_eq!(uniform.binding, 3);
}

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

    let no_occurrence = GpuCanvasProgram::compile("return function() return {} end").unwrap_err();
    assert!(
        no_occurrence.to_string().contains("GPUCanvas occurrence"),
        "{no_occurrence}"
    );

    let mut missing_draw =
        GpuCanvasProgram::compile("return function(context) context:gpuCanvas() return {} end")
            .expect("shape compiles before execution");
    let error = missing_draw.draw().unwrap_err();
    assert!(error.to_string().contains("drawCanvas"), "{error}");

    let mut infinite_draw = GpuCanvasProgram::compile(
        "return function(context)
            context:gpuCanvas()
            return { drawCanvas = function() while true do end end }
        end",
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
fn rejects_excessive_draw_counts_and_duplicate_resource_slots() {
    let excessive_draw = minimal_draw_script(&format!(
        "pass:draw({}, 2)",
        MAX_GPU_CANVAS_DRAW_INVOCATIONS
    ));
    let mut program = GpuCanvasProgram::compile(&excessive_draw).expect("shape compiles");
    let error = program.draw().unwrap_err();
    assert!(error.to_string().contains("invocations"), "{error}");

    let duplicate_draw = minimal_draw_script("pass:draw(3)\n            pass:draw(3)");
    let mut program = GpuCanvasProgram::compile(&duplicate_draw).expect("shape compiles");
    let plan = program.draw().expect("repeated draws are retained");
    assert_eq!(plan.render_passes[0].draws.len(), 2);

    let draw_before_pipeline = minimal_draw_script("pass:draw(3)").replace(
        "pass:setPipeline(pipeline)",
        "-- pipeline intentionally omitted",
    );
    let mut program = GpuCanvasProgram::compile(&draw_before_pipeline).expect("shape compiles");
    let error = program.draw().unwrap_err();
    assert!(
        error.to_string().contains("pipeline before draw"),
        "{error}"
    );

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
