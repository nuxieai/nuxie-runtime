--! attacks: an endless repeat-until-false loop doing table work each iteration; expect: error(exceeded); within: 15s
local t = {}
local i = 0
repeat i = i + 1 t[(i % 64) + 1] = i until false
