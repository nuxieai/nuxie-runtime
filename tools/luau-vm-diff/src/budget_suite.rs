//! The memory-budget suite (Phase 2, lane 3): every allocating opcode and
//! builtin of `nuxie-luau-vm`, run under a budget too small for what the
//! script asks for.
//!
//! Each case must end in the memory error, returned to the host, with the
//! budget never passed, the accounting exact (`Vm::accounting_drift` is
//! zero) and the VM usable afterwards. See the crate's `budget.rs` for the
//! charging rules these cases check.
//!
//! `every_builtin_is_classified` lists every function the VM's libraries
//! expose and fails on one that is neither covered by a case here nor
//! listed as non-allocating. A lane that adds a library function therefore
//! has to decide which it is, and add a case if it allocates.

use super::{Profile, compile};
use nuxie_luau_vm::{Error, ErrorKind, HostValue, Limits, Vm};

/// A budget large enough for the libraries and the case's setup.
const BUDGET: usize = 1 << 20;

/// The script grows the heap until it runs out; `covers` names the
/// builtins (as `lib.name`, or the base name) and opcodes it exercises.
struct Case {
    covers: &'static [&'static str],
    source: &'static str,
    /// The error the case must end in: the memory error unless a limit
    /// upstream also checks refuses the request before anything grows.
    error: &'static str,
}

const MEMORY: &str = "not enough memory";

const CASES: &[Case] = &[
    // ---- opcodes ----------------------------------------------------------
    Case {
        covers: &["NEWTABLE", "SETTABLEN"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = {} end",
        error: MEMORY,
    },
    Case {
        covers: &["NEWTABLE+SETLIST"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = {i, i, i, i, i, i, i, i} end",
        error: MEMORY,
    },
    Case {
        covers: &["DUPTABLE"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = {a = i, b = i, c = i} end",
        error: MEMORY,
    },
    Case {
        covers: &["SETTABLE"],
        source: "local t = {} for i = 1, 1e7 do t[i * 2 + 0.5] = i end",
        error: MEMORY,
    },
    Case {
        covers: &["SETTABLEKS", "CONCAT(number)"],
        source: r#"local t = {} for i = 1, 1e7 do t["k" .. i] = i end"#,
        error: MEMORY,
    },
    Case {
        covers: &["NEWCLOSURE", "CAPTURE(VAL)"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = function() return i end end",
        error: MEMORY,
    },
    Case {
        covers: &["CAPTURE(REF)"],
        source: "local keep = {}
                 for i = 1, 1e7 do
                     local x = i
                     keep[i] = function() x += 1 return x end
                 end",
        error: MEMORY,
    },
    Case {
        covers: &["CONCAT"],
        source: r#"local s = "ab" for i = 1, 40 do s = s .. s end"#,
        error: MEMORY,
    },
    Case {
        covers: &["CALL(stack)"],
        source: "local function f(n) return 1 + f(n + 1) end return f(1)",
        error: MEMORY,
    },
    Case {
        covers: &["GETVARARGS"],
        source: "local function f(...) return 1 + f(1, 2, 3, 4, ...) end return f()",
        error: MEMORY,
    },
    Case {
        covers: &["NEWTABLE(varargs)"],
        source: "local function f(...) local t = {...} return t end
                 local keep = {} for i = 1, 1e7 do keep[i] = f(i, i, i) end",
        error: MEMORY,
    },
    // ---- base library ------------------------------------------------------
    Case {
        covers: &["tostring"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = tostring(i + 0.5) end",
        error: MEMORY,
    },
    Case {
        covers: &["error"],
        source: r#"local s = string.rep("a", 600 * 1024) error(s)"#,
        error: MEMORY,
    },
    Case {
        covers: &["assert"],
        source: r#"local s = string.rep("a", 600 * 1024) assert(false, s)"#,
        error: MEMORY,
    },
    Case {
        covers: &["rawset"],
        source: "local t = {} for i = 1, 1e7 do rawset(t, i, i) end",
        error: MEMORY,
    },
    Case {
        covers: &["unpack", "table.unpack"],
        source: "local t = {} return select('#', unpack(t, 1, 500000))",
        // C++ `luaL_checkstack` refuses before the stack grows.
        error: "=budget:1: too many results to unpack",
    },
    // ---- string library ------------------------------------------------------
    Case {
        covers: &["string.rep"],
        source: r#"return string.rep("x", 2 * 1024 * 1024)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.byte"],
        source: r#"local s = string.rep("a", 300 * 1024) return string.byte(s, 1, -1)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.char"],
        source: "local codes = {}
                 for i = 1, 4096 do codes[i] = 65 end
                 local keep = {}
                 for i = 1, 1e7 do
                     codes[1], codes[2], codes[3] = i % 256, (i // 256) % 256, (i // 65536) % 256
                     keep[i] = string.char(unpack(codes))
                 end",
        error: MEMORY,
    },
    Case {
        covers: &["string.find"],
        source: r#"local s = string.rep("ab", 300 * 1024) return string.find(s, "b(.*)")"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.match"],
        source: r#"local s = string.rep("ab", 300 * 1024) return string.match(s, "b(.*)")"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.format"],
        source: r#"local s = string.rep("a", 600 * 1024) return string.format("%s!", s)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.gsub"],
        source: r#"local s = string.rep("a", 600 * 1024) return string.gsub(s, "a", "b")"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.lower"],
        source: r#"local s = string.rep("A", 600 * 1024) return string.lower(s)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.upper"],
        source: r#"local s = string.rep("a", 600 * 1024) return string.upper(s)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.reverse"],
        source: r#"local s = string.rep("a", 600 * 1024) .. "b" return string.reverse(s)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.sub"],
        source: r#"local s = string.rep("a", 600 * 1024) return string.sub(s, 2)"#,
        error: MEMORY,
    },
    Case {
        covers: &["string.split"],
        source: r#"local s = string.rep("ab,", 100000)
                   local keep = {} for i = 1, 1e7 do keep[i] = string.split(s) end"#,
        error: MEMORY,
    },
    Case {
        // Each iterator is a builtin closure with three upvalues.
        covers: &["string.gmatch"],
        source: r#"local keep = {} for i = 1, 1e7 do keep[i] = string.gmatch("abc", ".") end"#,
        error: MEMORY,
    },
    // ---- table library ---------------------------------------------------------
    Case {
        covers: &["table.concat"],
        source: r#"local s = string.rep("a", 600 * 1024) return table.concat({s, s})"#,
        error: MEMORY,
    },
    Case {
        covers: &["table.insert"],
        source: "local t = {} for i = 1, 1e7 do table.insert(t, i) end",
        error: MEMORY,
    },
    Case {
        covers: &["table.create"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = table.create(100, i) end",
        error: MEMORY,
    },
    Case {
        covers: &["table.clone"],
        source: "local t = table.create(1000, 1)
                 local keep = {} for i = 1, 1e7 do keep[i] = table.clone(t) end",
        error: MEMORY,
    },
    Case {
        covers: &["table.move"],
        source: "local src, dst = table.create(1000, 1), {}
                 for i = 0, 1e7 do table.move(src, 1, 1000, i * 1000 + 1, dst) end",
        error: MEMORY,
    },
    Case {
        // The sparse path: a temporary table of the entries in range, then
        // the destination's growth.
        covers: &["table.move(sparse)"],
        source: "local src = {} for i = 1, 2000 do src[i * 1000] = i end
                 local keep = {} for i = 1, 1e7 do keep[i] = table.move(src, 1, 2^30, 5, {}) end",
        error: MEMORY,
    },
    Case {
        covers: &["table.pack"],
        source: "local keep = {} for i = 1, 1e7 do keep[i] = table.pack(i, i, i, i) end",
        error: MEMORY,
    },
    // ---- coroutine library (lane 4) -------------------------------------------
    // A thread is a fixed part plus its stack, an external buffer; both are
    // charged before they exist (`Heap::new_thread`, `Vm::ensure_stack`).
    Case {
        covers: &["coroutine.create"],
        source: "local f = function() end
                 local keep = {} for i = 1, 1e7 do keep[i] = coroutine.create(f) end",
        error: MEMORY,
    },
    Case {
        covers: &["coroutine.wrap"],
        source: "local f = function() end
                 local keep = {} for i = 1, 1e7 do keep[i] = coroutine.wrap(f) end",
        error: MEMORY,
    },
    Case {
        // Arguments move onto each coroutine's stack and yielded values
        // back; the suspended coroutines keep their grown stacks.
        covers: &["coroutine.resume", "coroutine.yield"],
        source: "local args = {} for i = 1, 200 do args[i] = i end
                 local keep = {}
                 for i = 1, 1e7 do
                     local co = coroutine.create(function(...) coroutine.yield(...) end)
                     coroutine.resume(co, unpack(args))
                     keep[i] = co
                 end",
        error: MEMORY,
    },
    Case {
        // A coroutine's own stack growth: each one recurses until the
        // budget refuses inside it (resume returns false), and the dead
        // threads keep their stacks until the main thread runs out.
        covers: &["CALL(coroutine stack)"],
        source: "local function deep(n) return 1 + deep(n + 1) end
                 local keep = {}
                 for i = 1, 1e7 do
                     local co = coroutine.create(function() return deep(1) end)
                     coroutine.resume(co)
                     keep[i] = co
                 end",
        error: MEMORY,
    },
];

/// Library functions that allocate nothing in proportion to their
/// arguments: they read, compare, convert numbers, return an existing value
/// or an interned type name, or re-enter the VM (whose allocations the
/// cases above cover).
const NON_ALLOCATING: &[&str] = &[
    "gcinfo",
    "getfenv",
    "getmetatable",
    "ipairs",
    "next",
    "pairs",
    "pcall",
    "rawequal",
    "rawget",
    "rawlen",
    "select",
    "setfenv",
    "setmetatable",
    "tonumber",
    "type",
    "typeof",
    "xpcall",
    "string.len",
    "table.remove",
    "table.sort",
    // Lane 2: clear keeps both parts' capacity; find, getn, maxn and
    // isfrozen read; freeze sets a flag; foreach and foreachi re-enter.
    "table.clear",
    "table.find",
    "table.foreach",
    "table.foreachi",
    "table.freeze",
    "table.getn",
    "table.isfrozen",
    "table.maxn",
    // Coroutine queries, and `close`, which frees a thread's stack.
    "coroutine.close",
    "coroutine.isyieldable",
    "coroutine.running",
    "coroutine.status",
];

/// Every `math` function computes on numbers.
const NON_ALLOCATING_LIBRARIES: &[&str] = &["math"];

fn compiled(source: &str) -> Vec<u8> {
    compile(source, Profile::Product).unwrap_or_else(|e| panic!("{e}\n{source}"))
}

fn drift(vm: &Vm) -> i128 {
    vm.accounting_drift()
}

/// Runs one case; `Err` says what went wrong.
fn check(case: &Case) -> Result<(), String> {
    let mut vm = Vm::with_limits(Limits {
        max_memory_bytes: Some(BUDGET),
        ..Limits::default()
    })
    .map_err(|e| format!("vm: {e:?}"))?;
    let result = vm.run(b"=budget", &compiled(case.source));
    let kind = if case.error == MEMORY {
        ErrorKind::Memory
    } else {
        ErrorKind::Runtime
    };
    match &result {
        Err(Error::Lua { kind: k, message }) if *k == kind && message == case.error.as_bytes() => {}
        Err(other) => return Err(format!("expected {:?}, got {other:?}", case.error)),
        Ok(values) => {
            return Err(format!(
                "expected {:?}, got {} returned values",
                case.error,
                values.len()
            ));
        }
    }
    if vm.memory_bytes() > BUDGET {
        return Err(format!(
            "{} bytes in use, budget {BUDGET}",
            vm.memory_bytes()
        ));
    }
    if drift(&vm) != 0 {
        return Err(format!("accounting drift {} after the error", drift(&vm)));
    }
    vm.collect();
    if drift(&vm) != 0 {
        return Err(format!(
            "accounting drift {} after a collection",
            drift(&vm)
        ));
    }
    match vm.run(b"=again", &compiled("return 1 + 1")) {
        Ok(values) if values == vec![HostValue::Number(2.0)] => Ok(()),
        other => Err(format!("not usable after the error: {other:?}")),
    }
}

#[test]
fn every_allocating_case_ends_in_the_memory_error_within_budget() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| {
            check(case)
                .err()
                .map(|why| format!("{}: {why}", case.covers.join(", ")))
        })
        .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn collection_never_allocates() {
    let mut vm = Vm::new().expect("vm");
    vm.register("setblockallocations", |ctx| {
        let on = matches!(ctx.arg(1), Some(HostValue::Boolean(true)));
        ctx.set_block_allocations(on);
        Ok(0)
    })
    .expect("register");
    vm.register("collect", |ctx| {
        ctx.collect();
        Ok(0)
    })
    .expect("register");
    let returned = vm
        .run(
            b"=blocked",
            &compiled(
                r#"local weak = setmetatable({}, {__mode = "k"})
                   for i = 1, 20000 do weak[{}] = i end
                   for i = 1, 100 do weak[tostring(i)] = {} end
                   setblockallocations(true)
                   collect()
                   setblockallocations(false)
                   local n = 0
                   for k in pairs(weak) do n += 1 end
                   return n"#,
            ),
        )
        .expect("a collection with allocations blocked succeeds");
    assert_eq!(returned, vec![HostValue::Number(100.0)]);
    assert_eq!(drift(&vm), 0);
}

#[test]
fn every_builtin_is_classified() {
    let mut vm = Vm::new().expect("vm");
    let returned = vm
        .run(
            b"=names",
            &compiled(
                r#"local names = {}
                   for k, v in pairs(_G) do
                       if type(v) == "function" then
                           table.insert(names, k)
                       elseif type(v) == "table" and k ~= "_G" then
                           for f, g in pairs(v) do
                               if type(g) == "function" then
                                   table.insert(names, k .. "." .. f)
                               end
                           end
                       end
                   end
                   table.sort(names)
                   return table.concat(names, " ")"#,
            ),
        )
        .expect("lists the libraries");
    let Some(HostValue::String(names)) = returned.first() else {
        panic!("{returned:?}");
    };
    let names = String::from_utf8_lossy(names).into_owned();
    let covered: Vec<&str> = CASES
        .iter()
        .flat_map(|c| c.covers.iter().copied())
        .collect();
    let unclassified: Vec<&str> = names
        .split(' ')
        .filter(|name| {
            let library = name.split_once('.').map(|(lib, _)| lib);
            !covered.contains(name)
                && !NON_ALLOCATING.contains(name)
                && !library.is_some_and(|lib| NON_ALLOCATING_LIBRARIES.contains(&lib))
        })
        .collect();
    assert!(
        unclassified.is_empty(),
        "library functions with no budget case and not listed as non-allocating \
         (tools/luau-vm-diff/src/budget_suite.rs): {unclassified:?}"
    );
}
