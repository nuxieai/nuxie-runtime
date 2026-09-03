//! Narrow regressions for the Lua C API descriptor/coercion rules at e949498e.
use super::*;
use crate::vm::{RoutedTestFactory, ScriptVm};
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;

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
