-- luau-vm-diff: flags-on
-- The integer type (LuauIntegerType2: `1i` literals). The integer library
-- is not opened in the Rive profile; these are the core operations.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local a, b, big = 7i, -3i, 9223372036854775807i
print(type(a), typeof(b), tostring(a), tostring(b), tostring(big), tostring(-9223372036854775807i), tostring(0i))
print(a == 7i, a == 7, a ~= b, rawequal(a, 7i), 9007199254740993i == 9007199254740992i, rawequal(9007199254740993i, 9007199254740992i))
local p, q = 9007199254740993i, 9007199254740992i
local eq = function(x, y) return x == y end
print(p == q, p ~= q, rawequal(p, q), eq(p, q), eq(p, p), ({[p] = 1})[q])
-- Integer keys hash by their bits but compare as doubles, as upstream.
local hits = {}
local keys = {11i, 23i, 37i, 41i, 53i, 67i, 79i, 83i}
for size = 0, 7 do
  local h = {}
  for i = 1, size do h[keys[i]] = i end
  h[p] = "p"
  hits[#hits + 1] = tostring(h[q]) .. "/" .. tostring(h[p])
end
print(table.concat(hits, " "))
local t = {}
t[a] = "seven"; t[7] = "number seven"; t[9007199254740992i] = "big"
print(t[7i], t[7], t[9007199254740993i], rawget(t, a))
print(all(pcall(function() return a + b end)), all(pcall(function() return -a end)), all(pcall(function() return a < b end)))
print(all(pcall(function() return a .. "x" end)), all(pcall(function() return #a end)), all(pcall(function() return a.x end)))
print(all(pcall(string.rep, "x", 2i)), all(pcall(math.abs, 3i)), tonumber(a), all(pcall(tostring, a)))
print(integer == nil, select("#", 1i, 2i))
return a
