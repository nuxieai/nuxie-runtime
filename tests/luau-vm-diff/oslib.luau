-- os library over fixed timestamps and UTC formats only: local time is
-- UTC in this VM and the host's zone in the oracle.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local stamps = {0, 1, -1, 59, 86399, 951782400, 960595200, 1709214356, 32535215999, -62135596800, 1e15, -1e15}
for _, t in stamps do
  print(t, os.date("!%a %A %b %B %c|%d %H %I %j %m %M %p %S %U %w %W %x %X %y %Y %%", t))
  local d = os.date("!*t", t)
  if d then
    print(d.year, d.month, d.day, d.hour, d.min, d.sec, d.wday, d.yday, d.isdst, os.time(d))
  else
    print("nil date")
  end
end
print(os.date("", -1), os.date("!", 5), os.date("!%", 0), os.date("!%%%", 0), os.date("!a\0%Y", 0), os.date("!x%Yz", 0.9), os.date("!%Y", "86400"))
print(all(pcall(os.date, "!%9", 0)), all(pcall(os.date, "!%E", 0)), all(pcall(os.date, "!%Ea", 0)), all(pcall(os.date, {}, 0)), all(pcall(os.date, "!%Y", "x")))
print(os.time({year = 1969, month = 12, day = 31, hour = 23, min = 59, sec = 59}), os.time({year = 1970, month = 1, day = 1, hour = 0}), os.time({year = 2000, month = 60, day = 10}), os.time({year = "2000", month = 1.9, day = -100, min = 1e12}))
print(os.time({year = 2100, month = -600, day = 10, hour = 0}), os.time({year = 2^40, month = 1, day = 1}), os.time({year = 2000, month = 2^31, day = 1}), os.time({year = 2000, month = 1, day = 1, isdst = true}))
print(all(pcall(os.time, {hour = 12})), all(pcall(os.time, {year = 2000, day = 1})), all(pcall(os.time, {year = 2000, month = 1})), all(pcall(os.time, 5)), all(pcall(os.time, {year = 2000, month = 1, day = "x"})))
print(os.difftime(10, 3), os.difftime(10), os.difftime("5", "1"), all(pcall(os.difftime)), all(pcall(os.difftime, 1, {})))
print(type(os.clock()), os.clock() >= 0, type(os.time()), os.time() > 1.7e9, math.abs(os.time() - os.time(os.date("!*t"))) <= 1)
