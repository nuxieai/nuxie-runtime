--! attacks: Audio.streamReader on non-stream values and forged descriptors, oversized/forged records, many readers; expect: ok; within: 15s
for _, v in ipairs({nil, 1, "s", {}, {pcmStream = true}, {pcmStream = 1}, print}) do
  assert(not pcall(Audio.streamReader, v), "accepted " .. tostring(v))
end
local function descriptor(ringBytes, rate, channels, frames)
  return {
    pcmStream = true,
    ring = Audio.Ring.create(ringBytes),
    seeks = Audio.Region.create(8),
    sampleRate = rate,
    channels = channels,
    chunkFrames = frames,
  }
end
-- bad formats
for _, f in ipairs({{48000, 0, 4}, {48000, 2, 0}, {0, 2, 4}, {48000, 2, 2 ^ 32 - 1}, {48000, 2 ^ 32 - 1, 2 ^ 32 - 1}, {48000, 2, 0.5}, {48000, 2, 0 / 0}, {48000, -1, 4}}) do
  assert(not pcall(Audio.streamReader, descriptor(4096, f[1], f[2], f[3])), "format " .. f[2] .. "x" .. f[3])
end
-- ring too small for one chunk
assert(not pcall(Audio.streamReader, descriptor(16, 48000, 2, 256)))
-- seeks region too small
local d = descriptor(2 ^ 20, 48000, 1, 64)
d.seeks = Audio.Region.create(4)
assert(not pcall(Audio.streamReader, d))
-- a valid forged descriptor; feed it records that are not PCM
local good = descriptor(2 ^ 20, 48000, 1, 64)
local reader = Audio.streamReader(good)
local out = buffer.create(4096)
assert(reader:readInto(out) == 0)
-- a record larger than the reader's record buffer
assert(good.ring:send(string.rep("x", 8000)))
local ok, err = pcall(function() return reader:readInto(out) end)
assert(not ok, "an oversized record was read")
-- a short garbage record, and a record whose header claims a frame of NaN
assert(good.ring:send("abc"))
pcall(function() return reader:readInto(out) end)
assert(good.ring:send({0 / 0, math.huge, -1}))
pcall(function() return reader:readInto(out) end)
-- readInto into a zero-length and an odd-length buffer
pcall(function() return reader:readInto(buffer.create(0)) end)
pcall(function() return reader:readInto(buffer.create(3)) end)
-- bump the generation from script mid-stream
good.seeks:atomicAdd(0, 0xFFFFFFFF)
pcall(function() return reader:readInto(out) end)
-- many readers on a 16 MiB chunk (each reader holds 32 MiB on the host):
-- the budget must refuse before the readers hold more than the VM's 64 MiB
local big = descriptor(16 * 1024 * 1024 + 64, 48000, 1, 4 * 1024 * 1024 - 16)
local readers = {}
local refused = false
for i = 1, 8 do
  local okr, r = pcall(Audio.streamReader, big)
  if not okr then refused = true break end
  readers[i] = r
end
assert(refused and #readers <= 2, "made " .. #readers .. " readers of 32 MiB each")
return {}
