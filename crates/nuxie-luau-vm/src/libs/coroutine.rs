//! The coroutine library (C++ `lcorolib.cpp`), phase 2 lane 4: `create`,
//! `resume`, `yield`, `status`, `wrap`, `running`, `isyieldable`, `close`.
//!
//! `resume` and `wrap`'s function request the switch (`Vm::request_resume`)
//! and finish in a continuation when the coroutine yields, returns or dies;
//! `yield` requests the switch back. Nothing recurses on the Rust stack, so
//! coroutines nest as deep as `Limits::max_c_calls` allows, as in C++.

use crate::call::{Continuation, Outcome};
use crate::error::{ErrorKind, Throw, VmResult};
use crate::native::Args;
use crate::thread::{CoStatus, ThreadId, ThreadStatus};
use crate::value::Value;
use crate::vm::Vm;

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    vm.open_lib(
        "coroutine",
        &[
            ("create", create),
            ("resume", resume),
            ("running", running),
            ("status", status),
            ("wrap", wrap),
            ("yield", yield_),
            ("isyieldable", isyieldable),
            ("close", close),
        ],
    )?;
    Ok(())
}

fn check_thread(vm: &mut Vm, args: Args, n: usize) -> VmResult<ThreadId> {
    match vm.arg(args, n) {
        Some(Value::Thread(id)) => Ok(id),
        _ => Err(vm.arg_type_error(args, n, "thread")),
    }
}

fn create(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let f = match vm.arg(args, 1) {
        Some(f @ Value::Function(_)) => f,
        _ => return Err(vm.arg_type_error(args, 1, "function")),
    };
    let co = vm.new_coroutine(f)?;
    vm.ret(&[Value::Thread(co)])
}

fn running(vm: &mut Vm, _: Args) -> VmResult<usize> {
    // The main thread is not a coroutine.
    let v = if vm.current == vm.main_thread {
        Value::Nil
    } else {
        Value::Thread(vm.current)
    };
    vm.ret(&[v])
}

fn status(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let co = check_thread(vm, args, 1)?;
    let name = vm.co_status(co).name();
    vm.ret_str(name.as_bytes())
}

fn isyieldable(vm: &mut Vm, _: Args) -> VmResult<usize> {
    let yes = vm.yieldable();
    vm.ret(&[Value::Boolean(yes)])
}

fn yield_(vm: &mut Vm, args: Args) -> VmResult<usize> {
    if !vm.yieldable() {
        return Err(vm.runtime_error("attempt to yield across metamethod/C-call boundary"));
    }
    vm.request_yield(args.nargs)
}

/// C++ `auxresume` up to the switch: the `nargs` values at `first` go to
/// `co`. `Err` is the message of a coroutine that cannot be resumed.
fn aux_resume(
    vm: &mut Vm,
    co: ThreadId,
    first: usize,
    nargs: usize,
    wrap: bool,
) -> VmResult<Result<usize, Throw>> {
    let status = vm.co_status(co);
    if status != CoStatus::Suspended {
        let message = format!("cannot resume {} coroutine", status.name());
        return Ok(Err(vm.throw_string(message.as_bytes(), ErrorKind::Runtime)));
    }
    let at = vm.top;
    let cont = Box::new(ResumeCont { at, wrap });
    vm.request_resume(co, first, nargs, cont).map(Ok)
}

fn resume(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let co = check_thread(vm, args, 1)?;
    let first = args.base.saturating_add(1);
    match aux_resume(vm, co, first, args.nargs.saturating_sub(1), false)? {
        Ok(n) => Ok(n),
        Err(throw) => resume_failed(vm, throw),
    }
}

/// `coroutine.resume`'s `false, message`.
fn resume_failed(vm: &mut Vm, throw: Throw) -> VmResult<usize> {
    vm.ret(&[Value::Boolean(false), throw.value])
}

fn wrap(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let f = match vm.arg(args, 1) {
        Some(f @ Value::Function(_)) => f,
        _ => return Err(vm.arg_type_error(args, 1, "function")),
    };
    let co = vm.new_coroutine(f)?;
    // The thread is rooted by the closure from here on; until then it is
    // only in a local, and nothing below collects.
    let wrapped = vm.new_builtin("wrap", wrapped, &[Value::Thread(co)])?;
    vm.ret(&[wrapped])
}

/// The function `coroutine.wrap` returns (C++ `auxwrapy`).
fn wrapped(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let Value::Thread(co) = vm.builtin_upval(0) else {
        return Err(vm.runtime_error("internal error: wrap without a thread"));
    };
    match aux_resume(vm, co, args.base, args.nargs, true)? {
        Ok(n) => Ok(n),
        Err(throw) => Err(wrap_error(vm, throw)),
    }
}

/// C++ `auxwrapfinish`: a string (or number) error gets the caller's
/// position.
fn wrap_error(vm: &mut Vm, throw: Throw) -> Throw {
    let message = match throw.value {
        Value::String(_) | Value::Number(_) => throw.value,
        _ => return throw,
    };
    let mut text = vm.where_prefix(1);
    match vm.coerce_to_str(message) {
        Ok(Some(id)) => text.extend_from_slice(vm.heap.str_bytes(id)),
        Ok(None) => {}
        Err(throw) => return throw,
    }
    vm.throw_string(&text, ErrorKind::Runtime)
}

/// `resume` or a wrap function after the coroutine yielded, returned or
/// died; its values are from `at` up.
struct ResumeCont {
    at: usize,
    wrap: bool,
}

impl Continuation for ResumeCont {
    fn resume(self: Box<Self>, vm: &mut Vm, _: Args, outcome: Outcome) -> VmResult<usize> {
        match outcome {
            Outcome::Ok => {
                let n = vm.top.saturating_sub(self.at);
                if self.wrap {
                    return Ok(n);
                }
                vm.ensure_stack(vm.top.saturating_add(1))?;
                vm.copy_slots(self.at, self.at.saturating_add(1), n);
                vm.set(self.at, Value::Boolean(true));
                vm.top = self.at.saturating_add(n).saturating_add(1);
                Ok(n.saturating_add(1))
            }
            Outcome::Err(throw) => {
                vm.top = self.at;
                if self.wrap {
                    Err(wrap_error(vm, throw))
                } else {
                    resume_failed(vm, throw)
                }
            }
        }
    }
}

fn close(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let co = check_thread(vm, args, 1)?;
    let status = vm.co_status(co);
    if matches!(status, CoStatus::Running | CoStatus::Normal) {
        return Err(vm.lib_error(format!("cannot close {} coroutine", status.name())));
    }
    let failed = match vm.heap.thread(co).status {
        ThreadStatus::Failed(kind) => Some((kind, vm.heap.thread(co).error)),
        _ => None,
    };
    vm.reset_thread(co);
    match failed {
        None => vm.ret(&[Value::Boolean(true)]),
        Some((ErrorKind::Memory, _)) => {
            let v = Value::String(vm.heap.intern(b"not enough memory")?);
            vm.ret(&[Value::Boolean(false), v])
        }
        Some((ErrorKind::ErrorHandler, _)) => {
            let v = Value::String(vm.heap.intern(b"error in error handling")?);
            vm.ret(&[Value::Boolean(false), v])
        }
        Some((_, error)) => vm.ret(&[Value::Boolean(false), error]),
    }
}
