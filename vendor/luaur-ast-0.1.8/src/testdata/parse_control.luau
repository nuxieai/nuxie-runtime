local function f(a, b, ...)
    if a < b then
        return a
    elseif a == b then
        return 0
    else
        return b
    end
end

local t = { 1, 2, name = "v", [x] = true }

while true do
    f(1, 2)
    break
end

repeat
    g = g + 1
until g > 10

for i = 1, 10, 2 do
    t[i] = i
    t.field = i
end

for k, v in t do
    print(k, v)
end

do
    local inner = f(1, 2)
    return inner
end
