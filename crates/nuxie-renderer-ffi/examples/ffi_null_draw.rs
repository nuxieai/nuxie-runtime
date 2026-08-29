use anyhow::{Context, Result, bail};
use nuxie_binary::read_runtime_file;
use nuxie_render_api::PersistentFactory;
use nuxie_renderer_ffi::FfiFactory;
use nuxie_runtime::{File, RuntimeFactoryHandle};
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
    // Native File import requires its renderer factory up front. Decode only
    // the authored frame dimensions before creating that fixed-size surface;
    // all runtime execution below belongs to the translated native owners.
    let metadata = read_runtime_file(&bytes).context("failed to read runtime metadata")?;
    let artboard_object = metadata
        .artboard(0)
        .context("missing selected artboard object")?;
    let width = frame_dimension(artboard_object.double_property("width").unwrap_or(0.0));
    let height = frame_dimension(artboard_object.double_property("height").unwrap_or(0.0));
    let mut factory = PersistentFactory::new(
        FfiFactory::new_null(width, height).context("failed to create FFI factory")?,
    );
    let runtime = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).context("retain FFI factory")?,
        None,
        None,
        None,
    )
    .context("failed to import native runtime file")?;
    let artboard = runtime
        .with_file(File::artboard_default)
        .context("missing default artboard instance")?;
    artboard.advance_default(0.0);
    let mut frame = factory
        .borrow_mut()
        .begin_frame(0x00000000)
        .context("failed to begin FFI frame")?;
    artboard.draw(&mut frame);
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
