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
