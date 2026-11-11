-- weak tables (__mode): what a full collection clears, on both VMs
local function count(t)
    local n = 0
    for _ in pairs(t) do n += 1 end
    return n
end

-- weak keys: object keys go, strings, numbers and booleans stay
local wk = setmetatable({}, {__mode = "k"})
local keep = {}
for i = 1, 20 do wk[{}] = i end
for i = 1, 10 do local t = {}; keep[i] = t; wk[t] = i end
for i = 1, 5 do wk[i] = i end
for i = 1, 5 do wk[string.rep("k", i)] = i end
wk[true] = "yes"
collectgarbage()
print("k", count(wk), wk[keep[3]], wk[2], wk["kkk"], wk[true])

-- weak values: object values go from both parts, string values stay
local wv = setmetatable({}, {__mode = "v"})
for i = 1, 8 do wv[i] = {} end
for i = 1, 8 do wv["x" .. i] = {} end
wv[9] = string.rep("s", 30)
wv.kept = keep[1]
wv.f = function() end
local held = function() return 1 end
wv.held = held
collectgarbage()
print("v", count(wv), wv[9] == string.rep("s", 30), wv.kept == keep[1], type(wv.f), wv.held == held)

-- both: an entry goes when either side is collectable and unreachable
local wkv = setmetatable({}, {__mode = "kv"})
local x = {}
wkv[1] = x
wkv[x] = 1
wkv[{}] = x
wkv[x] = {}
wkv.s = "string"
collectgarbage()
print("kv", count(wkv), wkv[1] == x, wkv.s)
x = nil
collectgarbage()
print("kv after", count(wkv), wkv.s)

-- weak keys keep their values strong (no ephemerons in Luau): a value
-- that refers to its own key keeps the entry
local eph = setmetatable({}, {__mode = "k"})
do
    local k = {}
    eph[k] = {k}
end
collectgarbage()
print("ephemeron", count(eph))

-- the mode is read when the table is traversed: clearing __mode makes it strong
local mt = {__mode = "v"}
local changed = setmetatable({}, mt)
changed[1] = {}
mt.__mode = nil
collectgarbage()
print("mode removed", changed[1] ~= nil)
mt.__mode = "v"
collectgarbage()
print("mode restored", changed[1])

-- a non-string mode is ignored; "s" alone is not weak
local odd = setmetatable({}, {__mode = 1})
odd[1] = {}
local shrink = setmetatable({}, {__mode = "s"})
shrink[1] = {}
collectgarbage()
print("odd", odd[1] ~= nil, shrink[1] ~= nil)

-- a weak table referring to itself, and weak tables nested in weak tables
local self_ref = setmetatable({}, {__mode = "kv"})
self_ref[self_ref] = self_ref
local outer = setmetatable({}, {__mode = "v"})
outer[1] = setmetatable({{}}, {__mode = "v"})
collectgarbage()
print("self", self_ref[self_ref] == self_ref, outer[1])

-- clearing during iteration: next() keeps working across a collection
local iter = setmetatable({}, {__mode = "v"})
local strong = {}
for i = 1, 30 do
    local t = {}
    if i % 3 == 0 then strong[#strong + 1] = t end
    iter[i] = t
end
local seen = 0
for k, v in pairs(iter) do
    seen += 1
    if seen == 2 then collectgarbage() end
end
print("iterated", seen >= 10, count(iter))

-- closures as weak values and keys
local wf = setmetatable({}, {__mode = "k"})
local function make(n) return function() return n end end
local live = make(1)
wf[live] = "live"
wf[make(2)] = "dead"
collectgarbage()
print("functions", count(wf), wf[live])
return count(wk), count(wv)
