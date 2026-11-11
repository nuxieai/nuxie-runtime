-- string.format's `%*` calls `__tostring` (a continuation in nuxie-luau-vm,
-- `lua_call` from inside the C function in the vendored VM), and a host
-- function as an `xpcall` handler.

local T = setmetatable({}, { __tostring = function() return "T!" end })
print(string.format("[%*] [%*] [%*]", T, 1.5, nil))
print(string.format("%s|%*|%d", "a", T, 7))

-- `__tostring` that formats again, several levels deep.
local function nest(n)
	return setmetatable({}, {
		__tostring = function()
			if n == 0 then
				return "leaf"
			end
			return string.format("(%*)", nest(n - 1))
		end,
	})
end
print(string.format("%*", nest(10)))

-- Results that are not strings, errors inside, and a traceback taken
-- inside the metamethod.
print(pcall(string.format, "%*", setmetatable({}, { __tostring = function() return 1 end })))
print(pcall(string.format, "%*", setmetatable({}, { __tostring = function() return {} end })))
print(pcall(string.format, "x%*y", setmetatable({}, { __tostring = function() error("inner") end })))
print(select("#", pcall(string.format, "x%*y", setmetatable({}, { __tostring = function() error({}) end }))))
print(string.format("%*", setmetatable({}, {
	__tostring = function()
		return (debug.traceback("tb", 1):gsub("\n[^\n]*$", ""))
	end,
})))
print(string.format("%*", setmetatable({}, {
	__tostring = function()
		return tostring(debug.info(2, "n")) .. "/" .. tostring(debug.info(2, "s"))
	end,
})))

-- Unbounded recursion through `%*`.
local loop = {}
setmetatable(loop, { __tostring = function() return string.format("%*", loop) end })
local ok, err = pcall(string.format, "%*", loop)
print(ok, err)

-- A yield inside `__tostring` under `%*` crosses a C call.
local co = coroutine.create(function()
	return string.format("%*", setmetatable({}, {
		__tostring = function()
			coroutine.yield()
			return "after"
		end,
	}))
end)
print(coroutine.resume(co))

-- The format state survives the call: arguments after `%*`.
print(string.format("%*-%d-%*-%s", T, 3, T, "end"))

-- A host function (`print`) as the `xpcall` handler.
print(xpcall(function() error("boom") end, print))
print(xpcall(function() error("again", 0) end, print))
