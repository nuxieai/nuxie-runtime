-- Authored P3-e oracle for rive-runtime d788e8ec src/lua/renderer/lua_gpu.cpp.
-- This stays a focused fixture: it is intentionally not a golden corpus row.
return function(context)
    local canvas = context:gpuCanvas()
    canvas:resize(16, 16)
    assert(canvas.width == 16)
    assert(canvas.height == 16)
    assert(canvas.format == "rgba8unorm")

    local shader = context:shader("scene")
    local vertices = buffer.create(48)
    local indices = buffer.create(6)
    local uniforms = buffer.create(512)
    local pixels = buffer.create(16)

    local vertexBuffer = GPUBuffer.new {
        size = 48,
        usage = { "vertex" },
        data = vertices,
        immutable = true,
        label = "fixture vertices",
    }
    local indexBuffer = GPUBuffer.new {
        size = 6,
        usage = "index",
        data = indices,
        immutable = true,
        label = "fixture indices",
    }
    local uniformBuffer = GPUBuffer.new {
        size = 512,
        usage = "uniform",
        data = uniforms,
        label = "fixture uniforms",
    }
    assert(vertexBuffer.size == 48)
    uniformBuffer:write(uniforms, 0, 0, 256)

    local texture = GPUTexture.new {
        width = 2,
        height = 2,
        format = "rgba8unorm",
        type = "2d",
        renderTarget = false,
        sampleCount = 1,
        mipmaps = 1,
        layers = 1,
    }
    texture:upload {
        data = pixels,
        width = 2,
        height = 2,
        depth = 1,
        x = 0,
        y = 0,
        z = 0,
        mipLevel = 0,
        layer = 0,
        bytesPerRow = 8,
        rowsPerImage = 2,
    }
    assert(texture.width == 2)
    assert(texture.height == 2)
    assert(texture.format == "rgba8unorm")
    local textureView = texture:view {
        dimension = "2d",
        baseMipLevel = 0,
        mipCount = 1,
        baseLayer = 0,
        layerCount = 1,
    }
    assert(textureView.format == "rgba8unorm")

    local sampler = GPUSampler.new {
        min = "linear",
        mag = "linear",
        mipmap = "nearest",
        wrapU = "repeat",
        wrapV = "mirror-repeat",
        wrapW = "clamp-to-edge",
        compare = "always",
        minLod = 0,
        maxLod = 4,
        maxAnisotropy = 1,
    }

    local explicitLayout = GPUBindGroupLayout.new {
        groupIndex = 0,
        shader = shader,
        dynamicUBOs = { 0 },
    }
    local bindGroup = GPUBindGroup.new {
        layout = explicitLayout,
        ubos = { { slot = 0, buffer = uniformBuffer, offset = 0, size = 256 } },
        textures = { { slot = 1, view = textureView } },
        samplers = { { slot = 2, sampler = sampler } },
    }

    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "vs_main" },
        fragment = { module = shader, entryPoint = "fs_main" },
        vertexLayout = { {
            stride = 16,
            stepMode = "vertex",
            attributes = { { format = "float32x4", slot = 0, offset = 0 } },
        } },
        colorTargets = { {
            format = "rgba8unorm",
            writeMask = "rgba",
            blend = {
                srcColor = "src-alpha",
                dstColor = "one-minus-src-alpha",
                colorOp = "add",
                srcAlpha = "one",
                dstAlpha = "one-minus-src-alpha",
                alphaOp = "add",
            },
        } },
        depthStencil = {
            format = "depth32float",
            compare = "less-equal",
            write = true,
            depthBias = 0,
            depthBiasSlopeScale = 0,
            depthBiasClamp = 0,
        },
        stencilFront = {
            compare = "always", failOp = "keep", depthFailOp = "keep", passOp = "replace",
        },
        stencilBack = {
            compare = "always", failOp = "keep", depthFailOp = "keep", passOp = "replace",
        },
        stencilReadMask = 255,
        stencilWriteMask = 255,
        bindGroupLayouts = { explicitLayout },
        cullMode = "none",
        winding = "ccw",
        topology = "triangle-list",
        sampleCount = 1,
    }

    -- Exercise the auto-layout method independently of the explicit layout.
    local autoPipeline = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    local _autoLayout = autoPipeline:getBindGroupLayout(0)

    return {
        drawCanvas = function()
            local canvasView = canvas:colorView()
            assert(canvasView.format == canvas.format)
            local pass = canvas:beginRenderPass {
                color = { {
                    view = canvasView,
                    loadOp = "clear",
                    storeOp = "store",
                    clearColor = { 0.125, 0.25, 0.5, 1.0 },
                } },
            }
            pass:setPipeline(pipeline)
            pass:setVertexBuffer(0, vertexBuffer)
            pass:setIndexBuffer(indexBuffer, "uint16")
            pass:setBindGroup(0, bindGroup, { 0 })
            pass:setViewport(0, 0, 16, 16)
            pass:setScissorRect(0, 0, 16, 16)
            pass:setStencilReference(7)
            pass:setBlendColor(1, 0.5, 0.25, 1)
            pass:drawIndexed(3, 1, 0, 0, 0)
            pass:finish()
        end,
    }
end
