//! Narrow regressions for the Lua C API descriptor/coercion rules at e949498e.
use super::*;
use crate::vm::{RoutedTestFactory, ScriptVm};
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;

// scripting_context_test.cpp: closing an enclosing pass expires the nested
// script wrapper before any subsequent draw validation or recording.
#[test]
fn pass_closed_by_enclosing_pass_expires_for_script() {
    use nuxie_ore_metal::{
        ore_cmd::{
            ore_command_buffer::OreCommandBuffer, ore_render_pass_recording::RenderPassRecording,
        },
        render_pass::RenderPassApi,
    };
    let vm = recording_vm();
    let ore = context(vm.lua()).unwrap();
    let stream = Rc::new(RefCell::new(OreCommandBuffer::default()));
    let mut outer = RenderPassRecording::new(
        Some(ore.borrow().contextBase()),
        stream.clone(),
        &RenderPassDesc::default(),
    );
    let inner = RenderPassRecording::new(
        Some(ore.borrow().contextBase()),
        stream,
        &RenderPassDesc::default(),
    );
    let rp = vm
        .lua()
        .create_userdata(pass::Pass {
            pass: Some(Box::new(inner)),
            deferred_bind_groups: None,
            finished: false,
            sample_count: 1,
            pipeline_set: false,
            draw_call_count: 0,
            label: String::new(),
        })
        .unwrap();
    vm.lua().globals().set("rp", rp.clone()).unwrap();
    outer.finish();
    assert!(
        rp.borrow::<pass::Pass>()
            .unwrap()
            .pass
            .as_ref()
            .unwrap()
            .activeToken()
            .upgrade()
            .unwrap()
            .isFinished()
    );
    assert!(!ore.borrow().hasOpenRenderPasses());
    let error = vm.lua().load("rp:draw(3)").exec().unwrap_err();
    assert!(error.to_string().contains("render pass expired"));
}

fn recording_vm() -> ScriptVm {
    let vm = ScriptVm::new();
    let ore: OreContextHandle = Rc::new(RefCell::new(DeferredOreContext::fromReal(None)));
    let module = ore
        .borrow_mut()
        .makeShaderModule(&ShaderModuleDesc {
            code: Some(b"recorded test module"),
            codeSize: 20,
            ..ShaderModuleDesc::default()
        })
        .unwrap();
    let mut factory = nuxie_render_api::PersistentFactory::new(RoutedTestFactory {
        inner: nuxie_render_api::RecordingFactory::new(),
        ore: Some(ore),
        canvas_host: None,
    });
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    vm.lua()
        .globals()
        .set(
            "shader",
            vm.lua()
                .create_userdata(Shader {
                    entries: vec![
                        ShaderEntry {
                            stage: 0,
                            logical: "vertex".into(),
                            physical: "vertex".into(),
                            module: module.clone(),
                        },
                        ShaderEntry {
                            stage: 1,
                            logical: "fragment".into(),
                            physical: "fragment".into(),
                            module,
                        },
                    ],
                })
                .unwrap(),
        )
        .unwrap();
    let canvas =
        Canvas::create(vm.lua(), RendererBindings::for_lua(vm.lua()).unwrap(), 0, 0).unwrap();
    vm.lua().globals().set("canvas", canvas).unwrap();
    vm
}

// cd04cd33: bindings made before the first pipeline wait at the Lua owner,
// rather than issuing a backend bind with no pipeline layout.
#[test]
fn bind_groups_wait_for_first_pipeline() {
    use nuxie_ore_metal::cmd::command_stream::CommandReader;
    use nuxie_ore_metal::ore_cmd::{
        ore_command_buffer::OreCommandBuffer,
        ore_commands::{CommandType, SetBindGroupCmd, SetPipelineCmd},
        ore_render_pass_recording::RenderPassRecording,
    };
    let vm = recording_vm();
    let ore = context(vm.lua()).unwrap();
    let stream = Rc::new(RefCell::new(OreCommandBuffer::default()));
    let inner = RenderPassRecording::new(
        Some(ore.borrow().contextBase()),
        stream.clone(),
        &RenderPassDesc {
            colorCount: 0,
            ..RenderPassDesc::default()
        },
    );
    let rp = vm
        .lua()
        .create_userdata(pass::Pass {
            pass: Some(Box::new(inner)),
            deferred_bind_groups: None,
            finished: false,
            sample_count: 1,
            pipeline_set: false,
            draw_call_count: 0,
            label: String::new(),
        })
        .unwrap();
    vm.lua().globals().set("rp", rp.clone()).unwrap();
    vm.lua()
        .load(
            r#"
        layout = GPUBindGroupLayout.new { shader = shader }
        bg = GPUBindGroup.new { layout = layout }
        pipeline = GPUPipeline.new { vertex = shader, vertexLayout = {}, colorTargets = false }
    "#,
        )
        .exec()
        .unwrap();
    let before = stream.borrow().command_bytes().len();
    vm.lua().load("rp:setBindGroup(0, bg)").exec().unwrap();
    assert_eq!(stream.borrow().command_bytes().len(), before);
    assert!(
        rp.borrow::<pass::Pass>()
            .unwrap()
            .deferred_bind_groups
            .is_some()
    );
    vm.lua().load("rp:setPipeline(pipeline)").exec().unwrap();
    {
        let stream = stream.borrow();
        let mut reader = CommandReader::new(&stream.command_bytes()[before..], stream.blob_bytes());
        assert_eq!(reader.next::<CommandType>(), Some(CommandType::setPipeline));
        assert!(reader.next::<SetPipelineCmd>().is_some());
        assert_eq!(
            reader.next::<CommandType>(),
            Some(CommandType::setBindGroup)
        );
        let binding = reader.next::<SetBindGroupCmd>().unwrap();
        assert_eq!(binding.groupIndex, 0);
        assert_eq!(binding.dynamicOffsetCount, 0);
        assert_eq!(reader.next::<CommandType>(), None);
        assert!(!reader.overrun());
    }
    assert!(rp.borrow::<pass::Pass>().unwrap().pipeline_set);
    vm.lua().load("rp:finish()").exec().unwrap();
}

#[test]
fn bind_group_layout_fragment_requires_an_actual_fragment_entry() {
    let vm = recording_vm();
    let data: AnyUserData = vm.lua().globals().get("shader").unwrap();
    let mut vertex_only = data.borrow::<Shader>().unwrap().clone();
    vertex_only.entries.retain(|entry| entry.stage == 0);
    vm.lua()
        .globals()
        .set("vertexOnly", vm.lua().create_userdata(vertex_only).unwrap())
        .unwrap();
    vm.lua().load(r#"
        for _, fragment in {false, {}, vertexOnly} do
            local ok, err = pcall(function()
                GPUBindGroupLayout.new {shader = shader, fragment = fragment}
            end)
            assert(not ok)
            assert(string.find(err, "'fragment' must be a Shader with a @fragment entry point", 1, true))
        end
        assert(GPUBindGroupLayout.new {shader = shader, fragment = shader} ~= nil)
    "#).exec().unwrap();
}

#[test]
fn split_stage_fragment_bindings_reach_explicit_and_auto_layouts() {
    let vm = recording_vm();
    let ore = context(vm.lua()).unwrap();
    let map = [
        3, 2, 14, 0, 1, 0, 0, 0, 9, 0, 0, 0, 0, 7, 0, 2, 0, 255, 255, 1, 0, 255, 255, 0, 0, 0,
    ];
    let module = ore
        .borrow_mut()
        .makeShaderModule(&ShaderModuleDesc {
            code: Some(b"fragment test module"),
            codeSize: 20,
            bindingMapBytes: Some(&map),
            bindingMapSize: map.len() as u32,
            ..ShaderModuleDesc::default()
        })
        .unwrap();
    vm.lua()
        .globals()
        .set(
            "fragment",
            vm.lua()
                .create_userdata(Shader {
                    entries: vec![ShaderEntry {
                        stage: 1,
                        logical: "fragment".into(),
                        physical: "fragment".into(),
                        module,
                    }],
                })
                .unwrap(),
        )
        .unwrap();
    vm.lua()
        .load(
            r#"
        explicitSplitLayout = GPUBindGroupLayout.new {shader = shader, fragment = fragment}
        splitPipeline = GPUPipeline.new {vertex = shader, fragment = fragment, vertexLayout = {}}
        autoSplitLayout = splitPipeline:getBindGroupLayout(0)
    "#,
        )
        .exec()
        .unwrap();
    for name in ["explicitSplitLayout", "autoSplitLayout"] {
        let data: AnyUserData = vm.lua().globals().get(name).unwrap();
        let layout = data.borrow::<Layout>().unwrap();
        let entries = layout.resource.bindGroupLayoutBase().unwrap().entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].binding, 7);
        assert_eq!(entries[0].nativeSlotFS, 1);
        assert_eq!(entries[0].nativeSlotVS, u32::MAX);
        assert_eq!(entries[0].visibility.mask, 2);
    }
}

#[test]
fn integer_and_lua_numeric_string_descriptor_values_are_preserved() {
    let vm = recording_vm();
    let desc: Table = vm.lua().load("return { integer = 16, decimal = 1.5, numericString = ' 0x10 ', invalid = false, dynamicUBOs = { 0, ' 0x2 ', false, 3 } }").eval().unwrap();
    assert!(matches!(
        desc.get::<Value>("integer").unwrap(),
        Value::Integer(16)
    ));
    assert_eq!(number(&desc, "integer", 99.0).unwrap(), 16.0);
    assert_eq!(number(&desc, "decimal", 99.0).unwrap(), 1.5);
    assert_eq!(number(&desc, "numericString", 99.0).unwrap(), 16.0);
    assert_eq!(number(&desc, "invalid", 99.0).unwrap(), 99.0);
    assert_eq!(string(&desc, "integer").unwrap().as_deref(), Some("16"));
    desc.set("large", 1e30).unwrap();
    let formatted: String = vm.lua().load("return tostring(1e30)").eval().unwrap();
    assert_eq!(string(&desc, "large").unwrap(), Some(formatted));
    desc.set("terminated", "rgba8unorm\0ignored").unwrap();
    assert_eq!(
        string(&desc, "terminated").unwrap().as_deref(),
        Some("rgba8unorm")
    );
    assert_eq!(dynamic_ubo_bindings(&desc).unwrap(), [0, 2, 3]);
    vm.lua().load(r#"
        local b = GPUBuffer.new { size = 16, usage = 'vertex' }
        assert(b.size == 16)
        b:write(buffer.create(4), ' 0x4 ', false, '4')
        local t = GPUTexture.new { width = 4, height = ' 0x8 ', mipmaps = 2 }
        assert(t.width == 4 and t.height == 8)
        local v = t:view { baseMipLevel = 1, mipCount = 1 }
        assert(v ~= nil)
        pipeline = GPUPipeline.new {
            vertex = shader, vertexLayout = {{ stride = 16, attributes = {{ slot = 1, offset = 4 }} }},
            sampleCount = 4,
        }
    "#).exec().unwrap();
    let pipeline: AnyUserData = vm.lua().globals().get("pipeline").unwrap();
    assert_eq!(
        pipeline
            .borrow::<pipeline::Pipeline>()
            .unwrap()
            .sample_count,
        4
    );
}

#[test]
fn texture_descriptor_metafields_follow_source_order_and_failure_boundary() {
    let vm = recording_vm();
    vm.lua().load(r#"
        local reads = {}
        GPUTexture.new(setmetatable({}, {__index = function(_, key)
            table.insert(reads, key)
            if key == 'width' or key == 'height' then return 2 end
        end}))
        assert(table.concat(reads, ',') == 'width,height,format,type,renderTarget,sampleCount,mipmaps,layers')
        reads = {}
        assert(not pcall(function()
            GPUTexture.new(setmetatable({}, {__index = function(_, key)
                table.insert(reads, key)
                if key == 'width' then return 0 end
                if key == 'height' then return 2 end
            end}))
        end))
        assert(table.concat(reads, ',') == 'width,height')
    "#).exec().unwrap();
}

#[test]
fn vertex_layout_metafields_follow_source_order_and_failure_boundary() {
    let vm = recording_vm();
    vm.lua().load(r#"
        local reads = {}
        local invalidFormat = false
        local attribute = setmetatable({}, {__index = function(_, key)
            table.insert(reads, 'attribute.' .. key)
            if key == 'format' then
                return invalidFormat and 'invalid-format' or 'float32'
            end
            return 0
        end})
        local layout = setmetatable({}, {__index = function(_, key)
            table.insert(reads, key)
            if key == 'stride' then return 4 end
            if key == 'stepMode' then return 'instance' end
            if key == 'attributes' then return {attribute} end
        end})
        local function makePipeline()
            return GPUPipeline.new {vertex = shader, vertexLayout = {layout}}
        end
        assert(makePipeline() ~= nil)
        assert(table.concat(reads, ',') == 'stride,stepMode,attributes,attribute.format,attribute.slot,attribute.offset')

        reads = {}
        invalidFormat = true
        assert(not pcall(makePipeline))
        assert(table.concat(reads, ',') == 'stride,stepMode,attributes,attribute.format')
    "#).exec().unwrap();
}

#[test]
fn render_pass_labels_follow_lua_string_coercion_and_retain_the_value() {
    let vm = recording_vm();
    vm.lua()
        .load(
            r#"
        local texture = GPUTexture.new { width = 4, height = 4 }
        function labeledPass(label)
            local desc = {label = label, color = {{view = texture:view(), storeOp = 'store'}}}
            local pass = canvas:beginRenderPass(desc)
            desc.label = 'changed'
            pass:finish()
            return pass
        end
    "#,
        )
        .exec()
        .unwrap();
    for (expression, expected) in [
        ("'lighting'", "lighting"),
        ("123", "123"),
        ("false", ""),
        ("nil", ""),
        ("'prefix' .. string.char(0) .. 'suffix'", "prefix"),
    ] {
        let pass: AnyUserData = vm
            .lua()
            .load(format!("return labeledPass({expression})"))
            .eval()
            .unwrap();
        assert_eq!(pass.borrow::<pass::Pass>().unwrap().label, expected);
    }
}

#[test]
fn optional_non_tables_default_but_wrong_resource_userdata_errors() {
    let vm = recording_vm();
    vm.lua().load(r#"
        local texture = GPUTexture.new { width = 4, height = 4 }
        local view = texture:view(false)
        local sampler = GPUSampler.new('ignored')
        local layout = GPUBindGroupLayout.new { shader = shader, dynamicUBOs = false }
        local bg = GPUBindGroup.new { layout = layout, ubos = false, textures = 'ignored', samplers = 42 }
        local pipeline = GPUPipeline.new {
            vertex = shader, vertexLayout = {{ stride = 4 }},
            colorTargets = false, depthStencil = false,
            stencilFront = false, stencilBack = 17, bindGroupLayouts = false,
        }
        local colorPipeline = GPUPipeline.new {
            vertex = shader, vertexLayout = {},
            colorTargets = {{ format = 'rgba8unorm', blend = false }},
        }
        assert(not pcall(function()
            GPUPipeline.new { vertex = shader, vertexLayout = {}, bindGroupLayouts = {sampler} }
        end))
        assert(not pcall(function()
            GPUPipeline.new { vertex = shader, vertexLayout = {}, bindGroupLayouts = {false} }
        end))
        assert(not pcall(function()
            canvas:beginRenderPass { color = {{ view = view, storeOp = 'store', resolveTarget = sampler }} }
        end))
        assert(not pcall(function()
            canvas:beginRenderPass { color = {{ view = view, loadOp = false, storeOp = 'store' }} }
        end))
        local pass = canvas:beginRenderPass {
            color = {{ view = view, storeOp = 'store', clearColor = false }}, depthStencil = false,
        }
        pass:setPipeline(colorPipeline)
        pass:setBindGroup(0, bg, false)
        pass:draw(3, false, false, false)
        pass:finish()
        local depth = GPUTexture.new { width = 4, height = 4, format = 'depth32float' }
        local depthPass = canvas:beginRenderPass {
            color = false, depthStencil = {view = depth:view(), depthStoreOp = 'store', depthClearValue = false},
        }
        depthPass:finish()
    "#).exec().unwrap();
}

#[test]
fn descriptor_metamethods_can_reenter_the_selected_ore_context() {
    let vm = recording_vm();
    vm.lua()
        .load(
            r#"
        local lookups = {}
        local function descriptor(name, fields, inherited)
            return setmetatable(fields, {__index = function(_, key)
                lookups[name .. ':' .. key] = true
                -- A nested resource creation is valid even before a recorder
                -- has a device and therefore knows its capability limits.
                local nested = GPUBuffer.new {size = 4, usage = 'uniform'}
                assert(nested.size == 4)
                return inherited[key]
            end})
        end

        local texture = GPUTexture.new(descriptor('texture', {height = 1}, {width = 1}))
        local sampler = GPUSampler.new(descriptor('sampler', {}, {}))
        local layout = GPUBindGroupLayout.new(descriptor('layout', {shader = shader}, {}))
        local explicit = GPUPipeline.new(descriptor('explicit', {
            vertex = shader, vertexLayout = {},
        }, {bindGroupLayouts = {layout}}))
        local automatic = GPUPipeline.new(descriptor('automatic', {
            vertex = shader, vertexLayout = {},
        }, {}))
        assert(texture ~= nil and sampler ~= nil and layout ~= nil)
        assert(explicit ~= nil and automatic ~= nil)
        assert(lookups['texture:width'] and lookups['sampler:min'])
        assert(lookups['layout:groupIndex'])
        assert(lookups['explicit:bindGroupLayouts'] and lookups['explicit:sampleCount'])
        assert(lookups['automatic:bindGroupLayouts'] and lookups['automatic:sampleCount'])
    "#,
        )
        .exec()
        .unwrap();
}
