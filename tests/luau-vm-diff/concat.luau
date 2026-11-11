-- CONCAT
local a, b, c = "x", 1, 2.5
print(a .. b .. c, a .. "-" .. a, 1 .. 2, -0.0 .. "", 1e100 .. "")
local parts = ""
for i = 1, 5 do parts = parts .. i .. ";" end
print(parts)
local mt = {__concat = function(l, r)
    local ls = type(l) == "table" and "T" or tostring(l)
    local rs = type(r) == "table" and "T" or tostring(r)
    return ls .. "+" .. rs
end}
local t = setmetatable({}, mt)
print(t .. "s", "s" .. t, 1 .. t, t .. t, "a" .. "b" .. t .. "c")
local ok, err = pcall(function() return "a" .. {} end)
print(ok, err)
ok, err = pcall(function() return nil .. "a" end)
print(ok, err)
ok, err = pcall(function() local x = true; return "v" .. x end)
print(ok, err)
local long = string.rep("ab", 1000)
print(#(long .. long))
return a .. b
