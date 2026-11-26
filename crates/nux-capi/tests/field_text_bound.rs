//! A step's reports copy their fields' text only up to the step's result
//! bound. This binary counts what each thread allocates, so it keeps its one
//! test apart from the other focus input tests.

use nux_capi::*;
use nuxie_binary::focus_input_fixtures as fixture;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    ptr,
};

struct Counting;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATED: Cell<usize> = const { Cell::new(0) };
}

fn count(bytes: usize) {
    let _ = COUNTING.try_with(|counting| {
        if counting.get() {
            let _ = ALLOCATED.try_with(|total| total.set(total.get().saturating_add(bytes)));
        }
    });
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The C step result bound, `MAX_PLAYER_STEP_RESULT_BYTES` in the runtime.
const STEP_RESULT_BYTES: usize = 8 * 1024 * 1024;

fn view(text: &str) -> NuxStringView {
    NuxStringView {
        data: text.as_ptr().cast(),
        len: text.len(),
    }
}

fn step(player: *mut NuxPlayer, focus: &[NuxPlayerFocusInput]) -> NuxStatus {
    let mut result = ptr::null_mut();
    let status = unsafe {
        nux_player_step(
            player,
            &NuxPlayerStep {
                focus_inputs: focus.as_ptr(),
                focus_input_count: focus.len(),
                elapsed_seconds: 0.016,
                ..NuxPlayerStep::default()
            },
            &mut result,
        )
    };
    if !result.is_null() {
        assert_eq!(
            unsafe { nux_player_step_result_free(result) },
            NuxStatus::Ok
        );
    }
    status
}

/// Type `text` into a focused field whose text listener fires `fires`
/// events: the step's status and the bytes the step allocated.
fn type_into_field(text: &str, fires: usize) -> (NuxStatus, usize) {
    let bytes = fixture::text_field_listeners(
        include_bytes!("../../../fixtures/fonts/roboto-input.ttf"),
        fires,
    );
    unsafe {
        let mut file = ptr::null_mut();
        assert_eq!(
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file
            ),
            NuxStatus::Ok
        );
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        assert_eq!(step(player, &[]), NuxStatus::Ok);
        assert_eq!(
            step(
                player,
                &[NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_NEXT,
                    ..NuxPlayerFocusInput::default()
                }],
            ),
            NuxStatus::Ok
        );

        ALLOCATED.with(|total| total.set(0));
        COUNTING.with(|counting| counting.set(true));
        let status = step(
            player,
            &[NuxPlayerFocusInput {
                kind: NUX_PLAYER_FOCUS_KIND_TEXT,
                text: view(text),
                ..NuxPlayerFocusInput::default()
            }],
        );
        COUNTING.with(|counting| counting.set(false));
        let allocated = ALLOCATED.with(Cell::get);

        assert_eq!(nux_player_free(player), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
        (status, allocated)
    }
}

#[test]
fn reports_past_the_step_bound_copy_no_more_field_text() {
    // 8 KiB typed into a field whose text listener fires 1100 events: about
    // 9 MiB of field text if every report copied it, past the step's bound.
    // The same typing into a field with no events is the baseline: what
    // typing the text costs on its own.
    let text = "a".repeat(8 * 1024);
    let (quiet, typing) = type_into_field(&text, 0);
    assert_eq!(quiet, NuxStatus::Ok);
    let (status, allocated) = type_into_field(&text, 1100);
    assert_eq!(status, NuxStatus::LimitExceeded);
    let reports = allocated.saturating_sub(typing);
    assert!(
        reports < 2 * STEP_RESULT_BYTES,
        "the reports allocated {reports} bytes"
    );
}
