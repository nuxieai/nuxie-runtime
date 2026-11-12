//! The abort-profile probe on the safe VM (docs/luau-safe-vm-plan.md, goal 2).
//!
//! A port of `tools/luaur-rt-test-harness/examples/protected_error_recovery.rs`
//! onto `nuxie_scripting::luau` on the safe VM (the default `luau`). Run it normally and with
//! `-Cpanic=abort`; it must exit 0 in both. Stable Rust's test harness needs
//! unwinding, so only an executable shows containment in an aborting build.
//!
//! Left out of the port, because the facade has no such surface: the raw C
//! API helpers (`exec_raw`, light userdata, `luaL_getmetatable`), scoped
//! userdata, `Thread::resume_error`, `Table::equals` (the `__eq` failure is
//! reached through a chunk instead) and the async functions.
use nuxie_scripting::luau::{Error, Lua, Result, Table};
use std::cell::Cell;
use std::rc::Rc;

fn main() -> Result<()> {
    let lua = Lua::new();
    lua.globals().set(
        "host_error",
        lua.create_function(|_, ()| -> Result<()> { Err(Error::runtime("host callback failed")) })?,
    )?;

    assert_eq!(
        lua.load("return 6 * 7").into_function()?.call::<i32>(())?,
        42
    );
    println!("initial execution: passed");

    lua.set_interrupt(|_| Err(Error::runtime("interrupt requested stop")));
    let interrupted = lua.load("local n = 0; while true do n += 1 end").exec();
    lua.remove_interrupt();
    assert!(
        interrupted
            .unwrap_err()
            .to_string()
            .contains("interrupt requested stop")
    );
    assert_eq!(lua.load("return 42").into_function()?.call::<i32>(())?, 42);
    println!("interrupt failure and recovery: passed");

    let captured = Rc::new(Cell::new(0));
    lua.scope(|scope| {
        let counter = Rc::clone(&captured);
        let callback = scope.create_function(move |_, ()| {
            counter.set(counter.get() + 1);
            Ok(())
        })?;
        callback.call::<()>(())?;
        lua.globals().set("expired_callback", callback)?;
        Ok(())
    })?;
    assert_eq!(captured.get(), 1);
    assert!(lua.load("expired_callback()").exec().is_err());
    assert_eq!(lua.load("return 42").into_function()?.call::<i32>(())?, 42);
    println!("scoped callback invalidation and recovery: passed");

    let function = lua.load("return answer").into_function()?;
    let environment = lua.create_table_result()?;
    environment.set("answer", 42)?;
    for _ in 0..3 {
        assert!(function.set_environment(environment.clone())?);
        let current = function
            .try_environment()?
            .ok_or_else(|| Error::runtime("environment missing after set_environment"))?;
        assert_eq!(current.get::<i32>("answer")?, 42);
        assert_eq!(function.call::<i32>(())?, 42);
    }
    println!("repeated environment inspection and replacement: passed");

    for attempt in 0..3 {
        let guarded_table: Table = lua
            .load(
                r#"
            return setmetatable({ value = 42, fail = function() error("field call failed") end }, {
                __index = function() error("field lookup failed") end,
                __newindex = function() error("field write failed") end,
            })
        "#,
            )
            .into_function()?
            .call(())?;
        assert!(
            guarded_table
                .get::<i32>("missing")
                .unwrap_err()
                .to_string()
                .contains("field lookup failed")
        );
        assert!(
            guarded_table
                .set("missing", 7)
                .unwrap_err()
                .to_string()
                .contains("field write failed")
        );
        assert!(
            guarded_table
                .call_function_unit("fail", ())
                .unwrap_err()
                .to_string()
                .contains("field call failed")
        );
        assert_eq!(guarded_table.get::<i32>("value")?, 42);
        let equality = lua
            .load(
                r#"
            local mt = { __eq = function() error("equality failed") end }
            return setmetatable({}, mt) == setmetatable({}, mt)
        "#,
            )
            .into_function()
            .and_then(|function| function.call::<bool>(()));
        assert!(
            equality
                .unwrap_err()
                .to_string()
                .contains("equality failed")
        );
        assert_eq!(lua.load("return 42").into_function()?.call::<i32>(())?, 42);

        let coroutine = lua.create_thread(
            lua.load("coroutine.yield(7); error('resumed coroutine failed')")
                .into_function()?,
        )?;
        assert_eq!(coroutine.resume::<i32>(())?, 7);
        assert!(
            coroutine
                .resume::<()>(())
                .unwrap_err()
                .to_string()
                .contains("resumed coroutine failed")
        );
        assert!(coroutine.resume::<()>(()).is_err());
        let fresh_coroutine = lua.create_thread(lua.load("return 42").into_function()?)?;
        assert_eq!(fresh_coroutine.resume::<i32>(())?, 42);

        lua.load(
            r#"
            local marker = {}
            local ok, err = pcall(function() error(marker) end)
            assert(not ok and err == marker)

            local outerOk, innerOk, innerError = pcall(function()
                return pcall(host_error)
            end)
            assert(outerOk and not innerOk)
            assert(string.find(tostring(innerError), "host callback failed", 1, true))

            local handled, message = xpcall(function() error("original") end,
                function(err) return "handled:" .. tostring(err) end)
            assert(not handled and string.find(message, "handled:", 1, true))
            local doubleFailure = xpcall(function() error("original") end,
                function() error("handler failed") end)
            assert(not doubleFailure)

            assert(not pcall(function() return nil + 1 end))
            assert(not pcall(function() local missing = nil; return missing.value end))

            -- Library failures must return before using invalid arguments or
            -- modifying memory, and leave the same VM usable afterwards.
            assert(not pcall(math.abs, {}))
            assert(not pcall(math.clamp, 1, 4, 2))
            assert(math.clamp(3, 1, 5) == 3)
            assert(not pcall(vector.create, "invalid", 0, 0))
            local direction = vector.create(1, 2, 3)
            assert(direction.X == 1 and direction.Y == 2 and direction.Z == 3)

            local bytes = buffer.fromstring("safe")
            assert(not pcall(buffer.writeu8, bytes, 4, 255))
            assert(not pcall(buffer.fill, bytes, -1, 0, 2))
            assert(not pcall(buffer.readu32, bytes, 1))
            assert(not pcall(buffer.len, {}))
            assert(buffer.tostring(bytes) == "safe")
            buffer.writeu8(bytes, 0, 83)
            assert(buffer.tostring(bytes) == "Safe")

            assert(not pcall(bit32.extract, 255, 31, 2))
            assert(not pcall(bit32.band, 255, {}))
            assert(bit32.extract(255, 4, 4) == 15)

            assert(not pcall(string.sub, {}, 1))
            assert(not pcall(string.char, 256))
            assert(string.sub("recovered", 1, 7) == "recover")
            assert(string.reverse("abc") == "cba")
            assert(string.rep("ab", 400) == string.rep("abab", 200))
            assert(not pcall(string.match, "abc", "["))
            assert(not pcall(string.gsub, "abc", "(a)", "%2"))
            assert(not pcall(string.format, "%d", {}))
            assert(string.gsub("abc", "(a)", "%1%1") == "aabc")
            assert(string.format("%02d:%02d", 2, 5) == "02:05")
            assert(not pcall(string.pack, "i17", 1))
            assert(not pcall(string.unpack, "i4", "x"))
            assert(string.unpack("<i4", string.pack("<i4", 42)) == 42)
            assert(not pcall(utf8.char, 0x110000))
            assert(not pcall(utf8.codepoint, string.char(255)))
            assert(not pcall(utf8.offset, "é", 1, 2))
            assert(utf8.len("héllo") == 5)
            assert(utf8.codepoint("é") == 233)

            local intercepted = setmetatable({}, {
                __index = function() error(marker) end,
                __newindex = function() error(marker) end,
            })
            local readOk, readError = pcall(function() return intercepted.value end)
            assert(not readOk and readError == marker)
            local writeOk, writeError = pcall(function() intercepted.value = 42 end)
            assert(not writeOk and writeError == marker)
            assert(rawget(intercepted, "value") == nil)
            rawset(intercepted, "value", 42)
            assert(intercepted.value == 42)

            local badConcat = setmetatable({}, {
                __concat = function() error(marker) end,
            })
            local concatOk, concatError = pcall(function() return badConcat .. "suffix" end)
            assert(not concatOk and concatError == marker)
            assert(not pcall(function() return {} .. "suffix" end))
            assert("prefix" .. 42 .. "suffix" == "prefix42suffix")

            local frozen = table.freeze({3, 1, 2})
            assert(not pcall(table.clear, frozen))
            assert(not pcall(table.move, frozen, 1, 2, 2))
            assert(frozen[1] == 3 and frozen[2] == 1 and frozen[3] == 2)
            assert(not pcall(table.concat, {"a", {}}))
            assert(not pcall(table.create, -1))
            assert(not pcall(table.create, 1e9))
            assert(not pcall(table.find, {}, "x", 0))
            assert(not pcall(table.sort, {3, 1, 2}, function() error(marker) end))
            -- An inconsistent comparator may or may not raise; it must not abort.
            pcall(table.sort, {3, 1, 2, 5, 4}, function() return true end)
            local ordered = {3, 1, 2}
            table.sort(ordered, function(a, b) return a < b end)
            assert(table.concat(ordered, ",") == "1,2,3")

            local function deep(n) return deep(n + 1) + 1 end
            assert(not pcall(deep, 0))

            local worker = coroutine.create(function()
                coroutine.yield("ready")
                error("coroutine failed")
            end)
            local resumed, value = coroutine.resume(worker)
            assert(resumed and value == "ready")
            local resumedAgain, failure = coroutine.resume(worker)
            assert(not resumedAgain and string.find(tostring(failure), "coroutine failed", 1, true))

            local wrapped = coroutine.wrap(function()
                coroutine.yield("wrapped ready")
                error(marker)
            end)
            assert(wrapped() == "wrapped ready")
            local wrappedOk, wrappedError = pcall(wrapped)
            assert(not wrappedOk and wrappedError == marker)
            assert(not pcall(coroutine.resume, {}))
            "#,
        )
        .exec()?;
        assert!(
            lua.load("error('unprotected guest failure')")
                .exec()
                .is_err()
        );
        assert!(lua.load("local = malformed").exec().is_err());
        assert_eq!(
            lua.load("return 6 * 7").into_function()?.call::<i32>(())?,
            42
        );
        println!("protected errors and same-VM recovery #{attempt}: passed");
    }

    lua.set_memory_limit(4 * 1024 * 1024)?;
    let exhausted = lua
        .load("local parts = {} for i = 1, 1e7 do parts[i] = string.rep('x', 64) .. i end")
        .exec();
    assert!(exhausted.is_err());
    lua.set_memory_limit(0)?;
    lua.gc_collect()?;
    assert_eq!(
        lua.load("return 6 * 7").into_function()?.call::<i32>(())?,
        42
    );
    println!("memory limit failure and recovery: passed");
    Ok(())
}
