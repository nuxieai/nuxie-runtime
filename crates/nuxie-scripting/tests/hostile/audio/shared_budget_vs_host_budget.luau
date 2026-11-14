--! attacks: shared-object budget (512 MiB) larger than the VM's 64 MiB host budget: zero-filled regions kept alive; expect: ok; within: 30s; harness: instance
-- Each Region.create zero-fills its storage on the host. The VM's host memory
-- budget is 64 MiB; creation must be refused well before the process holds
-- hundreds of MiB. Spread over callbacks so no single call hits the 1 s timeout.
local kept = {}
local refused = nil
local function grow()
  for i = 1, 3 do
    if refused then return end
    local ok, region = pcall(Audio.Region.create, 32 * 1024 * 1024)
    if not ok then refused = region return end
    table.insert(kept, region)
  end
end
return function()
  return {
    init = function(self) grow() end,
    draw = function(self, renderer) grow() end,
    advance = function(self, dt) grow() return true end,
    update = function(self)
      grow()
      assert(#kept * 32 <= 128, "holding " .. (#kept * 32) .. " MiB of shared regions")
    end,
  }
end
