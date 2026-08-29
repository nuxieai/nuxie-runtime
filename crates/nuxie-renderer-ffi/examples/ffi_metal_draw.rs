#[cfg(target_os = "macos")]
mod app {
    use anyhow::{Context, Result, bail};
    use nuxie_binary::read_runtime_file;
    use nuxie_render_api::PersistentFactory;
    use nuxie_renderer_ffi::FfiFactory;
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    use std::env;
    use std::path::PathBuf;

    pub fn main() {
        if let Err(error) = run() {
            eprintln!("ffi_metal_draw error: {error:#}");
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
        // The native File owns the execution path. The binary preflight is
        // limited to dimensions needed to create Metal's fixed-size surface.
        let metadata = read_runtime_file(&bytes).context("failed to read runtime metadata")?;
        let artboard_object = metadata
            .artboard(0)
            .context("missing selected artboard object")?;
        let width = frame_dimension(artboard_object.double_property("width").unwrap_or(0.0));
        let height = frame_dimension(artboard_object.double_property("height").unwrap_or(0.0));
        let mut factory = PersistentFactory::new(
            FfiFactory::new_metal(width, height).context("failed to create Metal FFI factory")?,
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
            .context("failed to begin Metal FFI frame")?;
        artboard.draw(&mut frame);
        let draw_count = frame.end();
        if draw_count == 0 {
            bail!("Metal FFI renderer completed with zero draw calls");
        }

        let pixels = factory
            .borrow()
            .read_pixels()
            .context("failed to read Metal FFI pixels")?;
        let nonzero_pixels = pixels
            .chunks_exact(4)
            .filter(|rgba| rgba.iter().any(|byte| *byte != 0))
            .count();
        if nonzero_pixels == 0 {
            bail!("Metal FFI renderer produced all-clear pixels");
        }
        let checksum = pixels.iter().fold(0u64, |acc, byte| {
            acc.wrapping_mul(16_777_619) ^ u64::from(*byte)
        });

        println!(
            "ffi_metal_draw ok file={} size={}x{} draws={draw_count} nonzero_pixels={nonzero_pixels} checksum={checksum:016x}",
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
}

#[cfg(target_os = "macos")]
fn main() {
    app::main();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("ffi_metal_draw requires macOS Metal");
}
