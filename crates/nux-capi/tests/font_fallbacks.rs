//! Native drawing through the public C process hook and live text surfaces.
use nux_capi::*;
use nuxie_binary::BinaryWriter;
use std::{collections::HashMap, ffi::c_void, ptr};

const LATIN: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
const ARABIC: &[u8] = include_bytes!("../../../fixtures/fonts/ibm-plex-sans-arabic.ttf");

fn key(kind: &str, name: &str) -> u32 {
    let definition = nuxie_schema::definition_by_name(kind).unwrap();
    u32::from(
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|name| {
                nuxie_schema::definition_by_name(name)
                    .unwrap()
                    .properties
                    .iter()
            }))
            .find(|p| p.name == name)
            .unwrap()
            .key
            .int,
    )
}

fn object(writer: &mut BinaryWriter<'_>, kind: &str, fields: impl FnOnce(&mut BinaryWriter<'_>)) {
    writer.write_var_uint32(u32::from(
        nuxie_schema::definition_by_name(kind).unwrap().type_key.int,
    ));
    fields(writer);
    writer.write_var_uint32(0);
}

// Two independent authored font occurrences must both see the process hook.
fn scene(primary: &[u8], value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut w = BinaryWriter::new(&mut bytes);
    w.write_bytes(b"RIVE");
    for value in [7, 0, 9641, 0] {
        w.write_var_uint32(value);
    }
    object(&mut w, "Backboard", |_| {});
    for _ in 0..2 {
        object(&mut w, "FontAsset", |_| {});
        object(&mut w, "FileAssetContents", |w| {
            w.write_var_uint32(key("FileAssetContents", "bytes"));
            w.write_string(primary);
        });
    }
    object(&mut w, "Artboard", |w| {
        for (name, value) in [("width", 500.0), ("height", 300.0)] {
            w.write_var_uint32(key("Artboard", name));
            w.write_f32(value);
        }
    });
    for row in 0..2 {
        let text = 1 + row * 5;
        object(&mut w, "Text", |w| {
            w.write_var_uint32(key("Text", "parentId"));
            w.write_var_uint32(0);
            w.write_var_uint32(key("Text", "y"));
            w.write_f32(row as f32 * 100.0);
        });
        object(&mut w, "TextStylePaint", |w| {
            w.write_var_uint32(key("TextStylePaint", "parentId"));
            w.write_var_uint32(text);
            w.write_var_uint32(key("TextStylePaint", "fontAssetId"));
            w.write_var_uint32(row);
            w.write_var_uint32(key("TextStylePaint", "fontSize"));
            w.write_f32(32.0);
        });
        object(&mut w, "Fill", |w| {
            w.write_var_uint32(key("Fill", "parentId"));
            w.write_var_uint32(text + 1);
        });
        object(&mut w, "SolidColor", |w| {
            w.write_var_uint32(key("SolidColor", "parentId"));
            w.write_var_uint32(text + 2);
            w.write_var_uint32(key("SolidColor", "colorValue"));
            w.write_u32(0xff000000);
        });
        object(&mut w, "TextValueRun", |w| {
            w.write_var_uint32(key("TextValueRun", "parentId"));
            w.write_var_uint32(text);
            w.write_var_uint32(key("TextValueRun", "styleId"));
            w.write_var_uint32(text + 1);
            w.write_var_uint32(key("TextValueRun", "name"));
            w.write_string(format!("row{row}").as_bytes());
            w.write_var_uint32(key("TextValueRun", "text"));
            w.write_string(value.as_bytes());
        });
    }
    drop(w);
    bytes
}

#[derive(Default)]
struct Recording {
    paths: HashMap<u64, Vec<(Vec<u8>, Vec<u32>)>>,
    drawn: Vec<Vec<(Vec<u8>, Vec<u32>)>>,
}

unsafe fn recording<'a>(context: *mut c_void) -> &'a mut Recording {
    unsafe { &mut *context.cast() }
}
unsafe extern "C" fn empty_path(context: *mut c_void) -> u64 {
    let r = unsafe { recording(context) };
    let id = r.paths.len() as u64 + 1;
    r.paths.insert(id, Vec::new());
    id
}
unsafe extern "C" fn raw_path(context: *mut c_void, path: *const NuxRawPathView, _: u8) -> u64 {
    let id = unsafe { empty_path(context) };
    unsafe { add_raw(context, id, path) };
    id
}
unsafe extern "C" fn add_raw(context: *mut c_void, id: u64, path: *const NuxRawPathView) {
    let path = unsafe { &*path };
    if path.verb_count == 0 {
        return;
    }
    let verbs = unsafe { std::slice::from_raw_parts(path.verbs, path.verb_count) }.to_vec();
    let points = unsafe { std::slice::from_raw_parts(path.points, path.point_count * 2) }
        .iter()
        .map(|v| v.to_bits())
        .collect();
    unsafe { recording(context) }
        .paths
        .get_mut(&id)
        .unwrap()
        .push((verbs, points));
}
unsafe extern "C" fn rewind(context: *mut c_void, id: u64) {
    unsafe { recording(context) }
        .paths
        .get_mut(&id)
        .unwrap()
        .clear();
}
unsafe extern "C" fn add_path(context: *mut c_void, to: u64, from: u64, transform: *const f32) {
    let r = unsafe { recording(context) };
    let mut source = r.paths[&from].clone();
    let matrix = unsafe { std::slice::from_raw_parts(transform, 6) };
    for (_, points) in &mut source {
        for point in points.chunks_exact_mut(2) {
            let x = f32::from_bits(point[0]);
            let y = f32::from_bits(point[1]);
            point[0] = (matrix[0] * x + matrix[2] * y + matrix[4]).to_bits();
            point[1] = (matrix[1] * x + matrix[3] * y + matrix[5]).to_bits();
        }
    }
    r.paths.get_mut(&to).unwrap().extend(source);
}
unsafe extern "C" fn paint(_: *mut c_void) -> u64 {
    1
}
unsafe extern "C" fn draw(context: *mut c_void, path: u64, _: u64) {
    let r = unsafe { recording(context) };
    let path = r.paths[&path].clone();
    if !path.is_empty() {
        r.drawn.push(path);
    }
}

fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn render(primary: &[u8], value: &str, live_value: Option<&str>) -> Recording {
    let bytes = scene(primary, value);
    let mut r = Recording::default();
    let callbacks = NuxRenderCallbacks {
        user_data: ptr::from_mut(&mut r).cast(),
        make_render_path: Some(raw_path),
        make_empty_render_path: Some(empty_path),
        make_render_paint: Some(paint),
        render_path_add_raw_path: Some(add_raw),
        render_path_rewind: Some(rewind),
        render_path_add_render_path: Some(add_path),
        draw_path: Some(draw),
        ..Default::default()
    };
    unsafe {
        let mut file = ptr::null_mut();
        assert_eq!(
            nux_file_import(bytes.as_ptr(), bytes.len(), &callbacks, &mut file),
            NuxStatus::Ok
        );
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        if let Some(value) = live_value {
            let mutations = ["row0", "row1"].map(|name| NuxTextRunMutation {
                name: view(name),
                text: NuxByteView {
                    data: value.as_ptr(),
                    len: value.len(),
                },
            });
            let batch = NuxTextRunMutationBatch {
                mutations: mutations.as_ptr(),
                mutation_count: 2,
                ..Default::default()
            };
            assert_eq!(
                nux_artboard_instance_set_text_runs(artboard, &batch, ptr::null_mut()),
                NuxStatus::Ok
            );
        }
        // The occurrence retains the installed fonts after the file handle dies.
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
        assert_eq!(
            nux_artboard_instance_advance(artboard, 0.0, ptr::null_mut()),
            NuxStatus::Ok
        );
        assert_eq!(nux_artboard_instance_draw(artboard), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
    }
    r
}

fn byte_font(bytes: &[u8]) -> *mut NuxFont {
    let source = NuxFontSource {
        bytes: NuxByteView {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
        ..Default::default()
    };
    let mut font = ptr::null_mut();
    unsafe {
        assert_eq!(nux_font_new(&source, &mut font), NuxStatus::Ok);
    }
    font
}

unsafe extern "C" fn pick_font(
    context: *mut c_void,
    missing: u32,
    index: u32,
    weight: u16,
) -> *const NuxFont {
    assert_eq!(weight, 400);
    if index != 0 {
        return ptr::null();
    }
    let font = context.cast::<NuxFont>();
    let mut has = false;
    unsafe {
        assert_eq!(nux_font_has_glyph(font, missing, &mut has), NuxStatus::Ok);
    }
    if has { font } else { ptr::null() }
}

#[test]
fn process_hook_draws_arabic_keeps_latin_and_clear_restores_missing_glyphs() {
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
    }
    let covering = render(ARABIC, "مرحبا", None).drawn;
    assert_eq!(
        covering.len(),
        2,
        "independently authored covering rows draw"
    );
    assert!(render(LATIN, "مرحبا", None).drawn.is_empty());
    let latin = render(LATIN, "Ada", None).drawn;
    assert_eq!(latin.len(), 2);
    let fallback = byte_font(ARABIC);
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(Some(pick_font), fallback.cast()),
            NuxStatus::Ok
        );
    }
    let actual = render(LATIN, "مرحبا", None).drawn;
    // Clear before assertions so failed comparisons cannot leak global state.
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
    }
    assert_eq!(
        actual, covering,
        "the C hook must draw the covering font's Arabic outlines"
    );
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(Some(pick_font), fallback.cast()),
            NuxStatus::Ok
        );
    }
    assert_eq!(render(LATIN, "Ada", None).drawn, latin);
    assert_eq!(render(LATIN, "Ada", Some("مرحبا")).drawn, covering);
    // The same cache handle can answer on a different shaping thread.
    assert_eq!(
        std::thread::spawn(|| render(LATIN, "مرحبا", None).drawn)
            .join()
            .unwrap(),
        covering
    );
    unsafe {
        assert_eq!(
            nux_font_set_fallback_proc(None, ptr::null_mut()),
            NuxStatus::Ok
        );
        assert_eq!(nux_font_free(fallback), NuxStatus::Ok);
    }
    assert!(render(LATIN, "مرحبا", None).drawn.is_empty());
    assert_eq!(render(LATIN, "Ada", None).drawn, latin);
}

#[test]
fn mapped_probe_transfers_mapping_and_weight_copy_outlives_original_handles() {
    let path = std::path::Path::new(
        option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
    )
    .join("../../fixtures/fonts/ibm-plex-sans-arabic.ttf")
    .canonicalize()
    .unwrap();
    unsafe {
        let mut probe = ptr::null_mut();
        assert_eq!(
            nux_font_probe_file(view(path.to_str().unwrap()), 0, &mut probe),
            NuxStatus::Ok
        );
        let mut has = false;
        assert_eq!(
            nux_font_probe_has_glyph(probe, 'م' as u32, &mut has),
            NuxStatus::Ok
        );
        assert!(has);
        assert_eq!(
            nux_font_probe_has_glyph(probe, 0x10ffff, &mut has),
            NuxStatus::Ok
        );
        assert!(!has);
        let mut font = ptr::null_mut();
        assert_eq!(nux_font_probe_make_font(probe, &mut font), NuxStatus::Ok);
        assert_eq!(
            nux_font_probe_has_glyph(probe, 'م' as u32, &mut has),
            NuxStatus::Ok
        );
        assert!(!has, "promoted probes no longer own a face");
        let mut second = ptr::null_mut();
        assert_eq!(
            nux_font_probe_make_font(probe, &mut second),
            NuxStatus::NotFound
        );
        assert!(second.is_null());
        let mut weighted = ptr::null_mut();
        assert_eq!(
            nux_font_with_weight(font, 700, &mut weighted),
            NuxStatus::Ok
        );
        assert_eq!(nux_font_probe_free(probe), NuxStatus::Ok);
        assert_eq!(nux_font_free(font), NuxStatus::Ok);
        assert_eq!(
            nux_font_has_glyph(weighted, 'م' as u32, &mut has),
            NuxStatus::Ok
        );
        assert!(has);
        let address = weighted as usize;
        assert_eq!(
            std::thread::spawn(move || nux_font_free(address as *mut NuxFont))
                .join()
                .unwrap(),
            NuxStatus::Ok
        );
    }
}

#[test]
fn malformed_font_sources_publish_no_handle() {
    for (source, expected) in [
        (
            NuxFontSource {
                struct_size: 0,
                ..Default::default()
            },
            NuxStatus::InvalidStructSize,
        ),
        (NuxFontSource::default(), NuxStatus::InvalidArgument),
        (
            NuxFontSource {
                bytes: NuxByteView {
                    data: ptr::null(),
                    len: 1,
                },
                ..Default::default()
            },
            NuxStatus::NullArgument,
        ),
        (
            NuxFontSource {
                bytes: NuxByteView {
                    data: ptr::dangling(),
                    len: NUX_FONT_MAX_BYTES + 1,
                },
                ..Default::default()
            },
            NuxStatus::LimitExceeded,
        ),
        (
            NuxFontSource {
                bytes: NuxByteView {
                    data: ARABIC.as_ptr(),
                    len: ARABIC.len(),
                },
                face_index: 1,
                ..Default::default()
            },
            NuxStatus::ImportError,
        ),
        (
            NuxFontSource {
                immutable_path: view("relative.ttf"),
                ..Default::default()
            },
            NuxStatus::InvalidArgument,
        ),
    ] {
        let mut out = ptr::dangling_mut();
        unsafe {
            assert_eq!(nux_font_new(&source, &mut out), expected);
        }
        assert!(out.is_null());
    }
}
