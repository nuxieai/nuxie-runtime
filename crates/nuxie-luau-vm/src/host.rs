//! The host API: what an embedder (the conformance harness, the
//! differential tool, the bench) calls.
//!
//! The host never holds an object reference. Values cross the boundary as
//! [`HostValue`], plain data copied out of the VM; objects show only their
//! type name. A host function sees its arguments through [`HostCtx`], whose
//! slots are on the VM stack and therefore rooted (D2).
//!
//! Seam for lane 6 (embedding): userdata, persistent references
//! (`lua_ref`), and calling Lua functions held by the host are added here,
//! as slot- or registry-index-based handles; the rest of the crate does not
//! change.

use std::rc::Rc;

use crate::call::{CallMode, Exit, Saved};
use crate::error::{Error, LoadError, Throw};
use crate::heap::{CollectStats, Function, NativeClosure};
use crate::limits::Limits;
use crate::native::{Args, HostFn, NativeFn};
use crate::thread::ThreadStatus;
use crate::value::{TableId, Value};
use crate::vm::{MULTRET, Tm, Vm};

/// Where a chunk run with [`Vm::start`] or [`Vm::resume`] stopped.
#[derive(Clone, Debug, PartialEq)]
pub enum RunState {
    /// It returned these values.
    Finished(Vec<HostValue>),
    /// It yielded these values; [`Vm::resume`] continues it.
    Yielded(Vec<HostValue>),
}

/// A value copied out of the VM.
#[derive(Clone, Debug, PartialEq)]
pub enum HostValue {
    Nil,
    Boolean(bool),
    Number(f64),
    Vector([f32; 3]),
    String(Vec<u8>),
    /// A table, function or other object, by type name only.
    Object(&'static str),
}

impl HostValue {
    /// The Luau type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            HostValue::Nil => "nil",
            HostValue::Boolean(_) => "boolean",
            HostValue::Number(_) => "number",
            HostValue::Vector(_) => "vector",
            HostValue::String(_) => "string",
            HostValue::Object(name) => name,
        }
    }
}

/// An error a host function raises.
pub struct HostError(pub(crate) HostErrorKind);

pub(crate) enum HostErrorKind {
    /// Raised like `luaL_error`: prefixed with the calling Lua position.
    Message(String),
    /// Raised with the message as the error value, unprefixed.
    Raw(String),
    /// A Lua error the host received from the VM, propagated.
    Throw(Throw),
}

impl HostError {
    /// An error prefixed with the calling script position (`luaL_error`).
    pub fn new(message: impl Into<String>) -> HostError {
        HostError(HostErrorKind::Message(message.into()))
    }

    /// An error whose value is exactly `message`.
    pub fn raw(message: impl Into<String>) -> HostError {
        HostError(HostErrorKind::Raw(message.into()))
    }
}

impl std::fmt::Debug for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            HostErrorKind::Message(m) => write!(f, "HostError({m:?})"),
            HostErrorKind::Raw(m) => write!(f, "HostError::raw({m:?})"),
            HostErrorKind::Throw(_) => f.write_str("HostError(<lua error>)"),
        }
    }
}

/// A host function's view of its call: arguments in, results out.
pub struct HostCtx<'a> {
    pub(crate) vm: &'a mut Vm,
    pub(crate) args: Args,
}

impl HostCtx<'_> {
    fn throw(&mut self, throw: Throw) -> HostError {
        self.vm.host_errors.push(throw.value);
        HostError(HostErrorKind::Throw(throw))
    }

    /// The number of arguments.
    pub fn arg_count(&self) -> usize {
        self.args.nargs
    }

    /// Argument `n` (1-based) copied out; `None` when it was not passed.
    pub fn arg(&self, n: usize) -> Option<HostValue> {
        self.vm.arg(self.args, n).map(|v| self.vm.host_value(v))
    }

    /// `luaL_checklstring`: a string or number argument as bytes.
    pub fn check_string(&mut self, n: usize) -> Result<Vec<u8>, HostError> {
        match self.vm.check_string(self.args, n) {
            Ok(id) => Ok(self.vm.heap.str_bytes(id).to_vec()),
            Err(t) => Err(self.throw(t)),
        }
    }

    /// `luaL_optlstring` without a default: `None` for nil or absent.
    pub fn opt_string(&mut self, n: usize) -> Result<Option<Vec<u8>>, HostError> {
        match self.vm.opt_string(self.args, n) {
            Ok(id) => Ok(id.map(|id| self.vm.heap.str_bytes(id).to_vec())),
            Err(t) => Err(self.throw(t)),
        }
    }

    /// `luaL_optinteger`.
    pub fn opt_integer(&mut self, n: usize, default: i32) -> Result<i32, HostError> {
        self.vm
            .opt_integer(self.args, n, default)
            .map_err(|t| self.throw(t))
    }

    /// `luaL_tolstring`: argument `n` rendered as `tostring` would.
    ///
    /// A failing `__tostring` is unwound before its error is returned, as
    /// `pcall` unwinds, so a host function that handles the error and
    /// returns normally leaves the VM as it found it.
    pub fn tostring_value(&mut self, n: usize) -> Result<Vec<u8>, HostError> {
        let v = self.vm.arg_or_nil(self.args, n);
        let (depth, top, saved) = (self.vm.frames.len(), self.vm.top, self.vm.saved_calls());
        match self.vm.tostring_value(v) {
            Ok(id) => Ok(self.vm.heap.str_bytes(id).to_vec()),
            Err(t) => {
                self.vm.unwind(depth, top, saved);
                self.vm.top = top;
                Err(self.throw(t))
            }
        }
    }

    /// Pushes a result.
    pub fn push(&mut self, value: HostValue) -> Result<(), HostError> {
        let v = match self.vm.value_from_host(&value) {
            Ok(v) => v,
            Err(t) => return Err(self.throw(t)),
        };
        self.vm.push(v).map_err(|t| self.throw(t))
    }

    /// Pushes a copy of argument `n` (objects included).
    pub fn push_arg(&mut self, n: usize) -> Result<(), HostError> {
        let v = self.vm.arg_or_nil(self.args, n);
        self.vm.push(v).map_err(|t| self.throw(t))
    }

    /// Loads a bytecode chunk into the running thread's global environment
    /// and pushes the function (C++ `luau_load` with `env` 0). A rejected
    /// chunk is returned as `Ok(Err(..))` and pushes nothing.
    pub fn load(
        &mut self,
        chunk_name: &[u8],
        bytecode: &[u8],
    ) -> Result<Result<(), LoadError>, HostError> {
        let env = self.vm.globals;
        match crate::bytecode::load::load(self.vm, chunk_name, bytecode, env) {
            Ok(f) => {
                self.vm
                    .push(Value::Function(f))
                    .map_err(|t| self.throw(t))?;
                Ok(Ok(()))
            }
            Err(Error::Load(e)) => Ok(Err(e)),
            Err(_) => {
                let memory = self.vm.heap.memory_error();
                Err(self.throw(memory))
            }
        }
    }

    /// A full collection.
    pub fn collect(&mut self) -> CollectStats {
        self.vm.full_gc()
    }

    /// Approximate heap bytes in use.
    pub fn memory_bytes(&self) -> usize {
        self.vm.heap.total_bytes()
    }

    /// Stops or restarts automatic collection.
    pub fn set_gc_running(&mut self, running: bool) {
        self.vm.set_gc_running(running);
    }

    pub fn gc_running(&self) -> bool {
        self.vm.gc_running()
    }

    /// `lua_gc(LUA_GCSTEP, kb)`: about `kb` KiB worth of collector work;
    /// true when a cycle finished.
    pub fn gc_step(&mut self, kb: i32) -> bool {
        self.vm.gc_step(kb)
    }

    /// `lua_gc(LUA_GCSETGOAL / SETSTEPMUL / SETSTEPSIZE, value)`: sets the
    /// heap goal (percent), step multiplier (percent) or step size (KiB)
    /// and returns the previous value.
    pub fn gc_tune(&mut self, option: GcTune, value: i32) -> i32 {
        self.vm.gc_tune(option, value)
    }

    /// While `blocked`, every allocation fails with the memory error. Test
    /// hook (the conformance harness's `setblockallocations`).
    pub fn set_block_allocations(&mut self, blocked: bool) {
        self.vm.heap.set_blocked(blocked);
    }
}

/// A collector tuning parameter for [`HostCtx::gc_tune`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcTune {
    /// Heap goal, percent (C++ `gcgoal`, default 200).
    Goal,
    /// Work per step, percent of the step size (C++ `gcstepmul`, default 200).
    StepMul,
    /// Step size in KiB (C++ `gcstepsize >> 10`, default 1).
    StepSize,
}

impl Vm {
    /// A VM with default limits and the libraries opened.
    pub fn new() -> Result<Vm, Error> {
        Vm::with_limits(Limits::default())
    }

    /// A VM with the given limits and the libraries opened.
    pub fn with_limits(limits: Limits) -> Result<Vm, Error> {
        let mut vm = Vm::with_limits_bare(limits).map_err(|t| Error::Lua {
            kind: t.kind,
            message: b"not enough memory".to_vec(),
        })?;
        let opened = vm.open_libs();
        vm.host_result(opened)?;
        Ok(vm)
    }

    fn open_libs(&mut self) -> crate::error::VmResult<()> {
        crate::libs::base::open(self)?;
        crate::libs::coroutine::open(self)?;
        crate::libs::string::open(self)?;
        crate::libs::strpack::open(self)?;
        crate::libs::table::open(self)?;
        crate::libs::math::open(self)?;
        crate::libs::bit32::open(self)?;
        crate::libs::buffer::open(self)?;
        crate::libs::vector::open(self)?;
        crate::libs::utf8::open(self)?;
        crate::libs::os::open(self)?;
        crate::libs::debug::open(self)?;
        Ok(())
    }

    /// The limits this VM enforces.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Registers a global host function.
    pub fn register(
        &mut self,
        name: &str,
        f: impl Fn(&mut HostCtx<'_>) -> Result<usize, HostError> + 'static,
    ) -> Result<(), Error> {
        let host: Rc<HostFn> = Rc::new(f);
        let result = (|| {
            let id = self.heap.new_function(Function::Native(NativeClosure {
                func: NativeFn::Host(host),
                name: Rc::from(name),
            }))?;
            let globals = self.globals;
            self.raw_set_str(globals, name, Value::Function(id))
        })();
        // After rendering any error (its value is not rooted): the new value
        // is rooted in the globals, whatever it replaced may be garbage, and
        // the host may call this in a loop.
        let result = self.host_result(result);
        self.safepoint();
        result
    }

    /// Sets a global to a plain value.
    pub fn set_global(&mut self, name: &str, value: HostValue) -> Result<(), Error> {
        let result = (|| {
            let v = self.value_from_host(&value)?;
            let globals = self.globals;
            self.raw_set_str(globals, name, v)
        })();
        let result = self.host_result(result);
        self.safepoint();
        result
    }

    /// C++ `luaL_sandbox` then `luaL_sandboxthread`: library tables, the
    /// string metatable and the globals become read-only and safe; the
    /// thread then gets a fresh writable global table that reads through to
    /// them, and `_G` names it.
    pub fn sandbox(&mut self) -> Result<(), Error> {
        let result = (|| {
            let globals = self.globals;
            let mut cursor = 0usize;
            let mut libs: Vec<TableId> = Vec::new();
            while let Some((next, _, v)) = self.heap.table(globals).next_from(cursor) {
                if let Value::Table(t) = v {
                    libs.push(t);
                }
                cursor = next;
            }
            for t in libs {
                self.heap.table_mut(t).readonly = true;
            }
            if let Some(mt) = self.string_meta {
                self.heap.table_mut(mt).readonly = true;
            }
            self.heap.table_mut(globals).readonly = true;
            self.heap.table_mut(globals).safeenv = true;

            let proxy = self.new_table(0, 0)?;
            let meta = self.new_table(0, 1)?;
            let index = self.tm(Tm::Index);
            self.raw_set(meta, Value::String(index), Value::Table(globals))?;
            self.heap.table_mut(meta).readonly = true;
            self.heap.table_mut(proxy).meta = Some(meta);
            self.globals = proxy;
            self.heap.table_mut(proxy).safeenv = true;
            self.raw_set_str(proxy, "_G", Value::Table(proxy))
        })();
        self.host_result(result)
    }

    /// Loads a chunk into the global environment and runs it with no
    /// arguments; returns every result. Like C++ `lua_pcall` from the host,
    /// the chunk runs at C-call level 1 and cannot yield
    /// (`coroutine.isyieldable()` is false; `coroutine.yield` raises).
    pub fn run(&mut self, chunk_name: &[u8], bytecode: &[u8]) -> Result<Vec<HostValue>, Error> {
        let f = self.load_main(chunk_name, bytecode)?;
        let result = (|| {
            self.push(Value::Function(f))?;
            self.call_value(0, 0, MULTRET)
        })();
        let outcome = match result {
            Ok(()) => Ok(self.take_results(0)),
            Err(throw) => {
                let error = self.render_error(throw);
                self.reset_main();
                Err(error)
            }
        };
        // A safepoint at the host boundary: a chunk that never reaches one
        // (`return 1`) still leaves its prototype and closure behind. The
        // results are copied out, so nothing on the stack is live.
        self.safepoint();
        outcome
    }

    /// Loads a chunk and runs it as C++ `lua_resume` runs the main thread:
    /// the chunk may yield to the host (`coroutine.yield` at any depth the
    /// thread can yield from), and [`Vm::resume`] continues it.
    pub fn start(&mut self, chunk_name: &[u8], bytecode: &[u8]) -> Result<RunState, Error> {
        let f = self.load_main(chunk_name, bytecode)?;
        let result = (|| {
            self.push(Value::Function(f))?;
            self.enter_resumed();
            self.push_call(0, 0, MULTRET, CallMode::Plain)?;
            if let Some(frame) = self.frames.first_mut() {
                frame.thread_base = true;
            }
            self.execute_from(0)
        })();
        self.finish_resume(result)
    }

    /// Continues a chunk that yielded to the host, with `args` as the
    /// results of its `coroutine.yield`.
    pub fn resume(&mut self, args: &[HostValue]) -> Result<RunState, Error> {
        let main = self.main_thread;
        if self.current != main || self.heap.thread(main).status != ThreadStatus::Suspended {
            return Err(Error::Host("Vm::resume without a yielded chunk".to_owned()));
        }
        let result = (|| {
            for arg in args {
                let v = self.value_from_host(arg)?;
                self.push(v)?;
            }
            self.enter_resumed();
            match self.complete_yield_from_host(args.len())? {
                Some(exit) => Ok(exit),
                None => self.execute_from(0),
            }
        })();
        self.finish_resume(result)
    }

    /// The main thread's counters for a host resume (C++ `resume_start`
    /// with no `from`): level 1, yieldable.
    fn enter_resumed(&mut self) {
        let main = self.main_thread;
        self.heap.thread_mut(main).status = ThreadStatus::Running;
        self.c_calls = 1;
        self.base_c_calls = 1;
        self.resume_calls = 1;
        self.epoch = self.epoch.wrapping_add(1);
    }

    fn finish_resume(&mut self, result: crate::error::VmResult<Exit>) -> Result<RunState, Error> {
        let outcome = match result {
            Ok(Exit::Yielded { first, n }) => {
                let values = (first..first.saturating_add(n))
                    .map(|slot| self.host_value(self.get(slot)))
                    .collect();
                // The yielded values are the yield's arguments; the frame
                // keeps them until it resumes.
                Ok(RunState::Yielded(values))
            }
            Ok(Exit::Finished) => {
                let values = self.take_results(0);
                self.reset_main();
                Ok(RunState::Finished(values))
            }
            Ok(Exit::Returned) => {
                self.reset_main();
                Err(Error::Host("the chunk left the main thread".to_owned()))
            }
            Err(throw) => {
                let error = self.render_error(throw);
                self.reset_main();
                Err(error)
            }
        };
        self.safepoint();
        outcome
    }

    /// Loads a chunk for `run` or `start`, on an idle main thread.
    fn load_main(
        &mut self,
        chunk_name: &[u8],
        bytecode: &[u8],
    ) -> Result<crate::value::FnId, Error> {
        if !self.frames.is_empty() || self.current != self.main_thread {
            return Err(Error::Host(
                "Vm::run called from inside a running call".to_owned(),
            ));
        }
        let env = self.globals;
        match crate::bytecode::load::load(self, chunk_name, bytecode, env) {
            Ok(f) => {
                self.reset_main();
                Ok(f)
            }
            Err(error) => {
                // A rejected chunk may have allocated strings and prototypes
                // before the failure; collect them at the host boundary.
                self.safepoint();
                Err(error)
            }
        }
    }

    /// The values from `first` to `top`, copied out; empties the stack.
    fn take_results(&mut self, first: usize) -> Vec<HostValue> {
        let values = (first..self.top)
            .map(|slot| self.host_value(self.get(slot)))
            .collect();
        self.top = 0;
        values
    }

    /// The main thread idle at level 0: no frames, no open upvalues.
    fn reset_main(&mut self) {
        let saved = Saved {
            c_calls: 0,
            base_c_calls: 0,
            epoch: self.epoch,
        };
        self.unwind(0, 0, saved);
        self.top = 0;
        self.reentry = 0;
        self.request = None;
        let main = self.main_thread;
        self.heap.thread_mut(main).status = ThreadStatus::Running;
    }

    /// Loads `bytecode` without running it and returns the opcode of every
    /// instruction in every prototype, in code order (aux words skipped).
    /// For coverage tooling (the differential corpus checks it reaches every
    /// opcode the interpreter implements); not part of the embedding API.
    #[doc(hidden)]
    pub fn opcodes(&mut self, bytecode: &[u8]) -> Result<Vec<u8>, Error> {
        use crate::bytecode::{insn_op, op};
        let env = self.globals;
        let f = crate::bytecode::load::load(self, b"=opcodes", bytecode, env)?;
        let mut pending = match self.heap.function(f) {
            Function::Lua(closure) => vec![Rc::clone(&closure.proto)],
            Function::Native(_) => Vec::new(),
        };
        let mut ops = Vec::new();
        while let Some(proto) = pending.pop() {
            let mut skip = false;
            for &insn in proto.code.iter() {
                if skip {
                    skip = false;
                    continue;
                }
                let code = insn_op(insn);
                skip = op::has_aux(code);
                ops.push(code);
            }
            for &child in proto.children.iter() {
                pending.push(Rc::clone(self.heap.protos.get(child.0)));
            }
        }
        Ok(ops)
    }

    /// A full collection.
    pub fn collect(&mut self) -> CollectStats {
        self.full_gc()
    }

    /// Approximate heap bytes in use.
    pub fn memory_bytes(&self) -> usize {
        self.heap.total_bytes()
    }

    // ---- conversions ---------------------------------------------------

    pub(crate) fn host_value(&self, v: Value) -> HostValue {
        match v {
            Value::Nil => HostValue::Nil,
            Value::Boolean(b) => HostValue::Boolean(b),
            Value::Number(n) => HostValue::Number(n),
            Value::Vector(v) => HostValue::Vector(v),
            Value::String(id) => HostValue::String(self.heap.str_bytes(id).to_vec()),
            other => HostValue::Object(other.type_name()),
        }
    }

    pub(crate) fn value_from_host(&mut self, v: &HostValue) -> crate::error::VmResult<Value> {
        Ok(match v {
            HostValue::Nil | HostValue::Object(_) => Value::Nil,
            HostValue::Boolean(b) => Value::Boolean(*b),
            HostValue::Number(n) => Value::Number(*n),
            HostValue::Vector(v) => Value::Vector(*v),
            HostValue::String(bytes) => Value::String(self.heap.intern(bytes)?),
        })
    }

    /// The host-facing form of a Lua error: strings and numbers as text,
    /// anything else as C++ `lua_tostring` would fail on it.
    fn render_error(&mut self, throw: Throw) -> Error {
        let message = match throw.value {
            Value::String(id) => self.heap.str_bytes(id).to_vec(),
            Value::Number(n) => crate::num::num2str(n).into_bytes(),
            other => format!("(error object is a {} value)", other.type_name()).into_bytes(),
        };
        Error::Lua {
            kind: throw.kind,
            message,
        }
    }

    fn host_result<T>(&mut self, result: crate::error::VmResult<T>) -> Result<T, Error> {
        result.map_err(|t| self.render_error(t))
    }
}
