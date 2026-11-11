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
