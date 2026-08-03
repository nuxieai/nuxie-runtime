-- Residue anchor: crates/nuxie-scripting/tests/fixtures/lua-gpu-semantic-combinations.luau
return function(context)
    local canvas = context:gpuCanvas()
    canvas:resize(16, 16)
    local shader = context:shader("scene")
    local mode = 1

    local oneTarget = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    local msaaTargets = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = {
            { format = "rgba8unorm" },
            { format = "rgba16float", writeMask = "rg" },
        },
        sampleCount = 4,
    }
    local fourTargets = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = {
            { format = "r8unorm" },
            { format = "rg8unorm" },
            { format = "rgba8unorm" },
            { format = "rgba16float" },
        },
    }
    local threeTargets = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = {
            { format = "r16float" },
            { format = "rg32float" },
            { format = "bgra8unorm" },
        },
    }
    local depthOnly = GPUPipeline.new {
        vertex = shader,
        vertexLayout = {},
        depthStencil = {
            format = "depth32float",
            compare = "less-equal",
            write = true,
        },
    }

    local msaa0 = GPUTexture.new {
        width = 16, height = 16, format = "rgba8unorm",
        renderTarget = true, sampleCount = 4,
    }
    local msaa1 = GPUTexture.new {
        width = 16, height = 16, format = "rgba16float",
        renderTarget = true, sampleCount = 4,
    }
    local resolve1 = GPUTexture.new {
        width = 16, height = 16, format = "rgba16float", renderTarget = true,
    }
    local targets = {
        GPUTexture.new { width = 16, height = 16, format = "r8unorm", renderTarget = true },
        GPUTexture.new { width = 16, height = 16, format = "rg8unorm", renderTarget = true },
        GPUTexture.new { width = 16, height = 16, format = "rgba8unorm", renderTarget = true },
        GPUTexture.new { width = 16, height = 16, format = "rgba16float", renderTarget = true },
    }
    local three = {
        GPUTexture.new { width = 16, height = 16, format = "r16float", renderTarget = true },
        GPUTexture.new { width = 16, height = 16, format = "rg32float", renderTarget = true },
        GPUTexture.new { width = 16, height = 16, format = "bgra8unorm", renderTarget = true },
    }
    local depth = GPUTexture.new {
        width = 16, height = 16, format = "depth32float", renderTarget = true,
    }

    return {
        advance = function()
            mode = mode % 5 + 1
            return true
        end,
        drawCanvas = function()
            if mode == 1 then
                local first = canvas:beginRenderPass { color = { {
                    loadOp = "clear", storeOp = "store", clearColor = { 1, 0, 0, 1 },
                } } }
                first:setPipeline(oneTarget)
                first:draw(3)
                first:setViewport(0, 0, 8, 8)
                first:draw(3, 2)
                first:finish()

                local second = canvas:beginRenderPass { color = { {
                    loadOp = "load", storeOp = "store",
                } } }
                second:setPipeline(oneTarget)
                second:draw(3)
                second:finish()
            elseif mode == 2 then
                local pass = canvas:beginRenderPass { color = {
                    {
                        view = msaa0:view(), resolveTarget = canvas:colorView(),
                        loadOp = "clear", storeOp = "discard", clearColor = { 0, 0, 0, 1 },
                    },
                    {
                        view = msaa1:view(), resolveTarget = resolve1:view(),
                        loadOp = "clear", storeOp = "discard", clearColor = { 0, 0, 0, 1 },
                    },
                } }
                pass:setPipeline(msaaTargets)
                pass:draw(3)
                pass:finish()
            elseif mode == 3 then
                local pass = canvas:beginRenderPass { color = {
                    { view = targets[1]:view(), loadOp = "clear", storeOp = "store" },
                    { view = targets[2]:view(), loadOp = "clear", storeOp = "store" },
                    { view = targets[3]:view(), loadOp = "clear", storeOp = "store" },
                    { view = targets[4]:view(), loadOp = "clear", storeOp = "store" },
                } }
                pass:setPipeline(fourTargets)
                pass:draw(3)
                pass:finish()
            elseif mode == 4 then
                local pass = canvas:beginRenderPass { color = {
                    { view = three[1]:view(), loadOp = "clear", storeOp = "store" },
                    { view = three[2]:view(), loadOp = "clear", storeOp = "store" },
                    { view = three[3]:view(), loadOp = "clear", storeOp = "store" },
                } }
                pass:setPipeline(threeTargets)
                pass:draw(3)
                pass:finish()
            else
                local pass = canvas:beginRenderPass {
                    depthStencil = {
                        view = depth:view(),
                        depthLoadOp = "clear",
                        depthStoreOp = "store",
                        depthClearValue = 0.75,
                    },
                }
                pass:setPipeline(depthOnly)
                pass:draw(3)
                pass:finish()
            end
        end,
    }
end
