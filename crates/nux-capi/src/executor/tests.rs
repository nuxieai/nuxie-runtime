use super::*;
use std::sync::mpsc;

struct Task {
    run: Option<Box<dyn FnOnce() + Send>>,
    released: mpsc::Sender<()>,
}
unsafe extern "C" fn invoke(context: *mut c_void) {
    let task = unsafe { &mut *context.cast::<Task>() };
    if let Some(run) = task.run.take() {
        run();
    }
}
unsafe extern "C" fn release(context: *mut c_void) {
    let task = unsafe { Box::from_raw(context.cast::<Task>()) };
    let _ = task.released.send(());
}
fn task(run: impl FnOnce() + Send + 'static) -> (*mut c_void, mpsc::Receiver<()>) {
    let (released, rx) = mpsc::channel();
    (
        Box::into_raw(Box::new(Task {
            run: Some(Box::new(run)),
            released,
        }))
        .cast(),
        rx,
    )
}
fn create() -> *const NuxExecutor {
    let mut executor = std::ptr::null();
    assert_eq!(unsafe { nux_executor_new(&mut executor) }, NuxStatus::Ok);
    executor
}
fn wait(executor: *const NuxExecutor, timeout: u64) -> bool {
    let mut done = false;
    assert_eq!(
        unsafe { nux_executor_wait(executor, timeout, &mut done) },
        NuxStatus::Ok
    );
    done
}

#[test]
fn fifo_owner_thread_drain_rejection_and_observers() {
    let executor = create();
    let (unblock, blocked) = mpsc::channel();
    let (started, start_rx) = mpsc::channel();
    let (ctx, first_release) = task(move || {
        let _ = started.send(thread::current().id());
        let _ = blocked.recv();
    });
    assert_eq!(
        unsafe { nux_executor_submit(executor, ctx, Some(invoke), Some(release)) },
        NuxStatus::Ok
    );
    let worker = start_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_ne!(worker, thread::current().id());
    let (events, received) = mpsc::channel();
    let mut releases = Vec::new();
    for index in 0..8 {
        let events = events.clone();
        let (ctx, dropped) = task(move || {
            let _ = events.send((index, thread::current().id()));
        });
        assert_eq!(
            unsafe { nux_executor_submit(executor, ctx, Some(invoke), Some(release)) },
            NuxStatus::Ok
        );
        releases.push(dropped);
    }
    let mut pending = 0;
    assert_eq!(
        unsafe { nux_executor_pending_count(executor, &mut pending) },
        NuxStatus::Ok
    );
    assert_eq!(pending, 8);
    assert_eq!(unsafe { nux_executor_close(executor) }, NuxStatus::Ok);
    assert!(!wait(executor, 0));
    let (ctx, dropped) = task(|| {});
    assert_eq!(
        unsafe { nux_executor_submit(executor, ctx, Some(invoke), Some(release)) },
        NuxStatus::NotFound
    );
    assert!(dropped.try_recv().is_err()); // Rejection retains caller ownership.
    unsafe {
        release(ctx);
    }
    for index in 8..10 {
        let events = events.clone();
        let (ctx, dropped) = task(move || {
            let _ = events.send((index, thread::current().id()));
        });
        assert_eq!(
            unsafe { nux_executor_on_terminated(executor, ctx, Some(invoke), Some(release)) },
            NuxStatus::Ok
        );
        releases.push(dropped);
    }
    unblock.send(()).unwrap();
    assert!(wait(executor, 5000));
    first_release.recv_timeout(Duration::from_secs(5)).unwrap();
    for dropped in releases {
        dropped.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    assert_eq!(
        received.try_iter().collect::<Vec<_>>(),
        (0..10).map(|i| (i, worker)).collect::<Vec<_>>()
    );
    let (events, received) = mpsc::channel();
    let (ctx, dropped) = task(move || {
        let _ = events.send(thread::current().id());
    });
    assert_eq!(
        unsafe { nux_executor_on_terminated(executor, ctx, Some(invoke), Some(release)) },
        NuxStatus::Ok
    );
    assert_eq!(received.recv().unwrap(), thread::current().id());
    dropped.recv().unwrap();
    assert_eq!(unsafe { nux_executor_release(executor) }, NuxStatus::Ok);
}

#[test]
fn callback_can_close_and_release_last_owner_without_self_join() {
    let executor = create();
    let address = executor as usize;
    let (sent, received) = mpsc::channel();
    let (go, wait_go) = mpsc::channel();
    let (ctx, dropped) = task(move || {
        let _ = wait_go.recv();
        let executor = address as *const NuxExecutor;
        let mut done = false;
        let close = unsafe { nux_executor_close(executor) };
        let status = unsafe { nux_executor_wait(executor, u64::MAX, &mut done) };
        let released = unsafe { nux_executor_release(executor) };
        let _ = sent.send((close, status, released));
    });
    assert_eq!(
        unsafe { nux_executor_submit(executor, ctx, Some(invoke), Some(release)) },
        NuxStatus::Ok
    );
    go.send(()).unwrap(); // Ownership passes to the job after submit returns.
    assert_eq!(
        received.recv_timeout(Duration::from_secs(5)).unwrap(),
        (NuxStatus::Ok, NuxStatus::WrongThread, NuxStatus::Ok)
    );
    dropped.recv_timeout(Duration::from_secs(5)).unwrap();
}

#[test]
fn submit_racing_close_never_strands_accepted_jobs() {
    let executor = create();
    let (ran, received) = mpsc::channel();
    let barrier = Arc::new(std::sync::Barrier::new(5));
    let mut workers = Vec::new();
    for _ in 0..4 {
        assert_eq!(unsafe { nux_executor_retain(executor) }, NuxStatus::Ok);
        let address = executor as usize;
        let barrier = barrier.clone();
        let ran = ran.clone();
        workers.push(thread::spawn(move || {
            let executor = address as *const NuxExecutor;
            barrier.wait();
            let mut accepted = 0;
            let mut releases = Vec::new();
            for _ in 0..100 {
                let ran = ran.clone();
                let (ctx, dropped) = task(move || {
                    let _ = ran.send(());
                });
                let status =
                    unsafe { nux_executor_submit(executor, ctx, Some(invoke), Some(release)) };
                if status == NuxStatus::Ok {
                    accepted += 1;
                } else {
                    assert_eq!(status, NuxStatus::NotFound);
                    unsafe {
                        release(ctx);
                    }
                }
                releases.push(dropped);
            }
            unsafe {
                nux_executor_release(executor);
            }
            (accepted, releases)
        }));
    }
    barrier.wait();
    unsafe {
        nux_executor_close(executor);
    }
    let mut accepted = 0;
    for worker in workers {
        let (count, releases) = worker.join().unwrap();
        accepted += count;
        for released in releases {
            released.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
    assert!(wait(executor, 5000));
    assert_eq!(received.try_iter().count(), accepted);
    unsafe {
        nux_executor_release(executor);
    }
}

/// Pause a real CommandServer callback, then unwind in Rust (never across C).
fn fail_worker(executor: *const NuxExecutor) -> (mpsc::Sender<()>, ThreadId) {
    let (go, blocked) = mpsc::channel();
    let (entered, started) = mpsc::channel();
    unsafe { &*executor }
        .shared
        .lock()
        .queue
        .as_mut()
        .unwrap()
        .run_once(Box::new(move |_| {
            let _ = entered.send(thread::current().id());
            let _ = blocked.recv();
            panic!("injected executor worker failure");
        }));
    (go, started.recv_timeout(Duration::from_secs(5)).unwrap())
}

#[derive(Debug, PartialEq, Eq)]
enum CallbackEvent {
    Invoke(usize, ThreadId),
    Release(usize, ThreadId),
}

struct CountedContext {
    id: usize,
    events: mpsc::Sender<CallbackEvent>,
    invokes: std::sync::atomic::AtomicUsize,
    releases: std::sync::atomic::AtomicUsize,
    on_invoke: Box<dyn Fn() + Send + Sync>,
}

impl CountedContext {
    fn new(id: usize, events: &mpsc::Sender<CallbackEvent>) -> Arc<Self> {
        Self::with_callback(id, events, || {})
    }

    fn with_callback(
        id: usize,
        events: &mpsc::Sender<CallbackEvent>,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            events: events.clone(),
            invokes: std::sync::atomic::AtomicUsize::new(0),
            releases: std::sync::atomic::AtomicUsize::new(0),
            on_invoke: Box::new(callback),
        })
    }

    fn transfer(self: &Arc<Self>) -> *mut c_void {
        Arc::into_raw(self.clone()).cast_mut().cast()
    }

    fn assert_counts(self: &Arc<Self>, invokes: usize, releases: usize) {
        use std::sync::atomic::Ordering::SeqCst;
        assert_eq!(
            self.invokes.load(SeqCst),
            invokes,
            "context {} invokes",
            self.id
        );
        assert_eq!(
            self.releases.load(SeqCst),
            releases,
            "context {} releases",
            self.id
        );
        // The test owns the only remaining reference after a transferred context releases.
        assert_eq!(Arc::strong_count(self), 1, "context {} retained", self.id);
    }
}

unsafe extern "C" fn counted_invoke(context: *mut c_void) {
    let context = unsafe { &*context.cast::<CountedContext>() };
    context
        .invokes
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let _ = context
        .events
        .send(CallbackEvent::Invoke(context.id, thread::current().id()));
    (context.on_invoke)();
}

unsafe extern "C" fn counted_release(context: *mut c_void) {
    let pointer = context.cast::<CountedContext>();
    let context = unsafe { &*pointer };
    let previous = context
        .releases
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let _ = context
        .events
        .send(CallbackEvent::Release(context.id, thread::current().id()));
    // Keep duplicate-release failures observable without double-freeing the raw Arc.
    // The test retains its own reference until all callbacks have completed.
    if previous == 0 {
        drop(unsafe { Arc::from_raw(pointer) });
    }
}

#[test]
fn worker_failure_releases_abandoned_jobs_once_before_termination() {
    let executor = create();
    let (fail, worker) = fail_worker(executor);
    let (events, received) = mpsc::channel();
    let jobs: Vec<_> = (0..8).map(|id| CountedContext::new(id, &events)).collect();
    for job in &jobs {
        assert_eq!(
            unsafe {
                nux_executor_submit(
                    executor,
                    job.transfer(),
                    Some(counted_invoke),
                    Some(counted_release),
                )
            },
            NuxStatus::Ok
        );
    }
    let observer = CountedContext::new(8, &events);
    assert_eq!(
        unsafe {
            nux_executor_on_terminated(
                executor,
                observer.transfer(),
                Some(counted_invoke),
                Some(counted_release),
            )
        },
        NuxStatus::Ok
    );
    let mut pending = 0;
    assert_eq!(
        unsafe { nux_executor_pending_count(executor, &mut pending) },
        NuxStatus::Ok
    );
    assert_eq!(pending, 8);
    assert!(!wait(executor, 0));
    fail.send(()).unwrap();
    // Failure itself must close admission and finish; no explicit close drives this test.
    assert!(wait(executor, 5000));
    for job in &jobs {
        job.assert_counts(0, 1);
    }
    observer.assert_counts(1, 1);
    let observed: Vec<_> = received.try_iter().collect();
    assert_eq!(observed.len(), 10);
    // Abandonment order is unspecified; every release must precede the observer.
    for job in &jobs {
        assert!(observed[..8].contains(&CallbackEvent::Release(job.id, worker)));
    }
    assert_eq!(
        observed[8..],
        [
            CallbackEvent::Invoke(8, worker),
            CallbackEvent::Release(8, worker)
        ]
    );
    assert_eq!(
        unsafe { nux_executor_pending_count(executor, &mut pending) },
        NuxStatus::Ok
    );
    assert_eq!(pending, 0);

    let rejected = CountedContext::new(9, &events);
    let context = rejected.transfer();
    assert_eq!(
        unsafe {
            nux_executor_submit(
                executor,
                context,
                Some(counted_invoke),
                Some(counted_release),
            )
        },
        NuxStatus::NotFound
    );
    // Rejection transfers no ownership and invokes neither callback.
    drop(unsafe { Arc::from_raw(context.cast::<CountedContext>()) });
    rejected.assert_counts(0, 0);
    assert!(received.try_recv().is_err());
    assert_eq!(unsafe { nux_executor_release(executor) }, NuxStatus::Ok);
}

#[test]
fn worker_failure_drains_nested_observers_and_notifies_waiters() {
    let executor = create();
    let address = executor as usize;
    let (fail, worker) = fail_worker(executor);
    let (events, received) = mpsc::channel();
    let (entered, started) = mpsc::channel();
    let (unblock, blocked) = mpsc::channel();
    let blocked = Mutex::new(blocked);
    let (registration, registered) = mpsc::channel();
    let nested = CountedContext::new(2, &events);
    let nested_context = nested.transfer() as usize;
    let observer = CountedContext::with_callback(0, &events, move || {
        let executor = address as *const NuxExecutor;
        let status = unsafe {
            nux_executor_on_terminated(
                executor,
                nested_context as *mut c_void,
                Some(counted_invoke),
                Some(counted_release),
            )
        };
        let mut done = false;
        let self_wait = unsafe { nux_executor_wait(executor, 0, &mut done) };
        let _ = registration.send((status, self_wait, done));
        let _ = entered.send(());
        let _ = blocked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .recv();
    });
    let second = CountedContext::new(1, &events);
    for context in [&observer, &second] {
        assert_eq!(
            unsafe {
                nux_executor_on_terminated(
                    executor,
                    context.transfer(),
                    Some(counted_invoke),
                    Some(counted_release),
                )
            },
            NuxStatus::Ok
        );
    }
    let (waiting, waiter_started) = mpsc::channel();
    let waiter = thread::spawn(move || {
        let _ = waiting.send(());
        wait(address as *const NuxExecutor, 5000)
    });
    waiter_started.recv_timeout(Duration::from_secs(5)).unwrap();
    fail.send(()).unwrap();
    started.recv_timeout(Duration::from_secs(5)).unwrap();
    let self_wait = registered.recv_timeout(Duration::from_secs(5)).unwrap();
    let premature_completion = wait(executor, 0);
    // Always unblock the observer before assertions so a failed check cannot strand it.
    unblock.send(()).unwrap();
    assert!(waiter.join().unwrap());
    assert!(!premature_completion);
    assert_eq!(self_wait, (NuxStatus::Ok, NuxStatus::WrongThread, false));
    for context in [&observer, &second, &nested] {
        context.assert_counts(1, 1);
    }
    assert_eq!(
        received.try_iter().collect::<Vec<_>>(),
        vec![
            CallbackEvent::Invoke(0, worker),
            CallbackEvent::Release(0, worker),
            CallbackEvent::Invoke(1, worker),
            CallbackEvent::Release(1, worker),
            CallbackEvent::Invoke(2, worker),
            CallbackEvent::Release(2, worker),
        ]
    );

    let late = CountedContext::new(3, &events);
    assert_eq!(
        unsafe {
            nux_executor_on_terminated(
                executor,
                late.transfer(),
                Some(counted_invoke),
                Some(counted_release),
            )
        },
        NuxStatus::Ok
    );
    late.assert_counts(1, 1);
    assert_eq!(
        received.try_iter().collect::<Vec<_>>(),
        vec![
            CallbackEvent::Invoke(3, thread::current().id()),
            CallbackEvent::Release(3, thread::current().id()),
        ]
    );
    assert_eq!(unsafe { nux_executor_release(executor) }, NuxStatus::Ok);
}
