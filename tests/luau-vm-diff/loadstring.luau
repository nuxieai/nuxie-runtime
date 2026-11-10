-- loadstring; LOADKX (more than 32767 constants); JUMPX (a jump over more than 32767 instructions)
local f = loadstring("return 1 + 1")
print(f())
local g = loadstring("local a, b = ... return a * b", "named")
print(g(6, 7))
print(loadstring("return +"))
print(loadstring("x ="))
local h = loadstring("error('inside')", "chunkname")
print(pcall(h))

local items = {}
for i = 1, 33000 do items[i] = tostring(i + 0.5) end
local many = loadstring("return {" .. table.concat(items, ",") .. "}")
local t = many()
print(#t, t[1], t[32768], t[33000])

local body = {}
for i = 1, 33000 do body[i] = "x = x + 1" end
local long = loadstring("local x = 0 local flag = ... if flag then " .. table.concat(body, " ") .. " end return x")
print(long(true), long(false))
return "done"
