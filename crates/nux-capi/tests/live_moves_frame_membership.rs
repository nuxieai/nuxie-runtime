//! A Screen added between frames opens through the C API in each SDK's order
//! when its unit's spare copy writes its own model on its first advance
//! (UNIV-5924). The small fixture has no env global and no Screen phase, so
//! the SDK writes for those have nothing to write here.
use nux_capi::*;
use nuxie::live_move_fixture as fixture;
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
