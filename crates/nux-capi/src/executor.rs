//! Shared SDK execution lane. Scene ownership stays in the ordinary C ABI;
//! only job transport is delegated to the upstream-shaped command queue.
use crate::{NuxStatus, ffi_guard};
use nuxie::command_queue::CommandQueue;
use nuxie::command_server::CommandServer;
use nuxie::render_api::{NullFactory, PersistentFactory};
use nuxie::runtime::factory::RuntimeFactoryHandle;
use std::ffi::c_void;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, ThreadId};
use std::time::Duration;

/// Invoked without executor locks. Must not unwind or throw across C.
pub type NuxExecutorCallback = Option<unsafe extern "C" fn(*mut c_void)>;

struct Job {
    context: *mut c_void,
    invoke: unsafe extern "C" fn(*mut c_void),
    release: unsafe extern "C" fn(*mut c_void),
}

// The submit contract transfers exclusive context ownership to the executor.
// Only the worker invokes it; release also supports abandonment on worker failure.
unsafe impl Send for Job {}

impl Job {
    fn run(self) {
        unsafe { (self.invoke)(self.context) };
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe { (self.release)(self.context) };
    }
}

struct State {
    queue: Option<CommandQueue>,
    closing: bool,
    terminated: bool,
    worker: Option<ThreadId>,
    pending: usize,
    observers: Vec<Job>,
}

struct Shared {
    state: Mutex<State>,
    completed: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn close(&self) {
        let mut state = self.lock();
        if !state.closing {
            state.closing = true;
            if let Some(queue) = &mut state.queue {
                // Serialized with submit: every accepted RunOnce precedes Disconnect.
                queue.disconnect();
            }
        }
    }

    fn finish(&self) {
        // Drop unconsumed callbacks outside the state lock if the server failed.
        let queue = {
            let mut state = self.lock();
            state.closing = true;
            state.queue.take()
        };
        drop(queue);
        loop {
            let observers = {
                let mut state = self.lock();
                if state.observers.is_empty() {
                    state.pending = 0;
                    state.terminated = true;
                    self.completed.notify_all();
                    return;
                }
                std::mem::take(&mut state.observers)
            };
            for observer in observers {
                observer.run();
            }
        }
    }
}

/// Thread-safe, reference-counted executor handle, independent of scene handles.
/// Retain before sharing ownership. A borrowed handle must remain retained for
/// every call; releasing the last reference must not race a borrowed call.
/// Last release closes admission without blocking and drains accepted work.
pub struct NuxExecutor {
    shared: Arc<Shared>,
}

impl Drop for NuxExecutor {
    fn drop(&mut self) {
        self.shared.close();
    }
}

/// Create a dedicated native thread serving the runtime CommandQueue. Returns
/// only after startup. The returned handle owns one reference. No GPU is created.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_new(out: *mut *const NuxExecutor) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out = std::ptr::null();
        }
        let queue = CommandQueue::new();
        let worker_queue = queue.clone();
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                queue: Some(queue),
                closing: false,
                terminated: false,
                worker: None,
                pending: 0,
                observers: Vec::new(),
            }),
            completed: Condvar::new(),
        });
        let worker_shared = shared.clone();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let spawn = thread::Builder::new()
            .name("com.nuxie.runtime.native".into())
            .spawn(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut factory = PersistentFactory::new(NullFactory::new());
                    let Some(factory) = RuntimeFactoryHandle::from_factory(&mut factory) else {
                        return;
                    };
                    let mut server = CommandServer::new(worker_queue, factory, None);
                    worker_shared.lock().worker = Some(thread::current().id());
                    if ready_tx.send(()).is_ok() {
                        server.serve_until_disconnect();
                    }
                    // Server and its thread-affine factory die before observers.
                }));
                worker_shared.finish();
            });
        if spawn.is_err() || ready_rx.recv().is_err() {
            return NuxStatus::RuntimeError;
        }
        unsafe {
            *out = Arc::into_raw(Arc::new(NuxExecutor { shared }));
        }
        NuxStatus::Ok
    })
}

/// Add one reference to a live executor. Does not reopen a closed executor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_retain(executor: *const NuxExecutor) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if executor.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            Arc::increment_strong_count(executor);
        }
        NuxStatus::Ok
    })
}

/// Release one reference. May be called from a job on this executor; never joins
/// the calling thread. Use close and wait when synchronous teardown is required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_release(executor: *const NuxExecutor) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if !executor.is_null() {
            unsafe {
                drop(Arc::from_raw(executor));
            }
        }
        NuxStatus::Ok
    })
}

/// Submit one FIFO job. Ok transfers context ownership: invoke runs once on the
/// worker, then release runs once. On worker failure, release may run without
/// invoke; the adapter must resolve any pending result in that case. Both
/// callbacks are required and must not throw. Rejection (NotFound after close)
/// transfers no ownership and calls neither callback. Scene handles must be
/// created, used, read and freed inside jobs on this same executor.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_submit(
    executor: *const NuxExecutor,
    context: *mut c_void,
    invoke: NuxExecutorCallback,
    release: NuxExecutorCallback,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(executor) = (unsafe { executor.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let (Some(invoke), Some(release)) = (invoke, release) else {
            return NuxStatus::NullArgument;
        };
        let mut state = executor.shared.lock();
        if state.closing {
            return NuxStatus::NotFound;
        }
        let Some(pending) = state.pending.checked_add(1) else {
            return NuxStatus::LimitExceeded;
        };
        state.pending = pending;
        let shared = Arc::downgrade(&executor.shared);
        let job = Job {
            context,
            invoke,
            release,
        };
        state
            .queue
            .as_mut()
            .expect("open executor has queue")
            .run_once(Box::new(move |_| {
                if let Some(shared) = shared.upgrade() {
                    let mut state = shared.lock();
                    state.pending = state.pending.saturating_sub(1);
                }
                job.run();
            }));
        NuxStatus::Ok
    })
}

/// Stop accepting jobs and drain all accepted work. Idempotent and nonblocking.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_close(executor: *const NuxExecutor) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(executor) = (unsafe { executor.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        executor.shared.close();
        NuxStatus::Ok
    })
}

/// Observe drained native work and server destruction. Multiple observers are
/// supported. Before termination, callbacks execute on the worker; late
/// registration executes inline on the caller, without locks. Ok transfers
/// context ownership and guarantees invoke then release once. Observers may
/// register more observers. Wait includes observers registered before completion.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_on_terminated(
    executor: *const NuxExecutor,
    context: *mut c_void,
    invoke: NuxExecutorCallback,
    release: NuxExecutorCallback,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(executor) = (unsafe { executor.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let (Some(invoke), Some(release)) = (invoke, release) else {
            return NuxStatus::NullArgument;
        };
        let job = Job {
            context,
            invoke,
            release,
        };
        let mut state = executor.shared.lock();
        if state.terminated {
            drop(state);
            job.run();
        } else {
            state.observers.push(job);
        }
        NuxStatus::Ok
    })
}

/// Wait for close/drain/observers. Does not initiate close. UINT64_MAX waits
/// indefinitely; other values are milliseconds. Ok writes whether termination
/// completed (false on timeout). WrongThread prevents waiting on one's own
/// executor, including from a termination observer. Thread-local platform exit
/// hooks may finish after this logical termination notification.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_wait(
    executor: *const NuxExecutor,
    timeout_ms: u64,
    out_terminated: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(executor) = (unsafe { executor.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if out_terminated.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_terminated = false;
        }
        let mut state = executor.shared.lock();
        if state.worker == Some(thread::current().id()) {
            return NuxStatus::WrongThread;
        }
        if timeout_ms == u64::MAX {
            state = executor
                .shared
                .completed
                .wait_while(state, |s| !s.terminated)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        } else {
            state = executor
                .shared
                .completed
                .wait_timeout_while(state, Duration::from_millis(timeout_ms), |s| !s.terminated)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
        unsafe {
            *out_terminated = state.terminated;
        }
        NuxStatus::Ok
    })
}

/// Number of accepted jobs that have not begun execution (excludes observers).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_executor_pending_count(
    executor: *const NuxExecutor,
    out: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let Some(executor) = (unsafe { executor.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if out.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out = executor.shared.lock().pending;
        }
        NuxStatus::Ok
    })
}

#[cfg(test)]
mod tests;
