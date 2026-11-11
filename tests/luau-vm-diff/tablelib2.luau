-- the rest of the table library: create, clear, clone, move, find, getn,
-- maxn, foreach, foreachi, freeze, isfrozen; unpack's lua_checkstack cap
local function show(t, n) local p = {} for i = 1, n or #t do p[i] = tostring(t[i]) end return table.concat(p, " ") end
local function keys(t) local k = {} for key in pairs(t) do k[#k + 1] = tostring(key) end table.sort(k) return table.concat(k, ",") end

-- create
print(#table.create(0), #table.create(5), show(table.create(4, "x")), #table.create(3, nil))
print(show(table.create("3", 7)), #table.create(2.9, 1))
print(pcall(table.create, -1))
print(pcall(table.create))
print(pcall(table.create, "no"))
print(pcall(table.create, 2^26 + 1))
local c = table.create(4)
c[5] = 5
print(#c, c[5])

-- clear
local t = {1, 2, 3, a = 1, b = 2}
table.clear(t)
print(#t, next(t), keys(t))
t[1], t.z = "one", "zed"
print(#t, t[1], keys(t))
print(pcall(table.clear))
print(pcall(table.clear, 1, 2))
print(pcall(table.clear, table.freeze({1})))

-- clone
local mt = {__index = function(_, k) return "mt:" .. tostring(k) end}
local src = setmetatable({1, 2, x = "y"}, mt)
local copy = table.clone(src)
print(copy ~= src, getmetatable(copy) == mt, copy[1], copy[2], copy.x, copy.missing)
copy[1] = 100
print(src[1], copy[1])
local frozen = table.freeze({4, 5})
local thaw = table.clone(frozen)
print(table.isfrozen(frozen), table.isfrozen(thaw))
thaw[3] = 6
print(show(thaw))
print(pcall(table.clone, setmetatable({}, {__metatable = "locked"})))
print(pcall(table.clone, "x"))
print(pcall(table.clone))

-- move
print(show(table.move({10, 20, 30}, 1, 3, 2)))
print(show(table.move({10, 20, 30}, 2, 3, 1)))
print(show(table.move({10, 20, 30}, 1, 3, 3), 5))
local dst = {}
print(table.move({1, 2, 3}, 1, 3, 1, dst) == dst, show(dst))
print(table.move({1, 2, 3}, 1, 0, 1, dst) == dst, show(dst))
local neg = table.move({[-2] = "a", [-1] = "b", [0] = "c", "d"}, -2, 1, 5)
print(neg[5], neg[6], neg[7], neg[8], #neg)
-- the sparse path: a range far larger than both tables
local sparse = {1, 2, 3, [50] = "fifty", [99] = "late"}
local moved = table.move(sparse, 1, 1000, 101, {})
print(moved[101], moved[103], moved[150], moved[199], moved[200], keys(moved))
local self = {1, 2, 3, [40] = 40}
table.move(self, 1, 200, 3)
print(keys(self), self[3], self[5], self[42])
local clearing = {[500] = "old", [600] = "keep"}
table.move({}, 1, 100, 450, clearing)
print(clearing[500], clearing[600])
-- A destination array part grown past MAXSIZE fails before anything
-- moves. (Huge sparse ranges are in limit_tests: the flags-off profile runs
-- the vendored VM's element loop, before LuauTableMoveTimeoutFix.)
print(pcall(table.move, {[1000] = 1}, 1, 2^30, 1, {}))
print(pcall(table.move, 1, 2, 3, 4))
print(pcall(table.move, {}, 2, 3, 4, "foo"))
print(pcall(table.move, {}, -1, 2^31 - 1, 1))
print(pcall(table.move, {}, 1, 2^31 - 1, 2))
print(pcall(table.move, {1}, 1, 1, 1, table.freeze({})))
print(pcall(table.move, {}, 1, 2))

-- find
local f = {"a", "b", "c", "b"}
print(table.find(f, "b"), table.find(f, "b", 3), table.find(f, "z"), table.find(f, "a", 5))
print(table.find({1, 2, nil, 4}, 4), table.find({}, nil))
local eqmt = {__eq = function(x, y) return x.v == y.v end}
local o1, o2 = setmetatable({v = 1}, eqmt), setmetatable({v = 1}, eqmt)
print(table.find({o1}, o2), table.find({1, o1}, o2))
print(pcall(table.find, f, "a", 0))
print(pcall(table.find, f))
print(pcall(table.find, "f", "a"))

-- getn, maxn
print(table.getn({1, 2, 3}), table.getn({}), table.getn({1, nil, 3}))
print(pcall(table.getn, 1))
print(table.maxn({}), table.maxn({1, 2, 3}), table.maxn({[10] = 1, [2.5] = 1, [-4] = 1}), table.maxn({1, nil, nil, x = 7}))
print(table.maxn({[1e10] = true, 1}), pcall(table.maxn))

-- foreach, foreachi
local seen = {}
print(table.foreach({a = 1, b = 2, 3}, function(k, v) seen[#seen + 1] = tostring(k) .. "=" .. tostring(v) end))
table.sort(seen)
print(table.concat(seen, ","))
print(table.foreach({x = 90}, function(k, v) return v end))
print(table.foreach({}, error))
print(table.foreachi({x = 10, y = 20}, error))
print(table.foreachi({"a", "b", "c"}, function(i, v) if v == "b" then return i, "ignored" end end))
local grown = {1, 2}
print(table.foreachi(grown, function(i, v) grown[#grown + 1] = v end), #grown)
print(pcall(table.foreach, {}, 1))
print(pcall(table.foreachi, {}))
print(pcall(table.foreach, {1}, function() error("inner") end))
local deleting = {a = 1, b = 2, c = 3}
local count = 0
table.foreach(deleting, function(k) deleting[k] = nil; count += 1 end)
print(count, next(deleting))

-- freeze, isfrozen
local fz = {1}
print(table.freeze(fz) == fz, table.isfrozen(fz), table.isfrozen({}))
print(pcall(table.freeze, fz))
print(pcall(table.freeze, setmetatable({}, {__metatable = false})))
print(pcall(table.isfrozen, nil))
print(pcall(function() fz[1] = 2 end))
print(pcall(rawset, fz, 1, 2))
print(pcall(table.insert, fz, 1))

-- unpack refuses more than LUAI_MAXCSTACK slots
print(pcall(function() return select("#", table.unpack(table.create(7999, 0))) end))
print(pcall(function() return select("#", table.unpack(table.create(8000, 0))) end))
print(select("#", pcall(table.unpack, table.create(7998, 1))))
print(pcall(table.unpack, table.create(7999, 1), 1, 7999, "extra"))
return table.getn(f)
