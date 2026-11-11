//! The host helpers upstream's `Conformance.test.cpp` installs for single
//! scripts, written against the `nuxie-luau-vm` embedding API (plan
//! `docs/luau-safe-vm-plan.md`, Phase 2 lane 6). Each mirrors its C++
//! function; where the C++ reads `lua_upvalueindex`, the Rust closure
//! captures the value instead. A C++ `LUAU_ASSERT` inside a helper becomes
//! a raised error, so a broken invariant fails the script.

use nuxie_luau_vm::{CallStatus, Error, HostCtx, HostError, LightUserData, Value, Vm};

/// Installs `name`'s helpers (no-op for scripts that need none).
pub fn install(vm: &mut Vm, name: &str) -> Result<(), Error> {
    match name {
        "tables.luau" => tables(vm),
        "cyield.luau" => cyield(vm),
        "userdata.luau" => userdata(vm),
        "vector.luau" => vector(vm),
        "udata_direct.luau" | "native_userdata.luau" => {
            vector(vm)?;
            userdata_helpers(vm)
        }
        _ => Ok(()),
    }
}

fn set_global(vm: &mut Vm, name: &str, value: impl nuxie_luau_vm::IntoLua) -> Result<(), Error> {
    let globals = vm.globals()?;
    globals.raw_set(vm, name, value)
}

fn check(condition: bool, what: &str) -> Result<(), HostError> {
    if condition {
        Ok(())
    } else {
        Err(HostError::raw(format!("helper assertion failed: {what}")))
    }
}

// ---- tables.luau --------------------------------------------------------

/// `makelud(n)` is lightuserdata `n`; `makelud(obj)` is the lightuserdata
/// of `obj`'s address (`lua_topointer`).
fn tables(vm: &mut Vm) -> Result<(), Error> {
    let makelud = vm.create_host_function("makelud", |ctx| {
        let ptr = if matches!(ctx.value(1)?, Value::Number(_)) {
            u64::from(ctx.check_unsigned(1)?)
        } else {
            let value = ctx.value(1)?;
            let p = ctx.to_pointer(&value)?;
            check(p != 0, "makelud expects a GC value")?;
            p
        };
        ctx.push(LightUserData { ptr, tag: 0 })?;
        Ok(1)
    })?;
    set_global(vm, "makelud", makelud)
}

// ---- cyield.luau --------------------------------------------------------

fn integer_at(ctx: &mut HostCtx<'_>, i: isize) -> Result<i32, HostError> {
    // `lua_tointeger`: 0 for a non-number.
    Ok(match ctx.value(i)? {
        Value::Number(n) => n as i32,
        _ => 0,
    })
}

fn cyield(vm: &mut Vm) -> Result<(), Error> {
    let f = vm.create_host_function_k(
        "singleYield",
        |ctx| {
            ctx.push(2.0)?;
            ctx.yield_values(1)
        },
        |ctx, _| {
            ctx.push(4.0)?;
            Ok(1)
        },
    )?;
    set_global(vm, "singleYield", f)?;

    let f = vm.create_host_function_k(
        "multipleYields",
        |ctx| {
            ctx.set_top(1)?;
            let base = ctx.check_integer(1)?;
            let pos = 1;
            ctx.push(pos)?;
            ctx.push(base + pos)?;
            ctx.yield_values(1)
        },
        |ctx, _| {
            let base = ctx.check_integer(1)?;
            let pos = ctx.check_integer(2)? + 1;
            ctx.push(pos)?;
            ctx.replace(2)?;
            ctx.push(base + pos)?;
            if pos < 4 { ctx.yield_values(1) } else { Ok(1) }
        },
    )?;
    set_global(vm, "multipleYields", f)?;

    let f = vm.create_host_function_k(
        "multipleYieldsWithNestedCall",
        |ctx| {
            ctx.set_top(2)?;
            let nested_should_yield = ctx.check_boolean(2)?;
            ctx.push(0)?; // state
            let context = 5;
            let nested = if nested_should_yield {
                ctx.create_host_function_k(
                    "nestedMultipleYieldHelper",
                    move |ctx| {
                        ctx.push(100 + context)?;
                        ctx.yield_values(1)
                    },
                    move |ctx, _| {
                        ctx.push(110 + context)?;
                        Ok(1)
                    },
                )?
            } else {
                ctx.create_host_function("nestedMultipleYieldHelperNonYielding", move |ctx| {
                    ctx.push(105 + context)?;
                    Ok(1)
                })?
            };
            ctx.push(nested)?;
            ctx.call_yieldable(0, 1)
        },
        |ctx, _| {
            let state = ctx.check_integer(3)?;
            ctx.push(state + 1)?;
            ctx.replace(3)?;
            match state {
                0 => {
                    let n = ctx.top().saturating_sub(3);
                    ctx.yield_values(n)
                }
                1 => {
                    let base = ctx.check_integer(1)?;
                    ctx.push(f64::from(base) + 200.0)?;
                    ctx.yield_values(1)
                }
                _ => {
                    let base = ctx.check_integer(1)?;
                    ctx.push(f64::from(base) + 210.0)?;
                    Ok(1)
                }
            }
        },
    )?;
    set_global(vm, "multipleYieldsWithNestedCall", f)?;

    let f = vm.create_host_function_k(
        "passthroughCall",
        |ctx| {
            ctx.push_copy(1)?;
            ctx.push_copy(2)?;
            ctx.push_copy(3)?;
            ctx.call_yieldable(2, 1)
        },
        |ctx, _| {
            check(ctx.top() == 4, "3 original arguments and the return value")?;
            let x: f64 = ctx.get(-1)?;
            check(x == 0.5, "passthroughCall result")?;
            Ok(1)
        },
    )?;
    set_global(vm, "passthroughCall", f)?;

    let f = vm.create_host_function_k(
        "passthroughCallMoreResults",
        |ctx| {
            ctx.push_copy(1)?;
            ctx.push_copy(2)?;
            ctx.push_copy(3)?;
            ctx.call_yieldable(2, 10)
        },
        |ctx, _| {
            check(ctx.top() == 13, "3 original arguments and 10 return values")?;
            for _ in 0..9 {
                check(matches!(ctx.value(-1)?, Value::Nil), "padding results are nil")?;
                ctx.pop(1)?;
            }
            let x: f64 = ctx.get(-1)?;
            check(x == 0.5, "passthroughCallMoreResults result")?;
            Ok(1)
        },
    )?;
    set_global(vm, "passthroughCallMoreResults", f)?;

    let f = vm.create_host_function_k(
        "passthroughCallArgReuse",
        |ctx| ctx.call_yieldable(2, 1),
        |ctx, _| {
            check(ctx.top() == 1, "only the return value remains")?;
            let x: f64 = ctx.get(-1)?;
            check(x == 0.5, "passthroughCallArgReuse result")?;
            Ok(1)
        },
    )?;
    set_global(vm, "passthroughCallArgReuse", f)?;

    let f = vm.create_host_function_k(
        "passthroughCallVaradic",
        |ctx| {
            ctx.check_any(1)?;
            let n = ctx.top().saturating_sub(1);
            ctx.call_yieldable(n, -1)
        },
        |ctx, _| Ok(ctx.top()),
    )?;
    set_global(vm, "passthroughCallVaradic", f)?;

    let f = vm.create_host_function_k(
        "passthroughCallWithState",
        |ctx| {
            ctx.check_any(1)?;
            let args = ctx.top().saturating_sub(1);
            ctx.push(42.0)?;
            ctx.insert(1)?;
            ctx.call_yieldable(args, -1)
        },
        |ctx, _| {
            check(ctx.check_integer(1)? == 42, "state survives the call")?;
            Ok(ctx.top().saturating_sub(1))
        },
    )?;
    set_global(vm, "passthroughCallWithState", f)?;

    for (name, protected_second) in [("pcallThenCall", false), ("pcallThenPcall", true)] {
        let f = vm.create_host_function_k(
            name,
            |ctx| {
                ctx.check_any(1)?;
                ctx.check_any(2)?;
                ctx.push(0)?; // state
                ctx.push(0)?; // multiplier
                ctx.push_copy(1)?;
                ctx.pcall_yieldable(0, 1)
            },
            move |ctx, status| {
                let state = ctx.check_integer(3)?;
                if state == 0 {
                    if status != CallStatus::Ok {
                        ctx.push(-1)?;
                    }
                    ctx.replace(4)?;
                    ctx.push(1)?;
                    ctx.replace(3)?;
                    ctx.push_copy(2)?;
                    return if protected_second {
                        ctx.pcall_yieldable(0, -1)
                    } else {
                        ctx.call_yieldable(0, -1)
                    };
                }
                let multiplier = ctx.check_integer(4)?;
                let value = if status != CallStatus::Ok {
                    check(protected_second, "only pcallThenPcall sees an error here")?;
                    -1
                } else {
                    integer_at(ctx, -1)?
                };
                ctx.push(multiplier * value)?;
                Ok(1)
            },
        )?;
        set_global(vm, name, f)?;
    }
    Ok(())
}

// ---- userdata.luau ------------------------------------------------------

const INT64_TAG: u8 = 1;

/// C++ `getInt64`: the tagged payload, or a number as `lua_tointeger`
/// converts it, else `luaL_typeerror(L, 1, "int64")` (argument 1 always,
/// as upstream).
fn get_int64(ctx: &mut HostCtx<'_>, n: usize) -> Result<i64, HostError> {
    if let Some(cell) = ctx.to_userdata_tagged::<i64>(n, INT64_TAG) {
        let value = *cell.try_borrow().map_err(|_| HostError::raw("int64 already borrowed"))?;
        return Ok(value);
    }
    let value = ctx.value(n as isize)?;
    match ctx.coerce_number(&value)? {
        // `lua_tointeger` is `int`.
        Some(number) => Ok(i64::from(number as i32)),
        None => Err(ctx.type_error(1, "int64")),
    }
}

/// C++ `pushInt64`: a tagged userdata with the `int64` metatable from the
/// registry (`luaL_getmetatable`).
fn push_int64(ctx: &mut HostCtx<'_>, value: i64) -> Result<usize, HostError> {
    let ud = ctx.create_userdata_tagged(value, INT64_TAG)?;
    let mt: Option<nuxie_luau_vm::Table> = ctx.named_registry_value("int64")?;
    ud.set_metatable(ctx, mt.as_ref())?;
    ctx.push(ud)?;
    Ok(1)
}

fn int64_field(ctx: &mut HostCtx<'_>) -> Result<(std::rc::Rc<std::cell::RefCell<i64>>, String), HostError> {
    let Some(cell) = ctx.to_userdata_tagged::<i64>(1, INT64_TAG) else {
        return Err(ctx.type_error(1, "userdata"));
    };
    let name: String = ctx.get(2)?;
    Ok((cell, name))
}

type Int64Op = fn(i64, i64) -> Result<i64, HostError>;
type Int64Compare = fn(i64, i64) -> bool;
type Vec2Op = fn(f32, f32) -> f32;

fn userdata(vm: &mut Vm) -> Result<(), Error> {
    let mt = vm.create_table()?;
    mt.raw_set(vm, "__type", "int64")?;
    let index = vm.create_host_function("", |ctx| {
        let (cell, name) = int64_field(ctx)?;
        if name == "value" {
            let value = *cell.try_borrow().map_err(|_| HostError::raw("int64 already borrowed"))?;
            ctx.push(value as f64)?;
            return Ok(1);
        }
        Err(HostError::new(format!("unknown field {name}")))
    })?;
    mt.raw_set(vm, "__index", index)?;
    let newindex = vm.create_host_function("", |ctx| {
        let (cell, name) = int64_field(ctx)?;
        if name == "value" {
            let value = ctx.check_number(3)?;
            *cell.try_borrow_mut().map_err(|_| HostError::raw("int64 already borrowed"))? = value as i64;
            return Ok(0);
        }
        Err(HostError::new(format!("unknown field {name}")))
    })?;
    mt.raw_set(vm, "__newindex", newindex)?;

    let compare: [(&str, Int64Compare); 3] = [
        ("__eq", |a, b| a == b),
        ("__lt", |a, b| a < b),
        ("__le", |a, b| a <= b),
    ];
    for (event, op) in compare {
        let f = vm.create_host_function("", move |ctx| {
            let a = get_int64(ctx, 1)?;
            let b = get_int64(ctx, 2)?;
            ctx.push(op(a, b))?;
            Ok(1)
        })?;
        mt.raw_set(vm, event, f)?;
    }

    let arith: [(&str, Int64Op); 7] = [
        ("__add", |a, b| Ok(a.wrapping_add(b))),
        ("__sub", |a, b| Ok(a.wrapping_sub(b))),
        ("__mul", |a, b| Ok(a.wrapping_mul(b))),
        // __div truncates; __idiv floors through doubles (upstream's test
        // semantics).
        ("__div", |a, b| a.checked_div(b).ok_or_else(|| HostError::raw("int64 division by zero"))),
        ("__idiv", |a, b| Ok((a as f64 / b as f64).floor() as i64)),
        ("__mod", |a, b| a.checked_rem(b).ok_or_else(|| HostError::raw("int64 division by zero"))),
        ("__pow", |a, b| Ok((a as f64).powf(b as f64) as i64)),
    ];
    for (event, op) in arith {
        let f = vm.create_host_function("", move |ctx| {
            let a = get_int64(ctx, 1)?;
            let b = get_int64(ctx, 2)?;
            push_int64(ctx, op(a, b)?)
        })?;
        mt.raw_set(vm, event, f)?;
    }
    let unm = vm.create_host_function("", |ctx| {
        let a = get_int64(ctx, 1)?;
        push_int64(ctx, a.wrapping_neg())
    })?;
    mt.raw_set(vm, "__unm", unm)?;
    let tostring = vm.create_host_function("", |ctx| {
        let a = get_int64(ctx, 1)?;
        ctx.push(a.to_string())?;
        Ok(1)
    })?;
    mt.raw_set(vm, "__tostring", tostring)?;
    vm.set_named_registry_value("int64", &mt)?;

    let ctor = vm.create_host_function("int64", |ctx| {
        let v = ctx.check_number(1)?;
        push_int64(ctx, v as i64)
    })?;
    set_global(vm, "int64", ctor)
}

// ---- vector.luau (setupVectorHelpers) -----------------------------------

fn vector_dot(ctx: &mut HostCtx<'_>) -> Result<usize, HostError> {
    let a = ctx.check_vector(1)?;
    let b = ctx.check_vector(2)?;
    ctx.push(f64::from(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]))?;
    Ok(1)
}

fn vector_cross(ctx: &mut HostCtx<'_>) -> Result<usize, HostError> {
    let a = ctx.check_vector(1)?;
    let b = ctx.check_vector(2)?;
    ctx.push([
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ])?;
    Ok(1)
}

/// The vector metatable: `__index` (`Magnitude`, `Unit`, `Dot`) and
/// `__namecall` (`Dot`, `Cross`), read-only.
fn vector(vm: &mut Vm) -> Result<(), Error> {
    let mt = vm.create_table()?;
    let index = vm.create_host_function("", |ctx| {
        let v = ctx.check_vector(1)?;
        let name: String = ctx.get(2)?;
        let magnitude = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        match name.as_str() {
            "Magnitude" => ctx.push(f64::from(magnitude))?,
            "Unit" => {
                let inv = 1.0 / magnitude;
                ctx.push([v[0] * inv, v[1] * inv, v[2] * inv])?;
            }
            "Dot" => {
                let dot = ctx.create_host_function("Dot", vector_dot)?;
                ctx.push(dot)?;
            }
            _ => return Err(HostError::new(format!("{name} is not a valid member of vector"))),
        }
        Ok(1)
    })?;
    mt.raw_set(vm, "__index", index)?;
    let namecall = vm.create_host_function("", |ctx| {
        match ctx.namecall_method().as_deref() {
            Some(b"Dot") => return vector_dot(ctx),
            Some(b"Cross") => return vector_cross(ctx),
            _ => {}
        }
        let name: String = ctx.get(1)?;
        Err(HostError::new(format!("{name} is not a valid method of vector")))
    })?;
    mt.raw_set(vm, "__namecall", namecall)?;
    mt.set_readonly(vm, true)?;
    vm.set_named_registry_value("vector", &mt)?;
    vm.set_vector_metatable(Some(&mt))
}

// ---- udata_direct.luau, native_userdata.luau (setupUserdataHelpers) -----

const TAG_VEC2: u8 = 12;
const TAG_VERTEX: u8 = 13;

/// C++ `Vec2` (`ConformanceIrHooks.h`).
#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

/// C++ `Vertex`.
#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

fn borrowed() -> HostError {
    HostError::raw("userdata already borrowed")
}

/// `lua_vec2_get`: `luaL_checkudatatagged(idx, kTagVec2)`.
fn vec2_get(ctx: &mut HostCtx<'_>, n: usize) -> Result<Vec2, HostError> {
    match ctx.to_userdata_tagged::<Vec2>(n, TAG_VEC2) {
        Some(cell) => cell.try_borrow().map(|v| *v).map_err(|_| borrowed()),
        None => Err(ctx.type_error(n, "userdata")),
    }
}

fn vec2_cell(ctx: &mut HostCtx<'_>, n: usize) -> Result<std::rc::Rc<std::cell::RefCell<Vec2>>, HostError> {
    match ctx.to_userdata_tagged::<Vec2>(n, TAG_VEC2) {
        Some(cell) => Ok(cell),
        None => Err(ctx.type_error(n, "userdata")),
    }
}

/// `lua_vec2_push`: a tagged userdata with the tag's metatable.
fn vec2_push(ctx: &mut HostCtx<'_>, v: Vec2) -> Result<usize, HostError> {
    let ud = ctx.create_userdata_tagged(v, TAG_VEC2)?;
    ctx.push(ud)?;
    Ok(1)
}

/// `lua_vertex_get`: `lua_touserdatatagged`, else `luaL_typeerror(idx,
/// "vertex")`.
fn vertex_cell(ctx: &mut HostCtx<'_>, n: usize) -> Result<std::rc::Rc<std::cell::RefCell<Vertex>>, HostError> {
    match ctx.to_userdata_tagged::<Vertex>(n, TAG_VERTEX) {
        Some(cell) => Ok(cell),
        None => Err(ctx.type_error(n, "vertex")),
    }
}

fn vertex_get(ctx: &mut HostCtx<'_>, n: usize) -> Result<Vertex, HostError> {
    let cell = vertex_cell(ctx, n)?;
    cell.try_borrow().map(|v| *v).map_err(|_| borrowed())
}

fn vertex_push(ctx: &mut HostCtx<'_>, v: Vertex) -> Result<usize, HostError> {
    let ud = ctx.create_userdata_tagged(v, TAG_VERTEX)?;
    ctx.push(ud)?;
    Ok(1)
}

fn vec2_binary(op: Vec2Op) -> impl Fn(&mut HostCtx<'_>) -> Result<usize, HostError> {
    move |ctx| {
        let a = vec2_get(ctx, 1)?;
        let b = vec2_get(ctx, 2)?;
        vec2_push(ctx, Vec2 { x: op(a.x, b.x), y: op(a.y, b.y) })
    }
}

fn userdata_helpers(vm: &mut Vm) -> Result<(), Error> {
    // vec2
    let mt = vm.create_table()?;
    vm.set_tag_metatable(TAG_VEC2, Some(&mt))?;
    mt.raw_set(vm, "__type", "vec2")?;
    let index = vm.create_host_function("", |ctx| {
        let v = vec2_get(ctx, 1)?;
        let name: String = ctx.get(2)?;
        match name.as_str() {
            "X" => ctx.push(f64::from(v.x))?,
            "Y" => ctx.push(f64::from(v.y))?,
            "Magnitude" => ctx.push(f64::from((v.x * v.x + v.y * v.y).sqrt()))?,
            "Unit" => {
                let inv = 1.0 / (v.x * v.x + v.y * v.y).sqrt();
                return vec2_push(ctx, Vec2 { x: v.x * inv, y: v.y * inv });
            }
            "sizeof" => ctx.push(std::mem::size_of::<Vec2>() as f64)?,
            _ => return Err(HostError::new(format!("{name} is not a valid member of vector"))),
        }
        Ok(1)
    })?;
    mt.raw_set(vm, "__index", index)?;
    let newindex = vm.create_host_function("", |ctx| {
        let cell = vec2_cell(ctx, 1)?;
        let name: String = ctx.get(2)?;
        let value = ctx.check_number(3)? as f32;
        let mut v = cell.try_borrow_mut().map_err(|_| borrowed())?;
        match name.as_str() {
            "X" => v.x = value,
            "Y" => v.y = value,
            _ => return Err(HostError::new(format!("{name} is not a writable member of vec2"))),
        }
        Ok(0)
    })?;
    mt.raw_set(vm, "__newindex", newindex)?;
    let namecall = vm.create_host_function("", |ctx| {
        if let Some(method) = ctx.namecall_method() {
            let this = vec2_get(ctx, 1)?;
            match method.as_slice() {
                b"Dot" => {
                    let b = vec2_get(ctx, 2)?;
                    ctx.push(f64::from(this.x * b.x + this.y * b.y))?;
                    return Ok(1);
                }
                b"Min" => {
                    let b = vec2_get(ctx, 2)?;
                    let min = Vec2 {
                        x: if this.x < b.x { this.x } else { b.x },
                        y: if this.y < b.y { this.y } else { b.y },
                    };
                    return vec2_push(ctx, min);
                }
                b"Clone" => return vec2_push(ctx, this),
                b"Reenter" => {
                    // `lua_pcall(reenterCallback)`: an error is dropped.
                    let globals = ctx.globals()?;
                    let callback: nuxie_luau_vm::Function = globals.get(ctx, "reenterCallback")?;
                    let _ = callback.call::<_, ()>(ctx, ());
                    ctx.push(f64::from(this.x + this.y))?;
                    return Ok(1);
                }
                _ => {}
            }
        }
        let name: String = ctx.get(1)?;
        Err(HostError::new(format!("{name} is not a valid method of vector")))
    })?;
    mt.raw_set(vm, "__namecall", namecall)?;
    let ops: [(&str, Vec2Op); 4] = [
        ("__add", |a, b| a + b),
        ("__sub", |a, b| a - b),
        ("__mul", |a, b| a * b),
        ("__div", |a, b| a / b),
    ];
    for (event, op) in ops {
        let f = vm.create_host_function("", vec2_binary(op))?;
        mt.raw_set(vm, event, f)?;
    }
    let unm = vm.create_host_function("", |ctx| {
        let a = vec2_get(ctx, 1)?;
        vec2_push(ctx, Vec2 { x: -a.x, y: -a.y })
    })?;
    mt.raw_set(vm, "__unm", unm)?;
    mt.set_readonly(vm, true)?;
    let ctor = vm.create_host_function("vec2", |ctx| {
        let x = ctx.check_number(1)?;
        let y = ctx.check_number(2)?;
        vec2_push(ctx, Vec2 { x: x as f32, y: y as f32 })
    })?;
    set_global(vm, "vec2", ctor)?;

    // vertex
    let mt = vm.create_table()?;
    vm.set_tag_metatable(TAG_VERTEX, Some(&mt))?;
    let index = vm.create_host_function("", |ctx| {
        let v = vertex_get(ctx, 1)?;
        let name: String = ctx.get(2)?;
        match name.as_str() {
            "pos" => ctx.push(v.pos)?,
            "normal" => ctx.push(v.normal)?,
            "uv" => return vec2_push(ctx, Vec2 { x: v.uv[0], y: v.uv[1] }),
            "sizeof" => ctx.push(std::mem::size_of::<Vertex>() as f64)?,
            _ => return Err(HostError::new(format!("{name} is not a valid member of vertex"))),
        }
        Ok(1)
    })?;
    mt.raw_set(vm, "__index", index)?;
    let newindex = vm.create_host_function("", |ctx| {
        let cell = vertex_cell(ctx, 1)?;
        let name: String = ctx.get(2)?;
        match name.as_str() {
            "pos" => {
                let pos = ctx.check_vector(3)?;
                cell.try_borrow_mut().map_err(|_| borrowed())?.pos = pos;
            }
            "normal" => {
                let normal = ctx.check_vector(3)?;
                cell.try_borrow_mut().map_err(|_| borrowed())?.normal = normal;
            }
            "uv" => {
                let uv = vec2_get(ctx, 3)?;
                cell.try_borrow_mut().map_err(|_| borrowed())?.uv = [uv.x, uv.y];
            }
            _ => return Err(HostError::new(format!("{name} is not a writable member of vertex"))),
        }
        Ok(0)
    })?;
    mt.raw_set(vm, "__newindex", newindex)?;
    let namecall = vm.create_host_function("", |ctx| {
        if let Some(method) = ctx.namecall_method() {
            let this = vertex_get(ctx, 1)?;
            if method == b"Clone" {
                return vertex_push(ctx, this);
            }
        }
        let name: String = ctx.get(1)?;
        Err(HostError::new(format!("{name} is not a valid method of vertex")))
    })?;
    mt.raw_set(vm, "__namecall", namecall)?;
    mt.set_readonly(vm, true)?;
    let ctor = vm.create_host_function("vertex", |ctx| {
        let pos = ctx.check_vector(1)?;
        let normal = ctx.check_vector(2)?;
        let uv = vec2_get(ctx, 3)?;
        vertex_push(ctx, Vertex { pos, normal, uv: [uv.x, uv.y] })
    })?;
    set_global(vm, "vertex", ctor)?;
    // "check that metatables are correctly pinned"
    vm.collect();
    Ok(())
}
