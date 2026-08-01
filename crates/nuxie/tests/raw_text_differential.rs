use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use nuxie::{
    RawText, RawTextFont, RecordingFactory, Renderer, TextAlign, TextOverflow, TextSizing,
};
use serde_json::Value;

// Live matrix ownership: D-RT-API, D-RT-COLOR-188, D-RT-COLOR-402,
// D-RT-COLOR-423, D-RT-COLOR-457, D-RT-COLOR-474.

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("nuxie crate is in the workspace")
        .to_owned()
}

fn probe_path() -> Option<PathBuf> {
    std::env::var_os("RIVE_CPP_PROBE")
        .map(PathBuf::from)
        .or_else(|| {
            let os = if std::env::consts::OS == "macos" {
                "macosx"
            } else {
                std::env::consts::OS
            };
            let path = repo_root()
                .join("tools/cpp-probe/build")
                .join(os)
                .join("bin/debug/rive_cpp_probe");
            path.exists().then_some(path)
        })
}

fn asset(name: &str) -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets")
        .join(name)
}

fn font(path: &Path) -> RawTextFont {
    RawTextFont::decode(Arc::<[u8]>::from(
        std::fs::read(path).expect("read differential font"),
    ))
    .expect("decode differential font")
}

fn cpp_report() -> Option<Value> {
    let probe = probe_path()?;
    let output = Command::new(probe)
        .arg("--raw-text-probe")
        .arg(asset("RobotoFlex.ttf"))
        .arg(asset("TwemojiMozilla.subset.ttf"))
        .arg(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
                .join("skia/dependencies/skia/resources/fonts/sbix.ttf"),
        )
        .output()
        .expect("run freshly built C++ RawText probe");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Some(serde_json::from_slice(&output.stdout).expect("C++ RawText JSON"))
}

fn bounds_array(bounds: nuxie::Aabb) -> [f32; 4] {
    [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
}

fn assert_bounds_close(actual: [f32; 4], expected: &Value, tolerance: f32) {
    let expected = expected.as_array().expect("bounds array");
    for (actual, expected) in actual.into_iter().zip(expected) {
        let expected = expected.as_f64().expect("finite C++ bound") as f32;
        assert!(
            (actual - expected).abs() <= tolerance,
            "RawText bound differs: Rust {actual}, C++ {expected}, tolerance {tolerance}"
        );
    }
}

fn assert_observation(raw: &mut RawText<'_>, renderer: &mut dyn Renderer, expected: &Value) {
    assert_bounds_close(bounds_array(raw.bounds()), &expected["bounds"], 0.08);
    let order = raw.debug_command_kinds();
    let expected_order = expected["order"]
        .as_array()
        .expect("command order")
        .iter()
        .map(|value| value.as_str().expect("command kind"))
        .collect::<Vec<_>>();
    assert_eq!(order, expected_order);
    let style_bounds = raw.debug_style_path_bounds();
    let expected_style_bounds = expected["styleBounds"].as_array().expect("style bounds");
    assert_eq!(style_bounds.len(), expected_style_bounds.len());
    for (actual, expected) in style_bounds.into_iter().zip(expected_style_bounds) {
        assert_bounds_close(bounds_array(actual), expected, 0.08);
    }
    assert_eq!(raw.debug_has_clip(), expected["clip"].as_bool().unwrap());
    raw.render(renderer, None);
}

fn assert_recording_counts(stream: &str, expected: &Value) {
    for (needle, key) in [
        ("save\n", "saves"),
        ("restore\n", "restores"),
        ("clipPath ", "clips"),
        ("drawPath ", "drawPaths"),
        ("drawImage ", "drawImages"),
    ] {
        assert_eq!(
            stream.matches(needle).count(),
            expected[key].as_u64().unwrap() as usize,
            "recording counter {key}"
        );
    }
}

#[test]
fn d_rt_api_live_cpp_table() {
    let Some(cpp) = cpp_report() else {
        eprintln!("skipping D-RT-API; build or set RIVE_CPP_PROBE");
        return;
    };
    let regular = font(&asset("RobotoFlex.ttf"));

    let mut factory = RecordingFactory::new();
    let mut raw = RawText::new(&mut factory);
    let defaults = &cpp["api"]["defaults"];
    assert_eq!(raw.empty(), defaults["empty"].as_bool().unwrap());
    assert_eq!(raw.debug_dirty(), defaults["dirty"].as_bool().unwrap());
    assert_eq!(
        raw.sizing() as u8,
        defaults["sizing"].as_u64().unwrap() as u8
    );
    assert_eq!(
        raw.overflow() as u8,
        defaults["overflow"].as_u64().unwrap() as u8
    );
    assert_eq!(raw.align() as u8, defaults["align"].as_u64().unwrap() as u8);
    assert_bounds_close(bounds_array(raw.bounds()), &defaults["bounds"], 0.0);
    drop(raw);

    let mut factory = RecordingFactory::new();
    let mut empty = RawText::new(&mut factory);
    empty.append_default("", None, &regular);
    assert_eq!(
        !empty.empty(),
        cpp["api"]["append"]["emptyRunMakesNonempty"]
    );
    assert_eq!(empty.debug_dirty(), cpp["api"]["append"]["emptyRunDirty"]);
    drop(empty);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut raw = RawText::new(&mut factory);
    let paint = raw.make_paint();
    raw.append(
        "A\0ignored",
        Some(paint.clone()),
        &regular,
        24.0,
        -1.0,
        0.0,
        0xff12_3456,
    );
    raw.append("B", Some(paint), &regular, 12.0, 40.0, 3.0, 0xffab_cdef);
    let populated = raw.bounds();
    let append = &cpp["api"]["append"];
    assert_eq!(
        raw.debug_style_count(),
        append["styleCount"].as_u64().unwrap() as usize
    );
    assert_eq!(
        raw.debug_style_foreground(0),
        append["foreground"].as_u64().map(|v| v as u32)
    );
    assert_eq!(raw.debug_command_kinds(), vec!["style"]);
    assert_bounds_close(bounds_array(populated), &append["bounds"], 0.05);

    raw.set_sizing(raw.sizing());
    raw.set_overflow(raw.overflow());
    raw.set_align(raw.align());
    raw.set_max_width(raw.max_width());
    raw.set_max_height(raw.max_height());
    raw.set_paragraph_spacing(raw.paragraph_spacing());
    assert_eq!(!raw.debug_dirty(), cpp["api"]["setters"]["equalClean"]);
    raw.set_max_width(f32::NAN);
    assert_eq!(raw.debug_dirty(), cpp["api"]["setters"]["firstNanDirty"]);
    let _ = raw.bounds();
    raw.set_max_width(f32::NAN);
    assert_eq!(raw.debug_dirty(), cpp["api"]["setters"]["secondNanDirty"]);

    raw.set_max_width(80.0);
    raw.set_max_height(30.0);
    raw.set_sizing(TextSizing::Fixed);
    raw.set_overflow(TextOverflow::Clipped);
    assert_bounds_close(
        bounds_array(raw.bounds()),
        &cpp["api"]["clip"]["bounds"],
        0.0,
    );
    assert_eq!(raw.debug_has_clip(), cpp["api"]["clip"]["created"]);
    raw.render(&mut renderer, None);
    raw.set_overflow(TextOverflow::Visible);
    let _ = raw.bounds();
    assert_eq!(!raw.debug_has_clip(), cpp["api"]["clip"]["released"]);

    let stale = raw.bounds();
    raw.clear();
    assert_eq!(raw.bounds(), stale);
    assert_eq!(raw.empty(), cpp["api"]["clear"]["empty"]);
    assert_eq!(
        raw.debug_style_count(),
        cpp["api"]["clear"]["stylesRetained"].as_u64().unwrap() as usize
    );
    assert!(raw.debug_command_kinds().is_empty());
    drop(raw);

    for expected in cpp["api"]["layout"].as_array().expect("layout matrix") {
        let sizing = match expected["sizing"].as_u64().unwrap() {
            0 => TextSizing::AutoWidth,
            1 => TextSizing::AutoHeight,
            2 => TextSizing::Fixed,
            value => panic!("unexpected C++ sizing {value}"),
        };
        let overflow = match expected["overflow"].as_u64().unwrap() {
            0 => TextOverflow::Visible,
            1 => TextOverflow::Hidden,
            2 => TextOverflow::Clipped,
            3 => TextOverflow::Ellipsis,
            4 => TextOverflow::Fit,
            5 => TextOverflow::FitFontSize,
            value => panic!("unexpected C++ overflow {value}"),
        };
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut value = RawText::new(&mut factory);
        let paint = value.make_paint();
        value.set_max_width(70.0);
        value.set_max_height(22.0);
        value.set_paragraph_spacing(7.0);
        value.set_sizing(sizing);
        value.set_overflow(overflow);
        value.append_default("one two three\nfour five", Some(paint), &regular);
        assert_observation(&mut value, &mut renderer, &expected["value"]);
        drop(value);
        assert_recording_counts(&factory.stream(), &expected["value"]);
    }

    for expected in cpp["api"]["align"].as_array().expect("align matrix") {
        let align = match expected["align"].as_u64().unwrap() {
            0 => TextAlign::Left,
            1 => TextAlign::Right,
            2 => TextAlign::Center,
            value => panic!("unexpected C++ align {value}"),
        };
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut value = RawText::new(&mut factory);
        let paint = value.make_paint();
        value.set_max_width(160.0);
        value.set_sizing(TextSizing::AutoHeight);
        value.set_align(align);
        value.append("ABC", Some(paint), &regular, 20.0, -1.0, 0.0, 0xff00_0000);
        assert_observation(&mut value, &mut renderer, &expected["value"]);
        drop(value);
        assert_recording_counts(&factory.stream(), &expected["value"]);
    }

    for (key, text, width, height, overflow) in [
        ("bidi", "abc אבג", 180.0, 0.0, TextOverflow::Visible),
        (
            "ellipsis",
            "one two three four",
            45.0,
            20.0,
            TextOverflow::Ellipsis,
        ),
    ] {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut value = RawText::new(&mut factory);
        let paint = value.make_paint();
        value.set_max_width(width);
        value.set_max_height(height);
        value.set_sizing(if key == "ellipsis" {
            TextSizing::Fixed
        } else {
            TextSizing::AutoHeight
        });
        value.set_overflow(overflow);
        value.append(
            text,
            Some(paint),
            &regular,
            if key == "bidi" { 20.0 } else { 16.0 },
            -1.0,
            0.0,
            0xff00_0000,
        );
        assert_observation(&mut value, &mut renderer, &cpp["api"][key]);
        drop(value);
        assert_recording_counts(&factory.stream(), &cpp["api"][key]);
    }

    for (key, override_enabled) in [("plain", false), ("override", true)] {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut value = RawText::new(&mut factory);
        let override_paint = value.make_paint();
        value.append_default("A", None, &regular);
        let expected = &cpp["api"]["nullPaint"][key];
        assert_bounds_close(bounds_array(value.bounds()), &expected["bounds"], 0.08);
        assert_eq!(value.debug_command_kinds(), vec!["style"]);
        value.render(&mut renderer, override_enabled.then_some(&override_paint));
        drop(value);
        assert_recording_counts(&factory.stream(), expected);
    }

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut value = RawText::new(&mut factory);
    let first = value.make_paint();
    let second = value.make_paint();
    value.append_default("A", Some(first.clone()), &regular);
    value.append("B", Some(second), &regular, 20.0, -1.0, 0.0, 0xff00_0000);
    value.append("C", Some(first), &regular, 12.0, -1.0, 0.0, 0xff00_0000);
    assert_observation(&mut value, &mut renderer, &cpp["api"]["coalescing"]);
    drop(value);
    assert_recording_counts(&factory.stream(), &cpp["api"]["coalescing"]);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    let mut value = RawText::new(&mut factory);
    let paint = value.make_paint();
    value.set_max_width(50.0);
    value.set_max_height(20.0);
    value.set_sizing(TextSizing::Fixed);
    value.set_overflow(TextOverflow::Clipped);
    value.append_default("A", Some(paint), &regular);
    let _ = value.bounds();
    value.clear();
    assert_observation(&mut value, &mut renderer, &cpp["api"]["emptyClipRetained"]);
    drop(value);
    assert_recording_counts(&factory.stream(), &cpp["api"]["emptyClipRetained"]);

    let stored = &cpp["api"]["stored"];
    let mut factory = RecordingFactory::new();
    let mut value = RawText::new(&mut factory);
    value.set_sizing(TextSizing::Fixed);
    value.set_overflow(TextOverflow::FitFontSize);
    value.set_align(TextAlign::Center);
    value.set_max_width(-13.0);
    value.set_max_height(-17.0);
    value.set_paragraph_spacing(-19.0);
    assert_eq!(
        value.sizing() as u8,
        stored["sizing"].as_u64().unwrap() as u8
    );
    assert_eq!(
        value.overflow() as u8,
        stored["overflow"].as_u64().unwrap() as u8
    );
    assert_eq!(value.align() as u8, stored["align"].as_u64().unwrap() as u8);
    assert_eq!(
        value.max_width(),
        stored["maxWidth"].as_f64().unwrap() as f32
    );
    assert_eq!(
        value.max_height(),
        stored["maxHeight"].as_f64().unwrap() as f32
    );
    assert_eq!(
        value.paragraph_spacing(),
        stored["paragraphSpacing"].as_f64().unwrap() as f32
    );
    value.set_max_height(f32::INFINITY);
    value.set_paragraph_spacing(f32::NAN);
    assert_eq!(value.max_height().is_infinite(), stored["infiniteHeight"]);
    assert_eq!(value.paragraph_spacing().is_nan(), stored["nanSpacing"]);
}

#[test]
fn d_rt_color_188_402_423_457_474_live_cpp() {
    let Some(cpp) = cpp_report() else {
        eprintln!("skipping D-RT-COLOR; build or set RIVE_CPP_PROBE");
        return;
    };
    let emoji = font(&asset("TwemojiMozilla.subset.ttf"));
    let regular = font(&asset("RobotoFlex.ttf")).with_fallbacks([emoji.clone()]);
    let cases = [
        ("A", &emoji, 32.0, 200.0),
        ("❤❤❤", &emoji, 32.0, 400.0),
        ("Hello ❤ World", &regular, 32.0, 400.0),
        ("❤", &emoji, 1.0, 100.0),
        ("❤", &emoji, 200.0, 2000.0),
    ];
    for (index, (text, font, size, width)) in cases.into_iter().enumerate() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut raw = RawText::new(&mut factory);
        raw.set_max_width(width);
        raw.set_sizing(TextSizing::AutoHeight);
        raw.append(text, None, font, size, -1.0, 0.0, 0xff00_0000);
        let bounds = raw.bounds();
        raw.render(&mut renderer, None);
        let kinds = raw.debug_command_kinds();
        drop(raw);

        let expected = &cpp["colors"][index];
        assert_bounds_close(bounds_array(bounds), &expected["bounds"], 0.1);
        assert_eq!(
            kinds.len(),
            expected["commands"].as_u64().unwrap() as usize,
            "D-RT-COLOR case {index} command coalescing"
        );
        assert_eq!(
            kinds.iter().filter(|kind| **kind == "color").count(),
            expected["colorCommands"].as_u64().unwrap() as usize
        );
        assert_eq!(
            kinds.iter().filter(|kind| **kind == "style").count(),
            expected["styleCommands"].as_u64().unwrap() as usize
        );
        let stream = factory.stream();
        assert_eq!(
            stream.matches("drawPath ").count(),
            expected["drawPaths"].as_u64().unwrap() as usize
        );
        assert_eq!(
            stream.matches("drawImage ").count(),
            expected["drawImages"].as_u64().unwrap() as usize
        );
    }
}

#[test]
fn d_rt_engine_live_cpp_classification_and_layer_metadata() {
    let Some(cpp) = cpp_report() else {
        eprintln!("skipping D-RT-ENGINE; build or set RIVE_CPP_PROBE");
        return;
    };
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let fonts = [
        (
            "regular",
            std::fs::read(root.join("tests/unit_tests/assets/RobotoFlex.ttf")).unwrap(),
        ),
        (
            "colr",
            std::fs::read(root.join("tests/unit_tests/assets/TwemojiMozilla.subset.ttf")).unwrap(),
        ),
        (
            "raster",
            std::fs::read(root.join("skia/dependencies/skia/resources/fonts/sbix.ttf")).unwrap(),
        ),
    ];
    for (name, bytes) in fonts {
        let expected = &cpp["engine"][name];
        let found = (0..65536).find(|glyph| {
            nuxie_runtime::runtime_classify_color_glyph(&bytes, *glyph)
                != nuxie_runtime::RuntimeColorGlyphClassification::Monochrome
        });
        assert_eq!(
            found.is_some(),
            expected["found"].as_bool().unwrap(),
            "{name}"
        );
        let glyph_id = found.unwrap_or(0);
        assert_eq!(
            glyph_id,
            expected["glyphId"].as_u64().unwrap() as u32,
            "{name}"
        );
        let layers =
            nuxie_runtime::runtime_extract_color_glyph_layers(&bytes, glyph_id, 0xff12_3456);
        let solids = layers
            .iter()
            .filter(|layer| {
                matches!(
                    layer.paint,
                    nuxie_runtime::RuntimeColorGlyphPaint::Solid { .. }
                )
            })
            .count();
        let gradients = layers
            .iter()
            .filter(|layer| {
                matches!(
                    layer.paint,
                    nuxie_runtime::RuntimeColorGlyphPaint::LinearGradient { .. }
                        | nuxie_runtime::RuntimeColorGlyphPaint::RadialGradient { .. }
                        | nuxie_runtime::RuntimeColorGlyphPaint::SweepGradient { .. }
                )
            })
            .count();
        let (images, image_bytes) = layers.iter().fold((0usize, 0usize), |counts, layer| {
            if let nuxie_runtime::RuntimeColorGlyphPaint::Image { ref bytes, .. } = layer.paint {
                (counts.0 + 1, counts.1 + bytes.len())
            } else {
                counts
            }
        });
        assert_eq!(
            layers.len(),
            expected["layers"].as_u64().unwrap() as usize,
            "{name}"
        );
        assert_eq!(
            solids,
            expected["solids"].as_u64().unwrap() as usize,
            "{name}"
        );
        assert_eq!(
            gradients,
            expected["gradients"].as_u64().unwrap() as usize,
            "{name}"
        );
        assert_eq!(
            images,
            expected["images"].as_u64().unwrap() as usize,
            "{name}"
        );
        assert_eq!(
            image_bytes,
            expected["imageBytes"].as_u64().unwrap() as usize,
            "{name}"
        );
    }
}
