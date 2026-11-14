--! attacks: Audio.play* and clock functions with non-source and hostile arguments; expect: ok; within: 10s
for _, name in ipairs({"play", "playAtTime", "playInTime", "playAtFrame", "playInFrame"}) do
  local f = Audio[name]
  -- table.pack keeps the nil that ipairs would stop at.
  local args1 = table.pack(nil, 1, "x", {}, Audio.Ring.create(8), Audio.Blob.from("x"))
  for i1 = 1, args1.n do local src = args1[i1]
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
