//! The VM's libraries and the argument helpers they share (C++
//! `lauxlib.cpp`).
//!
//! Lane 1 ships the base library (`base.rs`) and the few `string`,
//! `table` and `math` functions the lane-1 conformance targets and the
//! differential corpus call (`string.rs`, `table.rs`, `math.rs`). Seam for
//! lane 5: each library is a module with an `open` function; completing
//! one is adding functions to its table.

pub(crate) mod base;
pub(crate) mod math;
pub(crate) mod string;
pub(crate) mod table;

use std::rc::Rc;

use crate::error::{Throw, VmResult};
use crate::heap::{Function, NativeClosure};
use crate::native::{Args, BuiltinFn, NativeFn};
use crate::value::{StrId, TableId, Value};
use crate::vm::Vm;

/// C++ `LUAI_MAXCSTACK`: the most slots a native function may claim with
/// `lua_checkstack`.
pub(crate) const LUAI_MAXCSTACK: usize = 8000;

impl Vm {
    /// Argument `n` (1-based), or `None` if it was not passed (C++
    /// `LUA_TNONE`).
    #[inline]
    pub(crate) fn arg(&self, args: Args, n: usize) -> Option<Value> {
        args.slot(n.checked_sub(1)?).map(|slot| self.get(slot))
    }

    /// The value of argument `n`, nil when absent.
    #[inline]
    pub(crate) fn arg_or_nil(&self, args: Args, n: usize) -> Value {
        self.arg(args, n).unwrap_or_default()
    }

    /// C++ `currfuncname`: the running native's debug name, or the method
    /// name for a `__namecall` dispatcher.
    fn current_func_name(&self) -> Option<String> {
        let frame = self.frames.last()?;
        let Function::Native(native) = self.heap.function(frame.closure) else {
            return None;
        };
        if native.name.is_empty() {
            return None;
        }
        if &*native.name == "__namecall" {
            return self
                .namecall
                .map(|id| crate::vm::c_lossy(self.heap.str_bytes(id)));
        }
        Some(native.name.to_string())
    }

    /// C++ `luaL_argerror`.
    pub(crate) fn arg_error(&mut self, n: usize, message: &str) -> Throw {
        let text = match self.current_func_name() {
            Some(name) => format!("invalid argument #{n} to '{name}' ({message})"),
            None => format!("invalid argument #{n} ({message})"),
        };
        self.lib_error(text)
    }

    /// C++ `luaL_typeerror`.
    pub(crate) fn arg_type_error(&mut self, args: Args, n: usize, expected: &str) -> Throw {
        let name = self.current_func_name();
        let text = match (self.arg(args, n), name) {
            (Some(v), Some(name)) => {
                format!(
                    "invalid argument #{n} to '{name}' ({expected} expected, got {})",
                    self.obj_type_name(v)
                )
            }
            (Some(v), None) => format!(
                "invalid argument #{n} ({expected} expected, got {})",
                self.obj_type_name(v)
            ),
            (None, Some(name)) => {
                format!("missing argument #{n} to '{name}' ({expected} expected)")
            }
            (None, None) => format!("missing argument #{n} ({expected} expected)"),
        };
        self.lib_error(text)
    }

    /// C++ `luaL_checkany`.
    pub(crate) fn check_any(&mut self, args: Args, n: usize) -> VmResult<Value> {
        match self.arg(args, n) {
            Some(v) => Ok(v),
            None => Err(self.lib_error(format!("missing argument #{n}"))),
        }
    }

    pub(crate) fn check_table(&mut self, args: Args, n: usize) -> VmResult<TableId> {
        match self.arg(args, n) {
            Some(Value::Table(id)) => Ok(id),
            _ => Err(self.arg_type_error(args, n, "table")),
        }
    }

    /// C++ `luaL_checknumber`: numbers and numeric strings.
    pub(crate) fn check_number(&mut self, args: Args, n: usize) -> VmResult<f64> {
        match self.arg(args, n).and_then(|v| self.to_number(v)) {
            Some(x) => Ok(x),
            None => Err(self.arg_type_error(args, n, "number")),
        }
    }

    /// C++ `luaL_checkinteger`: a number truncated to `int`.
    pub(crate) fn check_integer(&mut self, args: Args, n: usize) -> VmResult<i32> {
        Ok(self.check_number(args, n)? as i32)
    }

    pub(crate) fn opt_integer(&mut self, args: Args, n: usize, default: i32) -> VmResult<i32> {
        if self.arg_or_nil(args, n).is_nil() {
            Ok(default)
        } else {
            self.check_integer(args, n)
        }
    }

    /// C++ `luaL_checklstring`: strings, and numbers converted in place.
    pub(crate) fn check_string(&mut self, args: Args, n: usize) -> VmResult<StrId> {
        let Some(v) = self.arg(args, n) else {
            return Err(self.arg_type_error(args, n, "string"));
        };
        match self.coerce_to_str(v)? {
            Some(id) => {
                if let Some(slot) = args.slot(n.saturating_sub(1)) {
                    self.set(slot, Value::String(id));
                }
                Ok(id)
            }
            None => Err(self.arg_type_error(args, n, "string")),
        }
    }

    pub(crate) fn opt_string(&mut self, args: Args, n: usize) -> VmResult<Option<StrId>> {
        if self.arg_or_nil(args, n).is_nil() {
            Ok(None)
        } else {
            self.check_string(args, n).map(Some)
        }
    }

    /// C++ `lua_checkstack` from a builtin whose arguments start at
    /// `args.base`: room for `size` more values, refused past
    /// `LUAI_MAXCSTACK` slots in the frame or when the stack cannot grow.
    pub(crate) fn check_stack(&mut self, args: Args, size: usize) -> bool {
        let used = self.top.saturating_sub(args.base);
        if size > LUAI_MAXCSTACK || used.saturating_add(size) > LUAI_MAXCSTACK {
            return false;
        }
        self.ensure_stack(self.top.saturating_add(size)).is_ok()
    }

    /// Pushes results for a builtin and returns their count.
    pub(crate) fn ret(&mut self, values: &[Value]) -> VmResult<usize> {
        for &v in values {
            self.push(v)?;
        }
        Ok(values.len())
    }

    pub(crate) fn ret_str(&mut self, bytes: &[u8]) -> VmResult<usize> {
        let id = self.heap.intern(bytes)?;
        self.ret(&[Value::String(id)])
    }

    /// A native closure value.
    pub(crate) fn new_builtin(
        &mut self,
        name: &str,
        f: BuiltinFn,
        upvals: &[Value],
    ) -> VmResult<Value> {
        let func = if upvals.is_empty() {
            NativeFn::Builtin(f)
        } else {
            NativeFn::BuiltinWith(f, Rc::from(upvals))
        };
        let id = self.heap.new_function(Function::Native(NativeClosure {
            func,
            name: Rc::from(name),
        }))?;
        Ok(Value::Function(id))
    }

    /// The upvalue `i` of the running builtin.
    pub(crate) fn builtin_upval(&self, i: usize) -> Value {
        let Some(frame) = self.frames.last() else {
            return Value::Nil;
        };
        match self.heap.function(frame.closure) {
            Function::Native(NativeClosure {
                func: NativeFn::BuiltinWith(_, ups),
                ..
            }) => ups.get(i).copied().unwrap_or_default(),
            _ => Value::Nil,
        }
    }

    /// Replaces upvalue `i` of the running builtin (C++ `lua_replace` on
    /// `lua_upvalueindex(i + 1)`), for builtins that keep state between
    /// calls (`gmatch`'s position).
    pub(crate) fn set_builtin_upval(&mut self, i: usize, value: Value) {
        let Some(frame) = self.frames.last() else {
            return;
        };
        let closure = frame.closure;
        if let Function::Native(NativeClosure {
            func: NativeFn::BuiltinWith(_, ups),
            ..
        }) = self.heap.function_mut(closure)
        {
            // `function_mut` passes the write barrier. The upvalue count
            // never changes, so the replacement costs nothing beyond what
            // `new_function` charged; the copy keeps any clone a running
            // call holds unchanged.
            let mut values = ups.to_vec();
            if let Some(slot) = values.get_mut(i) {
                *slot = value;
                *ups = Rc::from(values);
            }
        }
    }

    /// Registers `functions` into `table` (C++ `luaL_register`).
    pub(crate) fn register_builtins(
        &mut self,
        table: TableId,
        functions: &[(&str, BuiltinFn)],
    ) -> VmResult<()> {
        for &(name, f) in functions {
            let v = self.new_builtin(name, f, &[])?;
            self.raw_set_str(table, name, v)?;
        }
        Ok(())
    }

    /// A library table registered as a global.
    pub(crate) fn open_lib(
        &mut self,
        name: &str,
        functions: &[(&str, BuiltinFn)],
    ) -> VmResult<TableId> {
        let t = self.new_table(0, functions.len())?;
        self.register_builtins(t, functions)?;
        let globals = self.globals;
        self.raw_set_str(globals, name, Value::Table(t))?;
        Ok(t)
    }
}

/// Fails with the memory error when a library result buffer of `len` bytes
/// would pass `Limits::max_string_bytes` or the memory budget. Builders
/// (`gsub`, `format`, `table.concat`) check before they grow, so a result
/// that will be refused is never built.
pub(crate) fn check_buffer(vm: &Vm, len: usize) -> VmResult<()> {
    if len > vm.heap.string_limit {
        return Err(vm.heap.memory_error());
    }
    vm.heap.check_alloc(len)
}

/// Appends `bytes` to a library result buffer after [`check_buffer`].
pub(crate) fn append(vm: &Vm, out: &mut Vec<u8>, bytes: &[u8]) -> VmResult<()> {
    check_buffer(vm, out.len().saturating_add(bytes.len()))?;
    out.extend_from_slice(bytes);
    Ok(())
}
