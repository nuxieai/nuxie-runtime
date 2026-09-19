//! Run normally and with CARGO_PROFILE_DEV_PANIC=abort. Guest errors must be
//! recoverable in both builds; success under Rust unwinding is insufficient.
use luaur_rt::{Error, Lua, Result};

fn main() -> Result<()> {
    let lua = Lua::new();
    lua.globals().set(
        "host_error",
        lua.create_function(|_, ()| -> Result<()> { Err(Error::runtime("host callback failed")) })?,
    )?;

    assert_eq!(lua.load("return 6 * 7").eval::<i32>()?, 42);
    println!("initial execution: passed");

    for attempt in 0..3 {
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

            local worker = coroutine.create(function()
                coroutine.yield("ready")
                error("coroutine failed")
            end)
            local resumed, value = coroutine.resume(worker)
            assert(resumed and value == "ready")
            local resumedAgain, failure = coroutine.resume(worker)
            assert(not resumedAgain and string.find(tostring(failure), "coroutine failed", 1, true))
            "#,
        )
        .exec()?;
        assert!(lua
            .load("error('unprotected guest failure')")
            .exec()
            .is_err());
        assert!(lua.load("local = malformed").exec().is_err());
        assert_eq!(lua.load("return 6 * 7").eval::<i32>()?, 42);
        println!("protected errors and same-VM recovery #{attempt}: passed");
    }
    Ok(())
}
