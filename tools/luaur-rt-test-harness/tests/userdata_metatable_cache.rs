use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use luaur_rt::{Lua, Result, UserData, UserDataFields, UserDataMethods};

#[test]
fn userdata_registration_is_reused_per_type_and_vm() -> Result<()> {
    static REGISTRATIONS: AtomicUsize = AtomicUsize::new(0);

    struct RegisteredOnce(i64);

    impl UserData for RegisteredOnce {
        fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
            REGISTRATIONS.fetch_add(1, Ordering::SeqCst);
            methods.add_method("value", |_, this, ()| Ok(this.0));
        }
    }

    REGISTRATIONS.store(0, Ordering::SeqCst);
    let lua = Lua::new();
    let first = lua.create_userdata(RegisteredOnce(1))?;
    let second = lua.create_userdata(RegisteredOnce(2))?;
    lua.globals().set("first", first)?;
    lua.globals().set("second", second)?;

    assert_eq!(
        lua.load("return first:value() + second:value()")
            .eval::<i64>()?,
        3
    );
    assert_eq!(
        REGISTRATIONS.load(Ordering::SeqCst),
        1,
        "userdata methods and fields should be registered once per VM/type"
    );

    let other_lua = Lua::new();
    other_lua.create_userdata(RegisteredOnce(3))?;
    assert_eq!(
        REGISTRATIONS.load(Ordering::SeqCst),
        2,
        "a distinct VM owns a distinct userdata registration"
    );

    Ok(())
}

#[test]
fn cached_field_metatable_supports_coroutines_without_retaining_the_vm() -> Result<()> {
    static FIELD_REGISTRATIONS: AtomicUsize = AtomicUsize::new(0);

    struct TrackedFields {
        value: i64,
        retained: Arc<()>,
    }

    impl UserData for TrackedFields {
        fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
            FIELD_REGISTRATIONS.fetch_add(1, Ordering::SeqCst);
            fields.add_field_method_get("value", |_, this| Ok(this.value));
            fields.add_field_method_get("references", |_, this| {
                Ok(Arc::strong_count(&this.retained) as i64)
            });
        }
    }

    let retained = Arc::new(());
    FIELD_REGISTRATIONS.store(0, Ordering::SeqCst);
    {
        let lua = Lua::new();
        lua.globals().set(
            "first",
            lua.create_userdata(TrackedFields {
                value: 1,
                retained: retained.clone(),
            })?,
        )?;
        lua.globals().set(
            "second",
            lua.create_userdata(TrackedFields {
                value: 2,
                retained: retained.clone(),
            })?,
        )?;
        assert_eq!(
            lua.load("return first.value + second.value, first.references")
                .eval::<(i64, i64)>()?,
            (3, 3)
        );
        assert_eq!(
            lua.load(
                r#"
                    local thread = coroutine.create(function()
                        return first.value + second.value
                    end)
                    local ok, value = coroutine.resume(thread)
                    return ok, value
                "#,
            )
            .eval::<(bool, i64)>()?,
            (true, 3),
            "cached field dispatch should work from a borrowed coroutine state"
        );
        assert_eq!(
            FIELD_REGISTRATIONS.load(Ordering::SeqCst),
            1,
            "field dispatch should be registered once for both instances"
        );
    }
    assert_eq!(
        Arc::strong_count(&retained),
        1,
        "cached dispatch callbacks must not capture a strong Lua handle"
    );

    Ok(())
}
