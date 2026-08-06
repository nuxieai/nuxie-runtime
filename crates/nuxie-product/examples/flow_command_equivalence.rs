//! Diagnostic latency/allocation measurement for UNIV-1631.
//!
//! This is deliberately not a pass/fail performance gate. Run with:
//! `cargo run -p nuxie-product --example flow_command_equivalence -- 1000`.

#[path = "../tests/flow_command_equivalence_support.rs"]
mod support;

use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use support::{run_command_scalar_iterations, run_flow_scalar_iterations};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

struct Measurement {
    elapsed: Duration,
    allocations: usize,
}

fn measure(run: impl FnOnce()) -> Measurement {
    ALLOCATIONS.store(0, Ordering::SeqCst);
    let started = Instant::now();
    run();
    let elapsed = started.elapsed();
    let allocations = ALLOCATIONS.load(Ordering::SeqCst);
    Measurement {
        elapsed,
        allocations,
    }
}

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|value| {
            value
                .parse::<usize>()
                .expect("iterations must be an integer")
        })
        .unwrap_or(1_000);
    assert!(iterations > 0, "iterations must be positive");

    black_box(run_flow_scalar_iterations(10));
    black_box(run_command_scalar_iterations(10));

    let flow = measure(|| {
        black_box(run_flow_scalar_iterations(iterations));
    });
    let command = measure(|| {
        black_box(run_command_scalar_iterations(iterations));
    });

    println!("flow_command_equivalence measurement (diagnostic; not a gate)");
    println!("iterations={iterations}");
    println!(
        "flow: elapsed_ns={} ns_per_iteration={} allocations={} allocations_per_iteration={:.2}",
        flow.elapsed.as_nanos(),
        flow.elapsed.as_nanos() / iterations as u128,
        flow.allocations,
        flow.allocations as f64 / iterations as f64,
    );
    println!(
        "command_server: elapsed_ns={} ns_per_iteration={} allocations={} allocations_per_iteration={:.2}",
        command.elapsed.as_nanos(),
        command.elapsed.as_nanos() / iterations as u128,
        command.allocations,
        command.allocations as f64 / iterations as f64,
    );
}
