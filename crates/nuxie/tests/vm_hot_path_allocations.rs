use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use anyhow::Result;
use nuxie::{ArtboardSpec, Scene, ViewModelInstanceSpec, ViewModelNumberSpec, ViewModelSpec};

struct CountingAllocator;

static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        pointer
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn direct_view_model_slot_writes_allocate_nothing() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, defaults, number), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Canvas".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let mut view_models = tx.view_models();
        let model = view_models.create(ViewModelSpec {
            name: "Playback".into(),
        })?;
        let number = view_models.create_number(
            model,
            ViewModelNumberSpec {
                name: "Duration".into(),
            },
        )?;
        let defaults = view_models.create_instance(
            model,
            ViewModelInstanceSpec {
                name: Some("Defaults".into()),
            },
        )?;
        view_models.set_artboard_default(artboard, defaults)?;
        Ok((artboard, defaults, number))
    })?;
    let instance = scene.instantiate(artboard)?;
    let cursor = scene.vm_cursor(instance, defaults, number)?;

    ALLOCATIONS.store(0, Ordering::Relaxed);
    TRACK_ALLOCATIONS.store(true, Ordering::Release);
    for index in 0..10_000 {
        let value = if index % 2 == 0 { 1.0 } else { 2.0 };
        let result = scene.frame().set_vm(cursor, value);
        if result != Ok(true) {
            TRACK_ALLOCATIONS.store(false, Ordering::Release);
            panic!("direct slot write failed: {result:?}");
        }
    }
    TRACK_ALLOCATIONS.store(false, Ordering::Release);

    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), 0);
    assert_eq!(scene.frame().get_vm(cursor)?, 2.0);
    Ok(())
}
