--! attacks: blob property set to huge strings/buffers across many instances (charged), nil and wrong types; expect: ok; within: 30s; harness: instance; viewmodels: data_bind_blob_test.riv
local big = string.rep("x", 16 * 1024 * 1024)
local kept = {}
local refused = false
local function assign(n)
  for i = 1, n do
    if refused then return end
    local inst = Data.Main.new()
    local ok = pcall(function() inst:getBlob("xml").value = big end)
    if not ok then refused = true return end
    table.insert(kept, inst)
  end
end
return function()
  return {
    init = function(self)
      local p = Data.Main.new():getBlob("xml")
      for _, v in ipairs({1, true, {}, print}) do assert(not pcall(function() p.value = v end)) end
      p.value = nil
      p.value = ""
      p.value = buffer.create(0)
      assign(4)
    end,
    advance = function(self, dt) assign(8) return true end,
    update = function(self)
      assign(8)
      assert(refused, "kept " .. #kept .. " instances with 16 MiB blobs")
    end,
  }
end
