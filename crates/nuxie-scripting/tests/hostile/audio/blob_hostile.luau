--! attacks: Audio.Blob.from with huge inputs kept alive, read at hostile offsets, toBuffer copies; expect: error(not enough memory); within: 20s
assert(not pcall(Audio.Blob.from, nil))
assert(not pcall(Audio.Blob.from, 12))
assert(not pcall(Audio.Blob.from, {}))
local empty = Audio.Blob.from("")
assert(empty.size == 0)
assert(#buffer.tostring(empty:toBuffer()) == 0)
local b = Audio.Blob.from(string.rep("a", 1024))
local out = buffer.create(16)
for _, off in ipairs({1024, 1023, 2 ^ 32, 2 ^ 53}) do
  local ok, n = pcall(function() return b:read(off, out) end)
  assert(not ok or n <= 1, "read past the end at " .. tostring(off))
end
assert(not pcall(function() return b:read(-1, out) end))
assert(not pcall(function() return b:read(0 / 0, out) end))
local kept = {Audio.Blob.from(string.rep("b", 16 * 1024 * 1024))}
-- toBuffer copies into the VM heap, so the VM's 64 MiB heap refuses (a VM
-- out-of-memory is not catchable by pcall and ends the chunk).
local copies = {}
for i = 1, 16 do copies[i] = kept[1]:toBuffer() end
return {}
