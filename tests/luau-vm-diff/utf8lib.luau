-- utf8 library: decoding limits, positions, error text.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local s = "h\u{E9}llo \u{10FFFF}\u{7FF}\u{800}"
print(utf8.len(s), utf8.len(s, 3), utf8.len(s, -4), all(utf8.len("ab\xFFc")), all(utf8.len("\xC0\x80")), all(utf8.len("\xED\xA0\x80")), all(utf8.len("\xF4\x90\x80\x80")))
print(all(utf8.codepoint(s, 1, -1)), all(utf8.codepoint(s, 2)), all(utf8.codepoint(s, 5, 4)))
print(utf8.char(), utf8.char(72, 0x10FFFF, 0xD800, 0, 0x7FF), #utf8.char(0x10000, 0x800, 0x80))
print(utf8.offset(s, 3), utf8.offset(s, -1), utf8.offset(s, 0, 3), utf8.offset(s, 20), utf8.offset(s, -20), utf8.offset("", 1), utf8.offset(s, 1, #s + 1))
local acc = {}
for p, c in utf8.codes(s) do acc[#acc + 1] = p .. ":" .. c end
print(table.concat(acc, " "), utf8.codes(s) ~= utf8.codes(s), #utf8.charpattern)
print(all(pcall(utf8.len, s, 0)), all(pcall(utf8.len, s, 1, 100)), all(pcall(utf8.codepoint, s, 0)), all(pcall(utf8.codepoint, s, 1, 100)), all(pcall(utf8.codepoint, "\xFF")))
print(all(pcall(utf8.char, -1)), all(pcall(utf8.char, 0x110000)), all(pcall(utf8.char, "x")), all(pcall(utf8.offset, s, 1, 0)), all(pcall(utf8.offset, s, 1, 3)))
print(all(pcall(function() for _ in utf8.codes("a\xFFb") do end end)), all(pcall(function() for _ in utf8.codes("a\xC3") do end end)), type(select(2, utf8.codes(1))), select("#", utf8.codes(1)))
print(utf8.len(12345), all(pcall(utf8.len)), utf8.char(65.7), all(pcall(utf8.codepoint, string.rep("a", 9000), 1, -1)))
