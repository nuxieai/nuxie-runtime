//! The embedding API of `nuxie-luau-vm` under real scripts (Phase 2 lane
//! 6): the parts the crate's own unit tests cannot reach without a
//! compiler. The differential side is `host_surface.rs`.

use std::cell::Cell;
use std::rc::Rc;

use super::{Profile, compile};
use nuxie_luau_vm::{
    AnyUserData, CallStatus, Error, Function, HostError, HostValue, MultiValue, RunState, Table,
    UserDataType, Value, Vm,
};

fn bytecode(source: &str) -> Vec<u8> {
    compile(source, Profile::Product).unwrap_or_else(|e| panic!("{e}\n{source}"))
}

fn run(vm: &mut Vm, source: &str) -> Result<Vec<HostValue>, Error> {
    vm.run(b"=api", &bytecode(source))
}

fn text(s: &str) -> HostValue {
    HostValue::String(s.as_bytes().to_vec())
}

fn set_global(vm: &mut Vm, name: &str, value: impl nuxie_luau_vm::IntoLua) {
    let globals = vm.globals().expect("globals");
    globals.raw_set(vm, name, value).expect("set");
}

#[test]
fn the_interrupt_fires_and_can_stop_a_script() {
    let mut vm = Vm::new().expect("vm");
    let fired = Rc::new(Cell::new(0u32));
    let seen = Rc::clone(&fired);
    let previous = vm.set_interrupt(move || {
        seen.set(seen.get() + 1);
        if seen.get() > 1000 {
            Err(HostError::raw("execution exceeded 1.0 second(s) timeout"))
        } else {
            Ok(())
        }
    });
    assert!(previous.is_none());
    let err = run(&mut vm, "local n = 0 while true do n += 1 end").unwrap_err();
    assert_eq!(err.message(), "execution exceeded 1.0 second(s) timeout");
    assert_eq!(fired.get(), 1001);
    assert_eq!(vm.accounting_drift(), 0);

    // Scoped: swap one in, restore the old one.
    let scoped = Rc::new(Cell::new(0u32));
    let count = Rc::clone(&scoped);
    let outer = vm.set_interrupt(move || {
        count.set(count.get() + 1);
        Ok(())
    });
    assert_eq!(
        run(&mut vm, "for i = 1, 10 do end return 1").unwrap(),
        vec![HostValue::Number(1.0)]
    );
    assert_eq!(scoped.get(), 11);
    vm.replace_interrupt(outer);
    fired.set(0);
    run(&mut vm, "for i = 1, 10 do end").unwrap();
    assert_eq!(fired.get(), 11, "the outer interrupt is back");
    vm.remove_interrupt();
    run(&mut vm, "for i = 1, 5000 do end").unwrap();
    assert_eq!(fired.get(), 11, "removed");
}

#[test]
fn an_interrupt_error_is_catchable_and_positioned_like_lual_error() {
    let mut vm = Vm::new().expect("vm");
    let armed = Rc::new(Cell::new(false));
    let trip = Rc::clone(&armed);
    vm.set_interrupt(move || {
        if trip.replace(false) {
            Err(HostError::new("stopped"))
        } else {
            Ok(())
        }
    });
    let arm = Rc::clone(&armed);
    let f = vm
        .create_function("arm", move |_, (): ()| {
            arm.set(true);
            Ok(())
        })
        .unwrap();
    set_global(&mut vm, "arm", f);
    let got = run(
        &mut vm,
        "local function body() arm() for i = 1, 3 do end end
         return pcall(body)",
    )
    .unwrap();
    assert_eq!(got, vec![HostValue::Boolean(false), text("=api:1: stopped")]);
}

#[test]
fn a_parked_main_thread_refuses_calls_until_it_finishes() {
    let mut vm = Vm::new().expect("vm");
    let f = vm.create_function("one", |_, (): ()| Ok(1.0)).unwrap();
    let state = vm
        .start(b"=api", &bytecode("local x = coroutine.yield(5) return x * 2"))
        .unwrap();
    assert_eq!(state, RunState::Yielded(vec![HostValue::Number(5.0)]));
    match f.call::<_, f64>(&mut vm, ()) {
        Err(Error::Host(m)) => assert!(m.contains("suspended in a yield"), "{m}"),
        other => panic!("{other:?}"),
    }
    assert!(vm.create_table().is_err());
    let state = vm.resume(&[HostValue::Number(4.0)]).unwrap();
    assert_eq!(state, RunState::Finished(vec![HostValue::Number(8.0)]));
    assert_eq!(f.call::<_, f64>(&mut vm, ()).unwrap(), 1.0);
    assert_eq!(vm.accounting_drift(), 0);
}

#[test]
fn a_host_function_yields_across_a_lua_coroutine_with_its_continuation() {
    let mut vm = Vm::new().expect("vm");
    // C++ multipleYields: yields base+1..base+3, keeping its state at
    // stack index 2, then returns base+4.
    let f = vm
        .create_host_function_k(
            "multipleYields",
            |ctx| {
                ctx.set_top(1)?;
                let base = ctx.check_integer(1)?;
                ctx.push(1.0)?;
                ctx.push(f64::from(base + 1))?;
                ctx.yield_values(1)
            },
            |ctx, status| {
                assert_eq!(status, CallStatus::Ok);
                let base = ctx.check_integer(1)?;
                let pos = ctx.check_integer(2)? + 1;
                ctx.push(f64::from(pos))?;
                ctx.replace(2)?;
                ctx.push(f64::from(base + pos))?;
                if pos < 4 { ctx.yield_values(1) } else { Ok(1) }
            },
        )
        .unwrap();
    set_global(&mut vm, "multipleYields", f);
    let got = run(
        &mut vm,
        "local co = coroutine.wrap(multipleYields)
         return co(10), co(), co(), co()",
    )
    .unwrap();
    let n = |x| HostValue::Number(x);
    assert_eq!(got, vec![n(11.0), n(12.0), n(13.0), n(14.0)]);
    assert_eq!(vm.accounting_drift(), 0);
}

#[test]
fn namecall_dispatch_sees_the_method_name() {
    let mut vm = Vm::new().expect("vm");
    struct Input;
    let mut ty = UserDataType::<Input>::new().type_name("Input");
    ty.add_meta_host_function("__namecall", |ctx| {
        let name = ctx.namecall_method().unwrap_or_default();
        if name == b"hit" {
            let x: f64 = ctx.get(2)?;
            ctx.push(x * 2.0)?;
            Ok(1)
        } else {
            Err(HostError::new(format!(
                "{} is not a valid method of Input",
                String::from_utf8_lossy(&name)
            )))
        }
    });
    vm.register_userdata_type(ty).unwrap();
    let ud = vm.create_userdata(Input).unwrap();
    set_global(&mut vm, "input", ud);
    let got = run(
        &mut vm,
        "return input:hit(21), select(2, pcall(function() return input:miss() end))",
    )
    .unwrap();
    assert_eq!(
        got,
        vec![
            HostValue::Number(42.0),
            text("=api:1: miss is not a valid method of Input")
        ]
    );
    assert!(vm.namecall_method().is_none(), "only inside __namecall");
}

#[test]
fn direct_fields_skip_index_for_their_keys() {
    #[derive(Clone, Copy)]
    struct Mat4([f32; 16]);
    const TAG: u8 = 62;
    let mut vm = Vm::new().expect("vm");
    let index_calls = Rc::new(Cell::new(0u32));
    let calls = Rc::clone(&index_calls);
    let mt = vm.create_table().unwrap();
    let index = vm
        .create_host_function("__index", move |ctx| {
            calls.set(calls.get() + 1);
            let key: String = ctx.get(2)?;
            Err(HostError::new(format!("'{key}' is not a valid index of Mat4")))
        })
        .unwrap();
    mt.raw_set(&mut vm, "__index", index).unwrap();
    mt.raw_set(&mut vm, "__type", "Mat4").unwrap();
    vm.set_tag_metatable(TAG, Some(&mt)).unwrap();
    vm.register_direct_fields::<Mat4>(
        TAG,
        &[("m11", |m| f64::from(m.0[0])), ("m44", |m| f64::from(m.0[15]))],
    )
    .unwrap();
    let mut values = [0f32; 16];
    values[0] = 1.5;
    values[15] = 4.0;
    let ud = vm.create_userdata_tagged(Mat4(values), TAG).unwrap();
    set_global(&mut vm, "m", &ud);
    let got = run(
        &mut vm,
        "local mm = m
         local a, b = mm.m11, mm.m44
         local ok, e = pcall(function() return mm.m12 end)
         return a, b, e, typeof(mm)",
    )
    .unwrap();
    assert_eq!(
        got,
        vec![
            HostValue::Number(1.5),
            HostValue::Number(4.0),
            text("=api:3: 'm12' is not a valid index of Mat4"),
            text("Mat4"),
        ]
    );
    assert_eq!(index_calls.get(), 1, "only the unknown key reached __index");
    ud.set_copy(&vm, Mat4([2.0; 16])).unwrap();
    // A global path (`return m.m11`) compiles to GETIMPORT, which resolves
    // through `__index` here as in the C++ VM: direct fields apply only
    // at GETTABLEKS (lvmexecute.cpp), so the read goes through a local.
    assert_eq!(
        run(&mut vm, "local mm = m return mm.m11").unwrap(),
        vec![HostValue::Number(2.0)]
    );
    assert_eq!(index_calls.get(), 1);
}

#[test]
fn inspect_stack_reports_the_calling_chunk() {
    let mut vm = Vm::new().expect("vm");
    let f = vm
        .create_host_function("where", |ctx| {
            let info = ctx.inspect_stack(1).expect("a caller");
            let own = ctx.inspect_stack(0).expect("itself");
            assert!(own.is_native);
            ctx.push(info.source.as_slice())?;
            ctx.push(f64::from(info.current_line))?;
            Ok(2)
        })
        .unwrap();
    set_global(&mut vm, "where_", f);
    let got = vm
        .run(b"@lib@1.0/module", &bytecode("\n\nreturn where_()"))
        .unwrap();
    assert_eq!(got, vec![text("@lib@1.0/module"), HostValue::Number(3.0)]);
}

struct Point {
    x: f64,
}

fn point_type() -> UserDataType<Point> {
    let mut ty = UserDataType::<Point>::new().type_name("Point");
    ty.add_field_method_get("x", |_, p| Ok(p.x));
    ty.add_field_method_set("x", |_, p, x: f64| {
        p.x = x;
        Ok(())
    });
    ty.add_method_mut("bump", |_, p, by: f64| {
        p.x += by;
        Ok(p.x)
    });
    // Calls back into Lua while it holds the borrow.
    ty.add_method("visit", |ctx, p, f: Function| {
        let _ = p.x;
        let self_ud: AnyUserData = ctx.get(1)?;
        f.call::<_, ()>(ctx, self_ud).map_err(HostError::from)
    });
    ty
}

#[test]
fn userdata_types_dispatch_like_luaur_rt_from_scripts() {
    let mut vm = Vm::new().expect("vm");
    vm.register_userdata_type(point_type()).unwrap();
    let p = vm.create_userdata(Point { x: 1.0 }).unwrap();
    set_global(&mut vm, "p", &p);
    let got = run(
        &mut vm,
        "p.x = 5
         local a = p:bump(2)
         local ok, e = pcall(function() p.y = 1 end)
         local ok2, e2 = pcall(function() p:visit(function(q) q:bump(1) end) end)
         local ok3, e3 = pcall(function() return p.bump(5, 1) end)
         return a, p.x, e, ok2, e2, e3, typeof(p)",
    )
    .unwrap();
    assert_eq!(
        got,
        vec![
            HostValue::Number(7.0),
            HostValue::Number(7.0),
            text("attempt to set unknown field 'y' on userdata"),
            HostValue::Boolean(false),
            text("=api:4: userdata already borrowed"),
            text("=api:5: invalid argument #1 to 'bump' (Point expected, got number)"),
            text("Point"),
        ]
    );
    assert_eq!(p.borrow(&vm, |p: &Point| p.x).unwrap(), 7.0);
    assert_eq!(vm.accounting_drift(), 0);
}

#[test]
fn handles_minted_on_a_coroutine_work_on_the_main_thread() {
    let mut vm = Vm::new().expect("vm");
    let stash: Rc<std::cell::RefCell<Option<Function>>> = Rc::default();
    let into = Rc::clone(&stash);
    let keep = vm
        .create_function("keep", move |_, f: Function| {
            *into.borrow_mut() = Some(f);
            Ok(())
        })
        .unwrap();
    set_global(&mut vm, "keep", keep);
    run(
        &mut vm,
        "local co = coroutine.create(function()
             local n = 40
             keep(function(k) return n + k end)
         end)
         coroutine.resume(co)",
    )
    .unwrap();
    vm.collect();
    let f = stash.borrow_mut().take().expect("kept");
    assert_eq!(f.call::<_, f64>(&mut vm, 2.0).unwrap(), 42.0);
}

#[test]
fn modules_load_with_their_own_environment() {
    let mut vm = Vm::new().expect("vm");
    let env = vm.create_table().unwrap();
    env.raw_set(&mut vm, "answer", 42.0).unwrap();
    let chunk = vm
        .load_with_env(b"module", &bytecode("return answer, print"), &env)
        .unwrap();
    let got: MultiValue = chunk.call(&mut vm, ()).unwrap();
    assert_eq!(got.0[0].as_number(), Some(42.0));
    assert!(got.0[1].is_nil(), "no globals leak into the module");
    assert_eq!(
        chunk.environment(&mut vm).unwrap().map(|t| vm.raw_equal(&Value::Table(t), &Value::Table(env.clone())).unwrap()),
        Some(true)
    );
    let main = vm.load(b"=main", &bytecode("return answer")).unwrap();
    let thread = vm.create_thread_with_env(&main, &env).unwrap();
    // The thread's globals are `env`; a chunk loaded *on* it would see
    // them, but `main` was loaded with the VM's globals.
    let got: Value = thread.resume(&mut vm, ()).unwrap();
    assert!(got.is_nil());
}

#[test]
fn a_script_hitting_the_memory_limit_leaves_no_drift() {
    let mut vm = Vm::new().expect("vm");
    vm.collect();
    vm.set_memory_limit(Some(vm.used_memory() + 256 * 1024));
    let err = run(
        &mut vm,
        "local t = {} for i = 1, 1e7 do t[i] = {i} end",
    )
    .unwrap_err();
    assert!(err.is_memory(), "{err:?}");
    assert!(vm.memory_exhausted());
    assert_eq!(vm.accounting_drift(), 0);
    vm.collect();
    assert_eq!(vm.accounting_drift(), 0);
    vm.set_memory_limit(None);
    vm.clear_memory_exhausted();
    assert_eq!(
        run(&mut vm, "return #string.rep('x', 100000)").unwrap(),
        vec![HostValue::Number(100000.0)]
    );
}

#[test]
fn table_handles_see_script_metatables() {
    let mut vm = Vm::new().expect("vm");
    let t: Table = {
        let f = vm
            .load(
                b"=t",
                &bytecode(
                    "return setmetatable({}, {
                        __index = function(_, k) return k .. '!' end,
                        __len = function() return 7 end,
                        __newindex = function(t, k, v) rawset(t, k, v * 10) end,
                    })",
                ),
            )
            .unwrap();
        f.call(&mut vm, ()).unwrap()
    };
    let v: String = t.get(&mut vm, "hi").unwrap();
    assert_eq!(v, "hi!");
    assert_eq!(t.len(&mut vm).unwrap(), 7.0);
    t.set(&mut vm, "k", 2.0).unwrap();
    let k: f64 = t.raw_get(&mut vm, "k").unwrap();
    assert_eq!(k, 20.0);
}

#[test]
fn a_host_vector_metatable_keeps_components_and_unnamed_namecall_sees_the_method() {
    let mut vm = Vm::new().expect("vm");
    let mt = vm.create_table().unwrap();
    // Debug name "" (C++ `lua_pushcfunction(L, f, nullptr)`): the
    // namecall atom must not depend on it.
    let namecall = vm
        .create_host_function("", |ctx| {
            let name = ctx.namecall_method().unwrap_or_default();
            ctx.push(name.as_slice())?;
            Ok(1)
        })
        .unwrap();
    mt.raw_set(&mut vm, "__namecall", namecall).unwrap();
    let index = vm
        .create_host_function("", |ctx| {
            let key: String = ctx.get(2)?;
            ctx.push(format!("index {key}"))?;
            Ok(1)
        })
        .unwrap();
    mt.raw_set(&mut vm, "__index", index).unwrap();
    vm.set_vector_metatable(Some(&mt)).unwrap();
    let got = run(
        &mut vm,
        "local v = vector.create(1, 2, 3)
         return v.x, v.Y, v.z, v.w, v.Magnitude, v:Cross()",
    )
    .unwrap();
    assert_eq!(
        got,
        vec![
            HostValue::Number(1.0),
            HostValue::Number(2.0),
            HostValue::Number(3.0),
            text("index w"),
            text("index Magnitude"),
            text("Cross"),
        ]
    );
}
