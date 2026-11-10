-- FORNPREP FORNLOOP FORGPREP FORGLOOP FORGPREP_INEXT FORGPREP_NEXT JUMPBACK
local s = 0
for i = 1, 10 do s = s + i end
print(s)
s = 0
for i = 10, 1, -2 do s = s + i end
print(s)
for i = 1, 0 do print("never") end
for i = 0.5, 2, 0.5 do io_line = i end
print(io_line)
local steps = {}
for i = 1, 3, 0.75 do steps[#steps + 1] = i end
print(table.concat(steps, " "))
local ok, err = pcall(function() for i = 1, 10, 0 do end end)
print(ok, err)
ok, err = pcall(function() for i = "a", 2 do end end)
print(ok, err)
ok, err = pcall(function() for i = 1, {} do end end)
print(ok, err)
ok, err = pcall(function() for i = 1, 2, "x" do end end)
print(ok, err)

local function sort(list)
    for i = 2, #list do
        local v, j = list[i], i - 1
        while j >= 1 and list[j] > v do list[j + 1] = list[j]; j = j - 1 end
        list[j + 1] = v
    end
end
local arr = {"a", "b", "c", nil, "e"}
local seen = {}
for i, v in ipairs(arr) do seen[#seen + 1] = i .. "=" .. v end
print(table.concat(seen, ","))
local keys = {}
for k, v in pairs({x = 1, y = 2, z = 3, 10, 20}) do keys[#keys + 1] = tostring(k) .. ":" .. tostring(v) end
sort(keys)
print(table.concat(keys, ","))
local nk = {}
for k in next, {p = 1, q = 2} do nk[#nk + 1] = k end
sort(nk)
print(table.concat(nk, ","))

-- generic iteration over a table without a call (Luau's built-in iteration)
local acc = {}
for k, v in {5, 6, 7} do acc[#acc + 1] = k * v end
print(table.concat(acc, ","))

-- a stateful iterator function
local function range(n)
    local i = 0
    return function()
        i = i + 1
        if i <= n then return i, i * i end
    end
end
local sq = {}
for i, v in range(4) do sq[#sq + 1] = v end
print(table.concat(sq, ","))

-- __iter
local obj = setmetatable({}, {__iter = function(t)
    local i = 0
    return function() i = i + 1; if i <= 3 then return i, "v" .. i end end
end})
local it = {}
for k, v in obj do it[#it + 1] = v end
print(table.concat(it, ","))

-- while / repeat / break / continue
local n = 0
while true do n = n + 1; if n >= 5 then break end end
local r = 0
repeat local q = r; r = r + 1 until q >= 3
local odd = {}
for i = 1, 9 do
    if i % 2 == 0 then continue end
    odd[#odd + 1] = i
end
print(n, r, table.concat(odd, ","))
ok, err = pcall(function() for k in 5 do end end)
print(ok, err)
return s
