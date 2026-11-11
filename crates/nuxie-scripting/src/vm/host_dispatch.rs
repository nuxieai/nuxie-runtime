//! `__index` dispatchers that keep method lookup inside the VM.
//!
//! A Rust `__index` closure makes every `value:method(...)` cross into the
//! host twice: once to find the method and once to call it. These Lua
//! dispatchers answer the keys the VM can answer from a table and hand every
//! other key to the unchanged Rust closure, so scripts observe the same values
//! and errors with one host crossing fewer per method call.

use crate::luau::{Function, Lua, Result, Table};

use super::embedded_chunk::EmbeddedChunk;

const HOST_DISPATCH_KEY: &str = "nuxie.host_dispatch";

fn exports(lua: &Lua) -> Result<Table> {
    if let Some(exports) = lua.named_registry_value::<Option<Table>>(HOST_DISPATCH_KEY)? {
        return Ok(exports);
    }
    let exports: Table = EmbeddedChunk::HostDispatch.load(lua)?.call(())?;
    lua.set_named_registry_value(HOST_DISPATCH_KEY, &exports)?;
    Ok(exports)
}

/// `__index` that returns `methods[key]` when it is present and otherwise
/// calls `fallback(self, key)`. `methods` must hold only keys whose
/// `fallback` answer is that same entry.
pub(super) fn methods_first_index(lua: &Lua, methods: &Table, fallback: Function) -> Result<Function> {
    let make: Function = exports(lua)?.get("methodsFirst")?;
    make.call((methods.clone(), fallback))
}

/// `__index` that calls `numeric(self, key)` for number keys and
/// `fallback(self, key)` for every other key.
pub(super) fn numbers_first_index(
    lua: &Lua,
    numeric: Function,
    fallback: Function,
) -> Result<Function> {
    let make: Function = exports(lua)?.get("numbersFirst")?;
    make.call((numeric, fallback))
}

/// Run `patch` the first time this VM creates a value of the type `Marker`
/// stands for. luaur-rt builds one metatable per VM and type and drops it
/// together with the VM's app data, so values created later skip the
/// metatable lookup, an `exec_raw` call that allocates a closure each time.
pub(super) fn patch_metatable_once<Marker: Default + 'static>(
    lua: &Lua,
    patch: impl FnOnce() -> Result<()>,
) -> Result<()> {
    if lua.try_app_data_ref::<Marker>()?.is_some() {
        return Ok(());
    }
    patch()?;
    lua.try_set_app_data(Marker::default())?;
    Ok(())
}

#[allow(dead_code)]
const HOST_DISPATCH_SOURCE: &str = r#"
return {
    methodsFirst = function(methods, fallback)
        return function(self, key)
            local method = methods[key]
            if method ~= nil then
                return method
            end
            return fallback(self, key)
        end
    end,
    numbersFirst = function(numeric, fallback)
        return function(self, key)
            if type(key) == "number" then
                return numeric(self, key)
            end
            return fallback(self, key)
        end
    end,
}
"#;

#[cfg(all(test, feature = "compiler"))]
mod method_dispatch_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::luau::{Error, Function, Lua, Value};

    use super::{methods_first_index, numbers_first_index};

    /// A Rust fallback that records every key it receives.
    fn recording_fallback(lua: &Lua, seen: Rc<RefCell<Vec<String>>>) -> Function {
        lua.create_function(move |_, (_this, key): (Value, Value)| {
            let shown = match &key {
                Value::String(key) => key.to_string_lossy(),
                Value::Boolean(true) => return Err(Error::runtime("bad key")),
                other => format!("{other:?}"),
            };
            seen.borrow_mut().push(shown.clone());
            Ok(format!("fallback:{shown}"))
        })
        .expect("fallback")
    }

    #[test]
    fn method_names_resolve_without_the_fallback() {
        let lua = Lua::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let methods = lua.create_table();
        let method = lua.create_function(|_, ()| Ok(1)).expect("method");
        methods.set("lineTo", method.clone()).expect("method entry");
        let index =
            methods_first_index(&lua, &methods, recording_fallback(&lua, seen.clone())).expect("index");
        assert_eq!(index.info().what, "Lua");

        let found: Function = index.call((Value::Nil, "lineTo")).expect("method lookup");
        assert_eq!(found, method);
        assert!(seen.borrow().is_empty(), "method lookup reached the host: {:?}", seen.borrow());

        let numeric: String = index.call((Value::Nil, 1)).expect("numeric key");
        let missing: String = index.call((Value::Nil, "missing")).expect("unknown name");
        assert_eq!(numeric, "fallback:Integer(1)");
        assert_eq!(missing, "fallback:missing");
        assert_eq!(*seen.borrow(), ["Integer(1)", "missing"]);

        let error = index.call::<Value>((Value::Nil, true)).expect_err("fallback error");
        assert!(error.to_string().contains("bad key"), "{error}");
    }

    #[test]
    fn number_keys_reach_the_rust_closure_and_others_the_dispatcher() {
        let lua = Lua::new();
        let numbers = Rc::new(RefCell::new(Vec::new()));
        let others = Rc::new(RefCell::new(Vec::new()));
        let index = numbers_first_index(
            &lua,
            recording_fallback(&lua, numbers.clone()),
            recording_fallback(&lua, others.clone()),
        )
        .expect("index");
        for key in [Value::Integer(2), Value::Number(2.5), Value::Number(f64::NAN)] {
            index.call::<String>((Value::Nil, key)).expect("number key");
        }
        index.call::<String>((Value::Nil, "length")).expect("string key");
        assert_eq!(numbers.borrow().len(), 3);
        assert_eq!(*others.borrow(), ["length"]);
    }

    fn rive_lua() -> Lua {
        let lua = Lua::new();
        super::super::lua_vec2d::install_vector_global(&lua).expect("Vector installs");
        super::super::lua_path::install_path_global(&lua).expect("Path installs");
        lua
    }

    #[test]
    fn path_answers_methods_in_the_vm_and_numbers_in_rust() {
        let lua = rive_lua();
        let observed: Vec<String> = lua
            .load(
                r#"
                local p = Path.new()
                p:moveTo(Vector.xy(0, 0))
                p:lineTo(Vector.xy(10, 0))
                p:lineTo(Vector.xy(10, 5))
                local ok, err = pcall(function() return p[true] end)
                return {
                    debug.info(getmetatable(p).__index, "s"),
                    tostring(#p),
                    type(p.lineTo),
                    tostring(p.missing),
                    p[1].type,
                    p[2].type,
                    tostring(p[2][1]),
                    p[3.0].type,
                    p[0].type,
                    p[99].type,
                    tostring(ok),
                    tostring(string.find(tostring(err), "Path index must be a string or number", 1, true) ~= nil),
                }
                "#,
            )
            .eval()
            .expect("path script");
        assert_eq!(
            observed,
            [
                "nuxie_host_dispatch",
                "3",
                "function",
                "nil",
                "moveTo",
                "lineTo",
                "10, 0, 0",
                "lineTo",
                "none",
                "none",
                "false",
                "true",
            ]
        );
    }

    #[test]
    fn vector_instance_methods_resolve_in_the_vm_and_other_keys_in_rust() {
        let lua = rive_lua();
        let observed: Vec<String> = lua
            .load(
                r#"
                local v = Vector.xy(3, 4)
                local ok, err = pcall(function() return v.nope end)
                return {
                    debug.info(getmetatable(v).__index, "s"),
                    tostring(v.length == Vector.length),
                    tostring(v:length()),
                    tostring(v[1]),
                    tostring(v["2"]),
                    tostring(v.x),
                    tostring(ok),
                    tostring(string.find(tostring(err), "'nope' is not a valid index of Vector", 1, true) ~= nil),
                }
                "#,
            )
            .eval()
            .expect("vector script");
        assert_eq!(
            observed,
            ["nuxie_host_dispatch", "true", "5", "3", "4", "3", "false", "true"]
        );
    }
}

#[cfg(all(test, feature = "compiler"))]
mod patch_once_tests {
    use std::cell::Cell;

    use crate::luau::{Function, Lua};

    use super::patch_metatable_once;

    #[derive(Default)]
    struct Marker;

    #[test]
    fn patch_runs_once_per_vm_and_again_in_a_new_vm() {
        let calls = Cell::new(0);
        for _ in 0..3 {
            let lua = Lua::new();
            for _ in 0..4 {
                patch_metatable_once::<Marker>(&lua, || {
                    calls.set(calls.get() + 1);
                    Ok(())
                })
                .expect("patch");
            }
            // Dropping the VM clears its app data with its metatable cache,
            // so the next VM (possibly at the same address) patches again.
        }
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn failed_patch_is_retried() {
        let lua = Lua::new();
        let calls = Cell::new(0);
        let failing = patch_metatable_once::<Marker>(&lua, || {
            calls.set(calls.get() + 1);
            Err(crate::luau::Error::runtime("patch failed"))
        });
        assert!(failing.is_err());
        patch_metatable_once::<Marker>(&lua, || {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .expect("patch");
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn path_metatable_is_patched_in_every_vm() {
        for _ in 0..3 {
            let lua = Lua::new();
            super::super::lua_path::install_path_global(&lua).expect("Path installs");
            let (first, second, command): (Function, Function, String) = lua
                .load(
                    r#"
                    local a = Path.new()
                    local b = Path.new()
                    a:moveTo(vector.create(1, 2, 0))
                    return getmetatable(a).__index, getmetatable(b).__index, a[1].type
                    "#,
                )
                .eval()
                .expect("path script");
            assert_eq!(first, second);
            assert_eq!(first.info().what, "Lua");
            assert_eq!(command, "moveTo");
        }
    }
}
