--! attacks: xpcall handler guards and other host frames (sort, gsub, pcall, print, Gradient.linear) captured with debug.info and called after their call returned; expect: ok; within: 10s
local kept = {}
local function grab(tag)
  for level = 1, 6 do
    local f = debug.info(level, "f")
    -- Host frames only: a kept script frame (the chunk itself) would just re-run script code.
    if f and debug.info(f, "s") == "[C]" then table.insert(kept, { tag = tag .. level, f = f }) end
  end
end
xpcall(function() error("first", 0) end, function(e) grab("xpcall"); return e end)
table.sort({ 3, 1, 2 }, function(a, b) grab("sort"); return a < b end)
string.gsub("aa", "a", function() grab("gsub"); return "b" end)
pcall(function() grab("pcall") end)
print(setmetatable({}, { __tostring = function() grab("print"); return "p" end }))
Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), { setmetatable({}, { __index = function(_, k) grab("gradient"); return k == "position" and 0 or 0xff000000 end }) })
coroutine.wrap(function() grab("wrap") end)()
local co = coroutine.create(function() grab("resume") end)
coroutine.resume(co)
assert(#kept > 8, #kept)
-- Call every kept function outside its call, with junk arguments, inside pcall.
local junk = { false, 1, "x", {}, Path.new(), function() end }
local outsideXpcall = 0
for _, entry in ipairs(kept) do
  for _, arg in ipairs(junk) do
    local ok, err = pcall(entry.f, arg, arg)
    if not ok and string.find(tostring(err), "outside its xpcall", 1, true) then outsideXpcall += 1 end
  end
end
assert(outsideXpcall > 0, "a kept xpcall guard must raise the ordinary outside-its-xpcall error")
-- Call them inside an unrelated xpcall, from a coroutine, and from a sort comparator.
xpcall(function()
  for _, entry in ipairs(kept) do pcall(entry.f, "probe") end
  error("later", 0)
end, function(e) for _, entry in ipairs(kept) do pcall(entry.f, e) end; return e end)
coroutine.wrap(function() for _, entry in ipairs(kept) do pcall(entry.f, "co") end end)()
-- Keep them reachable at return so the VM drop sees them.
return { kept = kept }
