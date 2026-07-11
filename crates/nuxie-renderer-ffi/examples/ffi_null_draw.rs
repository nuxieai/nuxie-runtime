use anyhow::{Context, Result, bail};
use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_renderer_ffi::FfiFactory;
use nuxie_runtime::{ArtboardInstance, preallocate_render_paints};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("ffi_null_draw error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let file = env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        let runtime_dir = env::var("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|_| "/Users/levi/dev/oss/rive-runtime".to_string());
        PathBuf::from(runtime_dir).join("tests/unit_tests/assets/dependency_test.riv")
    });

    let bytes =
        std::fs::read(&file).with_context(|| format!("failed to read {}", file.display()))?;
    let runtime = read_runtime_file(&bytes).context("failed to import runtime file")?;
    let graph = GraphFile::from_runtime_file(&runtime).context("failed to build graph")?;
    let (artboard_index, artboard) = graph
        .artboards
        .first()
        .map(|artboard| (0, artboard))
        .context("missing default artboard")?;
    let mut instance = ArtboardInstance::from_graph(&runtime, artboard)
        .context("failed to instantiate artboard")?;
    instance.update_components();

    let artboard_object = runtime
        .artboard(artboard_index)
        .context("missing selected artboard object")?;
    let width = frame_dimension(artboard_object.double_property("width").unwrap_or(0.0));
    let height = frame_dimension(artboard_object.double_property("height").unwrap_or(0.0));
    let mut factory =
        FfiFactory::new_null(width, height).context("failed to create FFI factory")?;
    let mut paint_by_global = preallocate_render_paints(&runtime, artboard, &mut factory);
    instance
        .prepare_static_artboard_paints(&runtime, artboard, &mut factory, &mut paint_by_global)
        .context("failed to prepare paints")?;

    let mut frame = factory
        .begin_frame(0x00000000)
        .context("failed to begin FFI frame")?;
    instance
        .draw_prepared_static_artboard(
            &runtime,
            artboard,
            &mut factory,
            &mut frame,
            &mut paint_by_global,
        )
        .context("failed to draw through FFI frame")?;
    let draw_count = frame.end();
    if draw_count == 0 {
        bail!("FFI renderer completed with zero draw calls");
    }

    println!(
        "ffi_null_draw ok file={} size={}x{} draws={draw_count}",
        file.display(),
        width,
        height
    );
    Ok(())
}

fn frame_dimension(value: f32) -> u32 {
    if value.is_finite() && value > 0.0 {
        value.ceil().min(u32::MAX as f32) as u32
    } else {
        1
    }
}
