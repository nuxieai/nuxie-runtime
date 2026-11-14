--! attacks: Audio.play* and clock functions with non-source and hostile arguments; expect: ok; within: 10s
for _, name in ipairs({"play", "playAtTime", "playInTime", "playAtFrame", "playInFrame"}) do
  local f = Audio[name]
  for _, src in ipairs({nil, 1, "x", {}, Audio.Ring.create(8), Audio.Blob.from("x")}) do
    for _, t in ipairs({0 / 0, math.huge, -math.huge, -1, 2 ^ 63}) do
      local ok = pcall(f, src, t)
    end
  end
end
pcall(Audio.time)
pcall(Audio.timeFrame)
pcall(Audio.sampleRate)
pcall(Audio.processorSampleRate)
pcall(Audio.processorChannels)
return {}
