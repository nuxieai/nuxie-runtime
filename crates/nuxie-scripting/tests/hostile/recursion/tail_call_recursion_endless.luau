--! attacks: endless recursion in tail position (Luau keeps a frame per tail call); expect: error(stack overflow); within: 15s
local function spin(n) return spin(n + 1) end
spin(1)
