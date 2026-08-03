-- Authored oracle for the adapted lua_gpu.cpp submission contract.
return function(context)
    local canvas = context:gpuCanvas()
    canvas:resize(16, 16)
    local shader = context:shader("scene")
    local mode = 1

    local firstBytes = buffer.create(16)
    buffer.writef32(firstBytes, 0, 1)
    local secondBytes = buffer.create(16)
    buffer.writef32(secondBytes, 0, 2)
    local firstUbo = GPUBuffer.new { size = 16, usage = "uniform", data = firstBytes }
    local secondUbo = GPUBuffer.new { size = 16, usage = "uniform", data = secondBytes }
    local layout = GPUBindGroupLayout.new { groupIndex = 0, shader = shader }
    local firstGroup = GPUBindGroup.new {
        layout = layout,
        ubos = { { slot = 0, buffer = firstUbo } },
    }
    local secondGroup = GPUBindGroup.new {
        layout = layout,
        ubos = { { slot = 0, buffer = secondUbo } },
    }
    local firstPipeline = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
        cullMode = "none",
    }
    local secondPipeline = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
        cullMode = "back",
    }
    local external = GPUTexture.new {
        width = 16,
        height = 16,
        format = "rgba8unorm",
        renderTarget = true,
    }
    local externalSampler = GPUSampler.new {}
    local externalGroup = GPUBindGroup.new {
        layout = layout,
        textures = { { slot = 0, view = external:view() } },
        samplers = { { slot = 1, sampler = externalSampler } },
    }

    return {
        advance = function()
            mode += 1
            return true
        end,
        drawCanvas = function()
            if mode == 1 then
                local first = canvas:beginRenderPass { color = { {
                    loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 },
                } } }
                first:setPipeline(firstPipeline)
                first:setBindGroup(0, firstGroup)
                first:draw(3)
                first:finish()

                local second = canvas:beginRenderPass { color = { {
                    loadOp = "load", storeOp = "store",
                } } }
                second:setPipeline(secondPipeline)
                second:setBindGroup(0, secondGroup)
                second:draw(3)
                second:finish()
            elseif mode == 2 then
                local empty = canvas:beginRenderPass { color = { {
                    loadOp = "clear", storeOp = "store", clearColor = { 1, 0, 0, 1 },
                } } }
                empty:finish()
            elseif mode == 3 then
                local seed = canvas:beginRenderPass { color = { {
                    view = external:view(),
                    loadOp = "clear", storeOp = "store", clearColor = { 1, 0, 0, 1 },
                } } }
                seed:finish()
            else
                local sample = canvas:beginRenderPass { color = { {
                    loadOp = "clear", storeOp = "store", clearColor = { 0, 0, 0, 1 },
                } } }
                sample:setPipeline(firstPipeline)
                sample:setBindGroup(0, externalGroup)
                sample:draw(3)
                sample:finish()
            end
        end,
    }
end
