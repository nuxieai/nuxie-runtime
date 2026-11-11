//! The incremental collector of `nuxie-luau-vm` under real scripts (Phase
//! 2, lane 3). The vendored VM's step counts depend on its own object
//! sizes, so these are checks of this VM alone, not differential cases.

use super::{Profile, compile};
use nuxie_luau_vm::{GcTune, HostError, HostValue, Vm};

/// A VM with upstream's conformance `collectgarbage` (the options the
/// scripts below use).
fn vm() -> Vm {
    let mut vm = Vm::new().expect("vm");
    vm.register("collectgarbage", |ctx| {
        let option = ctx.opt_string(1)?.unwrap_or_else(|| b"collect".to_vec());
        let extra = ctx.opt_integer(2, 0)?;
        let value = match option.as_slice() {
            b"collect" => {
                ctx.collect();
                HostValue::Number(0.0)
            }
            b"step" => HostValue::Boolean(ctx.gc_step(extra)),
            b"stop" => {
                ctx.set_gc_running(false);
                HostValue::Number(0.0)
            }
            b"restart" => {
                ctx.set_gc_running(true);
                HostValue::Number(0.0)
            }
            b"isrunning" => HostValue::Boolean(ctx.gc_running()),
            b"setstepsize" => HostValue::Number(f64::from(ctx.gc_tune(GcTune::StepSize, extra))),
            other => {
                return Err(HostError::new(format!(
                    "unknown option {}",
                    String::from_utf8_lossy(other)
                )));
            }
        };
        ctx.push(value)?;
        Ok(1)
    })
    .expect("register");
    vm
}

fn run(vm: &mut Vm, source: &str) -> Vec<HostValue> {
    let bytecode = compile(source, Profile::Product).expect("compiles");
    let returned = vm
        .run(b"=collector", &bytecode)
        .unwrap_or_else(|e| panic!("{}", e.message()));
    assert_eq!(vm.accounting_drift(), 0, "accounting drift");
    returned
}

/// Every store the write barrier covers, made after a partial mark of
/// every length from 1 to 80 steps: for some of them the holder is black
/// when the store happens. Without the barrier the stored object is swept
/// while still referenced, and its slot reads as an empty (or reused)
/// object afterwards.
#[test]
fn stores_into_marked_objects_survive_the_cycle() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"local function make()
               local v = nil
               return function(x) if x then v = x end return v end
           end
           -- The stores run in a callee, so once it returns the stored
           -- objects are reachable only through the objects stored into.
           local function stores(holder, cell, mt, list, g)
               holder.x = {value = 1}
               cell({value = 2})
               mt.__index = {value = 3}
               rawset(holder, "y", {value = 4})
               table.insert(list, {value = 5})
               setfenv(g, {value = 6})
               holder[1] = {value = 7}
           end
           local function scenario(k)
               collectgarbage()
               collectgarbage("stop")
               local holder, list, mt = {}, {}, {}
               local proxy = setmetatable({}, mt)
               local cell = make()
               local function g() return value end
               for i = 1, k do
                   if collectgarbage("step", 0) then break end
               end
               stores(holder, cell, mt, list, g)
               repeat until collectgarbage("step", 0)
               for i = 1, 300 do local _ = {value = -1} end
               collectgarbage()
               for i = 1, 300 do local _ = {value = -2} end
               assert(holder.x.value == 1, "SETTABLEKS")
               assert(cell().value == 2, "closed upvalue")
               assert(proxy.value == 3, "metatable field")
               assert(holder.y.value == 4, "rawset")
               assert(list[1].value == 5, "table.insert")
               assert(g() == 6, "setfenv")
               assert(holder[1].value == 7, "SETTABLEN")
               collectgarbage("restart")
           end
           for k = 1, 80 do scenario(k) end
           return "ok""#,
    );
    assert_eq!(returned, vec![HostValue::String(b"ok".to_vec())]);
}

/// gc.luau's `dosteps`: a cycle takes more than ten zero-size steps, fewer
/// large steps than small ones, and one huge step.
#[test]
fn explicit_steps_are_incremental() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"local function dosteps(siz)
               collectgarbage()
               collectgarbage("stop")
               local a = {}
               local b = nil
               for i = 1, 100 do
                   a[i] = {{}}
                   b = {}
               end
               b = nil
               local x = gcinfo()
               local i = 0
               repeat i = i + 1 until collectgarbage("step", siz)
               assert(gcinfo() < x)
               return i
           end
           assert(dosteps(-1) == 1, "a negative step is a whole cycle")
           return dosteps(0), dosteps(2), dosteps(6), dosteps(10000)"#,
    );
    let steps: Vec<f64> = returned
        .iter()
        .map(|v| match v {
            HostValue::Number(n) => *n,
            other => panic!("{other:?}"),
        })
        .collect();
    let [zero, two, six, huge] = steps.as_slice() else {
        panic!("{steps:?}");
    };
    assert!(*zero > 10.0, "{steps:?}");
    assert!(six < two, "{steps:?}");
    assert_eq!(*huge, 1.0, "{steps:?}");
}

/// A zero step size still finishes a cycle (one object per step).
#[test]
fn a_zero_step_size_still_finishes() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"collectgarbage("setstepsize", 0)
           local keep = {}
           for i = 1, 50 do keep[i] = {i} end
           local n = 0
           repeat n = n + 1 until collectgarbage("step", 0) or n > 100000
           return n <= 100000"#,
    );
    assert_eq!(returned, vec![HostValue::Boolean(true)]);
}

/// Automatic collection keeps a churning heap bounded while values stay
/// live across many incremental cycles.
#[test]
fn automatic_steps_bound_the_heap() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"local keep = {}
           local peak = 0
           for i = 1, 200000 do
               local t = {i, tostring(i)}
               if i % 1000 == 0 then keep[#keep + 1] = t end
               if i % 5000 == 0 then peak = math.max(peak, gcinfo()) end
           end
           for i, t in keep do assert(t[1] == i * 1000 and t[2] == tostring(i * 1000)) end
           return peak < 4096, #keep"#,
    );
    assert_eq!(
        returned,
        vec![HostValue::Boolean(true), HostValue::Number(200.0)]
    );
}

/// Review round 1: a mutator that keeps storing into marked tables must
/// not keep the cycle from ending (C++ re-propagates the barrier's queue
/// once, then finishes it in the atomic step).
#[test]
fn sustained_stores_into_marked_tables_do_not_stall_the_cycle() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"local tables = {}
           for i = 1, 300 do tables[i] = {} end
           local peak = 0
           for n = 1, 100000 do
               for i = 1, 300 do tables[i].v = n end
               local _ = {}
               if n % 1000 == 0 then peak = math.max(peak, gcinfo()) end
           end
           return peak < 2048"#,
    );
    assert_eq!(returned, vec![HostValue::Boolean(true)]);
}

/// Review round 1: a table traversed as weak whose mode is removed before
/// the cycle ends is strong at the atomic step and keeps its values.
#[test]
fn a_mode_removed_mid_cycle_keeps_the_values() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"-- Ballast, so a cycle takes many steps.
           local ballast = {}
           for i = 1, 3000 do ballast[i] = {i} end
           local function scenario(k)
               collectgarbage()
               collectgarbage("stop")
               local mt = {__mode = "v"}
               local t = setmetatable({}, mt)
               t[1] = {value = 1}
               for i = 1, k do
                   if collectgarbage("step", 0) then
                       -- The cycle ended while the table was weak.
                       collectgarbage("restart")
                       return 0
                   end
               end
               mt.__mode = nil
               local after = 0
               repeat after += 1 until collectgarbage("step", 0)
               collectgarbage("restart")
               assert(t[1] and t[1].value == 1, "lost after " .. k .. " steps, " .. after .. " more")
               return 1
           end
           local midcycle = 0
           for k = 1, 60 do midcycle += scenario(k) end
           return midcycle"#,
    );
    let Some(HostValue::Number(midcycle)) = returned.first() else {
        panic!("{returned:?}");
    };
    assert!(
        *midcycle >= 40.0,
        "only {midcycle} scenarios removed the mode mid-cycle"
    );
}

/// Review round 1: an explicit step taken while stopped leaves the
/// collector running, as upstream (the stopped threshold is a debt).
#[test]
fn a_step_while_stopped_restarts_like_upstream() {
    let mut vm = vm();
    let returned = run(
        &mut vm,
        r#"local keep = {}
           for i = 1, 2000 do keep[i] = {i} end
           collectgarbage()
           collectgarbage("stop")
           collectgarbage("step", 0)
           collectgarbage("stop")
           local finished = collectgarbage("step", 0)
           return finished, collectgarbage("isrunning")"#,
    );
    assert_eq!(
        returned,
        vec![HostValue::Boolean(false), HostValue::Boolean(true)]
    );
}

/// Review round 2: reclaimable garbage must be collected before a small
/// budget runs out, even while incremental steps are due.
#[test]
fn memory_pressure_wins_over_an_incremental_step() {
    let mut vm = Vm::with_limits(nuxie_luau_vm::Limits {
        max_memory_bytes: Some(40_000),
        ..nuxie_luau_vm::Limits::default()
    })
    .expect("vm");
    let returned = run(
        &mut vm,
        r#"local keep = {}
           for i = 1, 100 do keep[i] = {i} end
           for i = 1, 100 do local t = string.rep("a", 4096 + i) end
           return #keep"#,
    );
    assert_eq!(returned, vec![HostValue::Number(100.0)]);
}

/// Review round 2: dead registers above a native call are cleared when the
/// call collects. Otherwise their ids, freed by that collection and reused
/// by the weak table's values, are rooted again by automatic steps taken in
/// Lua code (which mark up to the frame's top) and the values never clear.
#[test]
fn dead_stack_slots_do_not_root_reused_ids() {
    let names: Vec<String> = (1..=40).map(|i| format!("a{i}")).collect();
    let tables = vec!["{}"; 40].join(", ");
    let source = format!(
        r#"local weak = setmetatable({{}}, {{__mode = "v"}})
           local function put() for i = 1, 40 do weak[i] = {{}} end end
           do local {} = {} end
           collectgarbage()
           put()
           -- Overwrite put's dead registers, which upstream also marks
           -- (they lie below this frame's top).
           local function clobber() local a, b, c, d, e, f, g, h = 1, 2, 3, 4, 5, 6, 7, 8 end
           clobber()
           for i = 1, 200000 do local _ = {{i}} end
           local n = 0
           for _ in pairs(weak) do n += 1 end
           return n"#,
        names.join(", "),
        tables
    );
    let mut vm = vm();
    let returned = run(&mut vm, &source);
    assert_eq!(returned, vec![HostValue::Number(0.0)]);
}
