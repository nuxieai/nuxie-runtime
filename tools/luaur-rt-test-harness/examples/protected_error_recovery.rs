//! Run normally and with CARGO_PROFILE_DEV_PANIC=abort. Guest errors must be
//! recoverable in both builds; success under Rust unwinding is insufficient.
use luaur_rt::{Error, Lua, Result, UserData, UserDataMethods};

struct ScopedCounter<'a>(&'a std::cell::Cell<i32>, std::rc::Rc<()>);

impl UserData for ScopedCounter<'_> {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("increment", |_, data, ()| {
            data.0.set(data.0.get() + 1);
            let _keep_alive = &data.1;
            Ok(())
        });
    }
}

fn main() -> Result<()> {
    let lua = Lua::new();
    #[cfg(feature = "async")]
    verify_async_recovery(&lua)?;
    lua.globals().set(
        "host_error",
        lua.create_function(|_, ()| -> Result<()> { Err(Error::runtime("host callback failed")) })?,
    )?;

    assert_eq!(lua.load("return 6 * 7").eval::<i32>()?, 42);
    println!("initial execution: passed");

    let marker = 42_i32;
    let pointer = (&marker as *const i32).cast_mut().cast();
    let value: luaur_rt::LightUserData = unsafe {
        lua.exec_raw((), |state| {
            luaur_vm::macros::lua_pushlightuserdata::lua_pushlightuserdata(state, pointer)
        })?
    };
    assert_eq!(value.0, pointer);
    let missing: luaur_rt::Value = unsafe {
        lua.exec_raw((), |state| {
            luaur_vm::macros::lua_l_getmetatable::luaL_getmetatable(
                state,
                c"missing-recovery-metatable".as_ptr(),
            )?;
            Ok(())
        })?
    };
    assert!(matches!(missing, luaur_rt::Value::Nil));
    println!("typed raw API helpers: passed");

    lua.set_interrupt(|_| Err(Error::runtime("interrupt requested stop")));
    let interrupted = lua.load("local n = 0; while true do n += 1 end").exec();
    lua.remove_interrupt();
    assert!(
        interrupted
            .unwrap_err()
            .to_string()
            .contains("interrupt requested stop")
    );
    assert_eq!(lua.load("return 42").eval::<i32>()?, 42);
    println!("interrupt failure and recovery: passed");

    let mut captured = 0;
    lua.scope(|scope| {
        let callback = scope.create_function_mut(|_, ()| {
            captured += 1;
            Ok(())
        })?;
        callback.call::<()>(())?;
        lua.globals().set("expired_callback", callback)?;
        Ok(())
    })?;
    assert_eq!(captured, 1);
    assert!(lua.load("expired_callback()").exec().is_err());
    assert_eq!(lua.load("return 42").eval::<i32>()?, 42);
    println!("scoped callback invalidation and recovery: passed");

    let count = std::cell::Cell::new(0);
    let lifetime = std::rc::Rc::new(());
    for fail_scope in [false, true] {
        let result: Result<()> = lua.scope(|scope| {
            let userdata = scope.create_userdata(ScopedCounter(&count, lifetime.clone()))?;
            lua.globals().set("expired_userdata", userdata)?;
            lua.load("expired_userdata:increment()").exec()?;
            if fail_scope {
                Err(Error::runtime("leave scope early"))
            } else {
                Ok(())
            }
        });
        assert_eq!(result.is_err(), fail_scope);
        assert_eq!(std::rc::Rc::strong_count(&lifetime), 1);
        assert!(lua.load("expired_userdata:increment()").exec().is_err());
        assert_eq!(lua.load("return 42").eval::<i32>()?, 42);
    }
    assert_eq!(count.get(), 2);
    println!("scoped userdata cleanup on success and error: passed");

    let function = lua.load("return answer").into_function()?;
    let environment = lua.create_table_result()?;
    environment.set("answer", 42)?;
    for _ in 0..3 {
        assert!(function.set_environment(environment.clone())?);
        assert_eq!(
            function.try_environment()?.unwrap().get::<i32>("answer")?,
            42
        );
        assert_eq!(function.call::<i32>(())?, 42);
    }
    println!("repeated environment inspection and replacement: passed");

    for attempt in 0..3 {
        // Raw embedding callbacks must return their failure to the protected
        // boundary too, and must not leave temporary values on the stack.
        let raw_error = unsafe {
            lua.exec_raw::<(), _>((), |state| {
                luaur_vm::functions::lua_pushlstring::lua_pushlstring(
                    state,
                    c"raw callback failed".as_ptr(),
                    19,
                )?;
                luaur_rt::ffi::lua_error(state)
            })
        };
        assert!(
            raw_error
                .unwrap_err()
                .to_string()
                .contains("raw callback failed")
        );
        let raw_value: i32 =
            unsafe { lua.exec_raw((), |state| luaur_rt::ffi::lua_pushnumber(state, 42.0))? };
        assert_eq!(raw_value, 42);

        let guarded_table: luaur_rt::Table = lua
            .load(
                r#"
            return setmetatable({ value = 42, fail = function() error("field call failed") end }, {
                __index = function() error("field lookup failed") end,
                __newindex = function() error("field write failed") end,
            })
        "#,
            )
            .eval()?;
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
        let (left, right): (luaur_rt::Table, luaur_rt::Table) = lua
            .load(
                r#"
            local mt = { __eq = function() error("equality failed") end }
            return setmetatable({}, mt), setmetatable({}, mt)
        "#,
            )
            .eval()?;
        assert!(
            left.equals(&right)
                .unwrap_err()
                .to_string()
                .contains("equality failed")
        );
        assert_eq!(lua.load("return 42").eval::<i32>()?, 42);

        let coroutine =
            lua.create_thread(lua.load("coroutine.yield(7); return 42").into_function()?)?;
        assert_eq!(coroutine.resume::<i32>(())?, 7);
        assert!(
            coroutine
                .resume_error::<()>("resume callback failed")
                .unwrap_err()
                .to_string()
                .contains("resume callback failed")
        );
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
            assert(not pcall(table.find, {}, "x", 0))
            assert(not pcall(table.sort, {3, 1, 2}, function() error(marker) end))
            local ordered = {3, 1, 2}
            table.sort(ordered, function(a, b) return a < b end)
            assert(table.concat(ordered, ",") == "1,2,3")

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
        assert_eq!(lua.load("return 6 * 7").eval::<i32>()?, 42);
        println!("protected errors and same-VM recovery #{attempt}: passed");
    }
    Ok(())
}

#[cfg(feature = "async")]
fn verify_async_recovery(lua: &Lua) -> Result<()> {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    // These futures wake themselves once; no timer, network, or executor
    // dependency is needed to exercise the actual coroutine/poller bridge.
    fn drive<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
        let mut future = std::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        for _ in 0..16 {
            if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
                return result;
            }
        }
        panic!("self-waking recovery fixture did not complete");
    }

    let callback = lua.create_async_function(|_, fail: bool| async move {
        let mut yielded = false;
        std::future::poll_fn(move |context| {
            if yielded {
                Poll::Ready(())
            } else {
                yielded = true;
                context.waker().wake_by_ref();
                Poll::Pending
            }
        })
        .await;
        if fail {
            Err(Error::runtime("async callback failed"))
        } else {
            Ok((40, 1, 1))
        }
    })?;
    for _ in 0..3 {
        assert!(
            drive(callback.call_async::<()>(true))
                .unwrap_err()
                .to_string()
                .contains("async callback failed")
        );
        assert_eq!(
            drive(callback.call_async::<(i32, i32, i32)>(false))?,
            (40, 1, 1)
        );
        assert_eq!(lua.load("return 42").eval::<i32>()?, 42);
    }
    println!("suspended async failure and same-VM recovery: passed");
    Ok(())
}
