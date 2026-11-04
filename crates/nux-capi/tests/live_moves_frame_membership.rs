//! A Screen added between frames opens through the C API in each SDK's order
//! when its unit's spare copy writes its own model on its first advance
//! (UNIV-5924). The small fixture has no env global and no Screen phase, so
//! the SDK writes for those have nothing to write here.
//!
//! A fade home draws its part once on every presented frame, through the C
//! API in the SDKs' order (UNIV-5899).
use nux_capi::*;
use nuxie::live_move_fixture as fixture;
use std::cell::RefCell;
use std::ptr;

fn sv(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
unsafe fn text(view: NuxStringView) -> String {
    if view.data.is_null() || view.len == 0 {
        return String::new();
    }
    String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) })
        .into_owned()
}

/// Each schema's name, and each property's schema, name and kind.
struct Catalog {
    schemas: Vec<String>,
    props: Vec<(usize, String, u32)>,
}
unsafe fn catalog(file: *mut NuxFile) -> Catalog {
    unsafe {
        let mut handle = ptr::null_mut();
        assert_eq!(
            nux_file_view_model_catalog(file, &mut handle),
            NuxStatus::Ok
        );
        let mut info = NuxViewModelCatalogInfo::default();
        assert_eq!(
            nux_view_model_catalog_info(handle, &mut info),
            NuxStatus::Ok
        );
        let mut out = Catalog {
            schemas: Vec::new(),
            props: Vec::new(),
        };
        for i in 0..info.schema_count {
            let mut schema = NuxViewModelSchemaView::default();
            assert_eq!(
                nux_view_model_catalog_schema(handle, i, &mut schema),
                NuxStatus::Ok
            );
            out.schemas.push(text(schema.name));
        }
        for i in 0..info.property_count {
            let mut p = NuxViewModelPropertyView::default();
            assert_eq!(
                nux_view_model_catalog_property(handle, i, &mut p),
                NuxStatus::Ok
            );
            out.props.push((p.schema_index, text(p.name), p.kind));
        }
        nux_view_model_catalog_free(handle);
        out
    }
}

/// The SDKs' value markers (NuxieNativeValuePolicy.markers(in:) on iOS): a
/// number, boolean, color or enum `x` with a boolean `isset:x` sibling, or
/// `state:isset:x` for `state:x`.
fn sdk_markers(catalog: &Catalog) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (index, schema) in catalog.schemas.iter().enumerate() {
        let props: Vec<_> = catalog.props.iter().filter(|p| p.0 == index).collect();
        for value in &props {
            if ![
                NUX_VIEW_MODEL_VALUE_KIND_NUMBER,
                NUX_VIEW_MODEL_VALUE_KIND_BOOL,
                NUX_VIEW_MODEL_VALUE_KIND_COLOR,
                NUX_VIEW_MODEL_VALUE_KIND_ENUM,
            ]
            .contains(&value.2)
            {
                continue;
            }
            let marker = match value.1.strip_prefix("state:") {
                Some(rest) => format!("state:isset:{rest}"),
                None => format!("isset:{}", value.1),
            };
            if props
                .iter()
                .any(|p| p.1 == marker && p.2 == NUX_VIEW_MODEL_VALUE_KIND_BOOL)
            {
                out.push((schema.clone(), value.1.clone(), marker));
            }
        }
    }
    out
}

/// A page opened the way the SDKs open one: the page's own model bound, the
/// SDK's value markers set, a layout size and semantics on.
struct Page {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    root: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
    catalog: Catalog,
    rows: Vec<*mut NuxViewModelInstance>,
}
impl Page {
    unsafe fn open(bytes: &[u8]) -> Self {
        unsafe {
            let caps = NuxVideoPlaybackCapabilities {
                playback_available: 1,
                ..Default::default()
            };
            let mut file = ptr::null_mut();
            let mut imported = ptr::null_mut();
            let status = nux_file_import_with_video_capabilities(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &caps,
                &mut file,
                &mut imported,
            );
            if !imported.is_null() {
                nux_capi_result_free(imported);
            }
            assert_eq!(status, NuxStatus::Ok, "import");
            let catalog = catalog(file);
            let markers = sdk_markers(&catalog);
            assert!(!markers.is_empty(), "the Button model has a value marker");
            let entries: Vec<_> = markers
                .iter()
                .map(|(model, value, marker)| NuxValueMarker {
                    model: sv(model),
                    value: sv(value),
                    marker: sv(marker),
                })
                .collect();
            assert_eq!(
                nux_file_set_value_markers(file, entries.as_ptr(), entries.len()),
                NuxStatus::Ok
            );
            let mut artboard = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            let mut root = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_default(artboard, &mut root),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(artboard, root),
                NuxStatus::Ok
            );
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            assert_eq!(
                nux_player_layout_size_set(player, 402.0, 874.0),
                NuxStatus::Ok
            );
            Self {
                file,
                artboard,
                root,
                player,
                catalog,
                rows: Vec::new(),
            }
        }
    }
    /// The SDKs' prepareRow: a fresh default model of the Screen's schema.
    unsafe fn prepare_row(&mut self, schema: &str) -> *mut NuxViewModelInstance {
        unsafe {
            let index = self
                .catalog
                .schemas
                .iter()
                .position(|name| name == schema)
                .unwrap_or_else(|| panic!("no schema {schema}"));
            let mut row = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_schema_default(self.file, index, &mut row),
                NuxStatus::Ok
            );
            self.rows.push(row);
            row
        }
    }
    unsafe fn insert(&self, row: *mut NuxViewModelInstance, index: usize) -> NuxStatus {
        unsafe {
            let write = [NuxViewModelMutation {
                kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
                instance: self.root,
                path: sv("screens"),
                related_instance: row,
                index,
                ..Default::default()
            }];
            let batch = NuxViewModelMutationBatch {
                mutations: write.as_ptr(),
                mutation_count: write.len(),
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            let status = nux_view_model_mutate(&batch, &mut result);
            if !result.is_null() {
                nux_view_model_mutation_result_free(result);
            }
            status
        }
    }
    /// The SDKs' row wake, which flushes the list first (iOS's `.expand`).
    unsafe fn wake(&self, row: u32) -> NuxStatus {
        unsafe { nux_player_set_row_collapsed(self.player, sv("screens"), row, 0) }
    }
    /// One frame's status and, when it failed, its diagnostic.
    unsafe fn step(&self, elapsed_seconds: f32) -> (NuxStatus, String) {
        unsafe {
            let step = NuxPlayerStep {
                elapsed_seconds,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            let status = nux_player_step(self.player, &step, &mut result);
            let mut message = String::new();
            if !result.is_null() {
                if status != NuxStatus::Ok {
                    let mut diagnostic = NuxCapiDiagnosticView::default();
                    if nux_player_step_result_diagnostic(result, &mut diagnostic) == NuxStatus::Ok {
                        message =
                            format!("{}: {}", text(diagnostic.code), text(diagnostic.message));
                    }
                }
                nux_player_step_result_free(result);
            }
            (status, message)
        }
    }
    unsafe fn frame(&self, elapsed_seconds: f32, what: &str) {
        let (status, message) = unsafe { self.step(elapsed_seconds) };
        assert_eq!(status, NuxStatus::Ok, "{what}: {message}");
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            nux_player_free(self.player);
            for row in &self.rows {
                nux_view_model_instance_free(*row);
            }
            nux_view_model_instance_free(self.root);
            nux_artboard_instance_free(self.artboard);
            nux_file_free(self.file);
        }
    }
}

/// C1. iOS opens its first Screen this way (ExperienceScreenViewController,
/// mountInteractiveScreen): prepare the row, insert it at 0, then the first
/// 0 s frame. Screen A's spare copy writes on its first advance.
#[test]
fn a_screen_opens_in_the_ios_order_when_a_spare_copy_writes() {
    unsafe {
        let mut page = Page::open(&fixture::spare_writes_fixture(true));
        let row = page.prepare_row("Screen0");
        assert_eq!(page.insert(row, 0), NuxStatus::Ok);
        page.frame(0.0, "the first frame");
        page.frame(1.0 / 60.0, "the second frame");
    }
}

/// C1's control: one row wake after the insert, before the first frame.
#[test]
fn a_screen_woken_after_its_insert_opens_in_the_ios_order() {
    unsafe {
        let mut page = Page::open(&fixture::spare_writes_fixture(true));
        let row = page.prepare_row("Screen0");
        assert_eq!(page.insert(row, 0), NuxStatus::Ok);
        assert_eq!(page.wake(0), NuxStatus::Ok);
        page.frame(0.0, "the first frame");
        page.frame(1.0 / 60.0, "the second frame");
    }
}

/// C2. Android opens a new incoming Screen this way (ExperienceInteractivePage
/// takeIncomingOnLane, ExperiencePageStep): insert the row with no wake, a
/// 0 s settle frame, then Begin, the flight's frames and End. Screen A is
/// woken after its own insert, so only the incoming Screen is added between
/// frames.
#[test]
fn a_new_incoming_screen_opens_in_the_android_order() {
    unsafe {
        let mut page = Page::open(&fixture::spare_writes_fixture(true));
        let a = page.prepare_row("Screen0");
        assert_eq!(page.insert(a, 0), NuxStatus::Ok);
        assert_eq!(page.wake(0), NuxStatus::Ok);
        page.frame(0.0, "A's first frame");
        page.frame(1.0 / 60.0, "A at rest");
        let b = page.prepare_row("Screen1");
        assert_eq!(page.insert(b, 1), NuxStatus::Ok);
        page.frame(0.0, "the settle frame after B's insert");
        assert_eq!(
            nux_player_step_begin(page.player, sv("B"), &sv("t1")),
            NuxStatus::Ok,
            "Begin"
        );
        for i in 0..10 {
            page.frame(1.0 / 60.0, &format!("flight frame {i}"));
        }
        assert_eq!(
            nux_player_step_end(page.player, sv("B"), &sv("t1")),
            NuxStatus::Ok,
            "End"
        );
        page.frame(1.0 / 60.0, "the first frame in B");
    }
}

/// Where each path of a frame is drawn: the x of the renderer's transform at
/// the draw. The fixture's unit draws one path, and its home on B sits 190
/// to the right of its home on A, so a draw's x says which home holds it.
#[derive(Default)]
struct Ink {
    next: u64,
    stack: Vec<[f32; 6]>,
    current: Option<[f32; 6]>,
    xs: Vec<f32>,
}
thread_local! {
    static INK: RefCell<Ink> = RefCell::new(Ink::default());
}
const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
unsafe extern "C" fn ink_handle(_: *mut std::ffi::c_void) -> u64 {
    INK.with_borrow_mut(|ink| {
        ink.next += 1;
        ink.next
    })
}
unsafe extern "C" fn ink_path(_: *mut std::ffi::c_void, _: *const NuxRawPathView, _: u8) -> u64 {
    unsafe { ink_handle(ptr::null_mut()) }
}
unsafe extern "C" fn ink_release(_: *mut std::ffi::c_void, _: u64) {}
unsafe extern "C" fn ink_save(_: *mut std::ffi::c_void) {
    INK.with_borrow_mut(|ink| {
        let top = ink.current.unwrap_or(IDENTITY);
        ink.stack.push(top);
    });
}
unsafe extern "C" fn ink_restore(_: *mut std::ffi::c_void) {
    INK.with_borrow_mut(|ink| ink.current = ink.stack.pop());
}
unsafe extern "C" fn ink_transform(_: *mut std::ffi::c_void, m: *const f32) {
    let b = unsafe { *m.cast::<[f32; 6]>() };
    INK.with_borrow_mut(|ink| {
        let a = ink.current.unwrap_or(IDENTITY);
        ink.current = Some([
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
        ]);
    });
}
unsafe extern "C" fn ink_draw(_: *mut std::ffi::c_void, _: u64, _: u64) {
    INK.with_borrow_mut(|ink| {
        let x = ink.current.unwrap_or(IDENTITY)[4];
        ink.xs.push(x);
    });
}
fn ink_callbacks() -> NuxRenderCallbacks {
    NuxRenderCallbacks {
        make_render_path: Some(ink_path),
        make_empty_render_path: Some(ink_handle),
        make_render_paint: Some(ink_handle),
        release_render_path: Some(ink_release),
        release_render_paint: Some(ink_release),
        release_render_shader: Some(ink_release),
        release_render_buffer: Some(ink_release),
        release_render_image: Some(ink_release),
        save: Some(ink_save),
        restore: Some(ink_restore),
        transform: Some(ink_transform),
        draw_path: Some(ink_draw),
        ..NuxRenderCallbacks::default()
    }
}

/// The two-row fade-home page, opened through the C API with the ink
/// recorder, after `t1` has flown its unit U from A to B.
struct FadePage {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    root: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
    /// Where U draws at rest on A, and on B.
    at: Option<(f32, f32)>,
}
impl FadePage {
    unsafe fn on_b(callbacks: &NuxRenderCallbacks) -> Self {
        unsafe {
            let bytes = fixture::fade_home_fixture(2);
            let caps = NuxVideoPlaybackCapabilities {
                playback_available: 1,
                ..Default::default()
            };
            let mut page = Self {
                file: ptr::null_mut(),
                artboard: ptr::null_mut(),
                root: ptr::null_mut(),
                player: ptr::null_mut(),
                at: None,
            };
            let mut imported = ptr::null_mut();
            let status = nux_file_import_with_video_capabilities(
                bytes.as_ptr(),
                bytes.len(),
                callbacks,
                &caps,
                &mut page.file,
                &mut imported,
            );
            if !imported.is_null() {
                nux_capi_result_free(imported);
            }
            assert_eq!(status, NuxStatus::Ok, "import");
            assert_eq!(
                nux_artboard_instance_new_named(page.file, sv("Page"), &mut page.artboard),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_instance_new_default(page.artboard, &mut page.root),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(page.artboard, page.root),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_new_default(page.artboard, &mut page.player),
                NuxStatus::Ok
            );
            page.frame(0.0, "the first frame on A");
            let on_a = page.draw();
            page.call(
                nux_player_step_begin(page.player, sv("B"), &sv("t1")),
                "t1 begin",
            );
            page.frame(1.0 / 60.0, "t1's flight");
            page.call(
                nux_player_step_end(page.player, sv("B"), &sv("t1")),
                "t1 end",
            );
            page.frame(1.0 / 60.0, "at rest on B");
            let on_b = page.draw();
            let (&[a], &[b]) = (on_a.as_slice(), on_b.as_slice()) else {
                panic!("U draws once on A and on B: {on_a:?}, {on_b:?}");
            };
            assert!(b - a > 100.0, "B's home is to the right of A's: {a}, {b}");
            page.at = Some((a, b));
            page
        }
    }
    unsafe fn call(&self, status: NuxStatus, what: &str) {
        assert_eq!(status, NuxStatus::Ok, "{what}");
    }
    unsafe fn frame(&self, elapsed_seconds: f32, what: &str) {
        let step = NuxPlayerStep {
            elapsed_seconds,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        let status = unsafe { nux_player_step(self.player, &step, &mut result) };
        if !result.is_null() {
            unsafe { nux_player_step_result_free(result) };
        }
        assert_eq!(status, NuxStatus::Ok, "{what}");
    }
    /// Draws the page, as a presented frame, and returns each path's x.
    unsafe fn draw(&self) -> Vec<f32> {
        INK.with_borrow_mut(|ink| {
            ink.xs.clear();
            ink.stack.clear();
            ink.current = None;
        });
        assert_eq!(
            unsafe { nux_artboard_instance_draw(self.artboard) },
            NuxStatus::Ok
        );
        INK.with_borrow(|ink| ink.xs.clone())
    }
    /// Draws the page and says where U's ink is.
    unsafe fn ink(&self) -> Where {
        let (a, b) = self.at.expect("measured after t1");
        match unsafe { self.draw() }.as_slice() {
            [x] if (x - a).abs() < (x - b).abs() => Where::A,
            [_] => Where::B,
            xs => Where::Drawn(xs.len()),
        }
    }
    unsafe fn collapse_b(&self) {
        unsafe {
            self.call(
                nux_player_set_row_collapsed(self.player, sv("screens"), 1, 1),
                "retire B by collapsing it",
            )
        }
    }
    unsafe fn remove_b(&self) {
        unsafe {
            let write = [NuxViewModelMutation {
                kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
                instance: self.root,
                path: sv("screens"),
                index: 1,
                ..Default::default()
            }];
            let batch = NuxViewModelMutationBatch {
                mutations: write.as_ptr(),
                mutation_count: write.len(),
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            let status = nux_view_model_mutate(&batch, &mut result);
            if !result.is_null() {
                nux_view_model_mutation_result_free(result);
            }
            self.call(status, "retire B by removing it");
        }
    }
}
impl Drop for FadePage {
    fn drop(&mut self) {
        unsafe {
            nux_player_free(self.player);
            nux_view_model_instance_free(self.root);
            nux_artboard_instance_free(self.artboard);
            nux_file_free(self.file);
        }
    }
}
#[derive(Debug, PartialEq)]
enum Where {
    A,
    B,
    /// Not exactly one path: the element is missing or drawn twice.
    Drawn(usize),
}
#[derive(Clone, Copy, Debug)]
enum Retire {
    Collapse,
    Remove,
}

/// The fade home in both SDKs' order for a scene transition: Begin, the
/// fade's frames, then End and retiring B in one batch (iOS
/// `ExperiencePageStep.swift`, Android `ExperiencePageStep.kt`). This fixture
/// has no Screen phases, so the two orders make the same calls here; the
/// SDKs retire B by collapsing it while it stays reachable and by removing it
/// otherwise, and the test runs both. U's ink appears once on every frame,
/// inside B until the landing frame and inside A from it.
#[test]
fn a_fade_home_draws_its_part_once_in_the_sdks_order() {
    for retire in [Retire::Collapse, Retire::Remove] {
        unsafe {
            let page = FadePage::on_b(&ink_callbacks());
            page.call(
                nux_player_step_begin(page.player, sv("A"), &sv("fadeBA")),
                "fadeBA begin",
            );
            for i in 0..10 {
                page.frame(1.0 / 60.0, "a fade frame");
                assert_eq!(page.ink(), Where::B, "{retire:?}: fade frame {i}");
            }
            page.call(
                nux_player_step_end(page.player, sv("A"), &sv("fadeBA")),
                "fadeBA end",
            );
            match retire {
                Retire::Collapse => page.collapse_b(),
                Retire::Remove => page.remove_b(),
            }
            for i in 0..3 {
                page.frame(1.0 / 60.0, "a frame on A");
                assert_eq!(page.ink(), Where::A, "{retire:?}: frame {i} on A");
            }
        }
    }
}

/// Both SDKs' branch with no scene transition retires B before End: Begin
/// and the collapse in one batch, End in a later one. The collapse brings U
/// home inside the call, so every frame from it draws U once, inside A.
#[test]
fn a_fade_home_retired_before_its_end_draws_its_part_once() {
    unsafe {
        let page = FadePage::on_b(&ink_callbacks());
        page.call(
            nux_player_step_begin(page.player, sv("A"), &sv("fadeBA")),
            "fadeBA begin",
        );
        page.collapse_b();
        for i in 0..3 {
            page.frame(1.0 / 60.0, "a frame before End");
            assert_eq!(page.ink(), Where::A, "frame {i} before End");
        }
        page.call(
            nux_player_step_end(page.player, sv("A"), &sv("fadeBA")),
            "fadeBA end",
        );
        page.frame(1.0 / 60.0, "a frame after End");
        assert_eq!(page.ink(), Where::A, "after End");
    }
}
