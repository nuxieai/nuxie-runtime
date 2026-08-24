use nuxie_render_stream::RenderStream;
use pixel_compare::RgbaImage;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Options {
    stream: PathBuf,
    output: PathBuf,
    backend: String,
    frame: usize,
    clear: Option<u32>,
    #[cfg_attr(
        not(any(
            feature = "native-vulkan-exact",
            feature = "native-webgpu-exact",
            feature = "browser-webgpu-exact",
            all(feature = "native-metal", target_os = "macos"),
            all(feature = "ffi", target_os = "macos")
        )),
        allow(dead_code)
    )]
    mode: String,
    command_limit: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    validate_backend_mode(&options.backend, &options.mode)?;
    let mut stream = RenderStream::parse(&fs::read_to_string(&options.stream)?)?;
    apply_command_limit(&mut stream, options.frame, options.command_limit)?;
    let (width, height) = stream
        .frame_size
        .ok_or("recorded stream does not declare frameSize")?;
    let clear = options.clear.or(stream.clear_color).unwrap_or(0);
    let (mut pixels, adapter): (Vec<u8>, Option<String>) = match options.backend.as_str() {
        "stub" => (clear_pixels(width, height, clear), None),
        #[cfg(feature = "native-vulkan-exact")]
        "rust-vulkan-exact" => {
            replay_native_vulkan(&stream, options.frame, width, height, clear, &options.mode)?
        }
        #[cfg(any(feature = "native-webgpu-exact", feature = "browser-webgpu-exact"))]
        "rust-webgpu-exact" => {
            replay_native_webgpu(&stream, options.frame, width, height, clear, &options.mode)?
        }
        #[cfg(all(feature = "native-metal", target_os = "macos"))]
        "rust-metal" => replay_native_metal(&stream, options.frame, width, height, clear)?,
        #[cfg(all(feature = "native-metal", target_os = "macos"))]
        "rust-metal-atomic" => {
            replay_native_metal_atomic(&stream, options.frame, width, height, clear, &options.mode)?
        }
        #[cfg(all(feature = "ffi", target_os = "macos"))]
        "ffi-metal" => {
            replay_ffi_metal(&stream, options.frame, width, height, clear, &options.mode)?
        }
        #[cfg(all(
            feature = "perf-dawn",
            any(target_os = "macos", target_os = "emscripten")
        ))]
        "ffi-dawn" => replay_ffi_dawn(&stream, options.frame, width, height, clear, &options.mode)?,
        #[cfg(all(feature = "ffi-vulkan", target_os = "macos"))]
        "ffi-vulkan" => {
            replay_ffi_vulkan(&stream, options.frame, width, height, clear, &options.mode)?
        }
        #[cfg(all(feature = "ffi-webgl2", target_os = "emscripten"))]
        "ffi-webgl2" => {
            replay_ffi_webgl2(&stream, options.frame, width, height, clear, &options.mode)?
        }
        backend => {
            return Err(format!(
                "backend `{backend}` is unavailable; use `stub`{}{}{}{}{}{}{}",
                if cfg!(feature = "native-vulkan-exact") {
                    " or `rust-vulkan-exact`"
                } else {
                    ""
                },
                if cfg!(any(
                    feature = "native-webgpu-exact",
                    feature = "browser-webgpu-exact"
                )) {
                    " or `rust-webgpu-exact`"
                } else {
                    ""
                },
                if cfg!(all(feature = "native-metal", target_os = "macos")) {
                    " or `rust-metal` or `rust-metal-atomic`"
                } else {
                    ""
                },
                if cfg!(all(feature = "ffi", target_os = "macos")) {
                    " or `ffi-metal`"
                } else {
                    ""
                },
                if cfg!(all(
                    feature = "perf-dawn",
                    any(target_os = "macos", target_os = "emscripten")
                )) {
                    " or `ffi-dawn`"
                } else {
                    ""
                },
                if cfg!(all(feature = "ffi-vulkan", target_os = "macos")) {
                    " or `ffi-vulkan`"
                } else {
                    ""
                },
                if cfg!(all(feature = "ffi-webgl2", target_os = "emscripten")) {
                    " or `ffi-webgl2`"
                } else {
                    ""
                }
            )
            .into());
        }
    };
    #[cfg(all(feature = "browser-webgpu-exact", target_os = "emscripten"))]
    if options.backend == "rust-webgpu-exact" {
        // TestingWindowWGPU::endFrame reverses the browser readback rows, and
        // TestHarness::savePNG then passes flipY=true to WritePNGFile. Preserve
        // that complete source-owned output chain rather than treating the
        // intermediate pixel buffer as the browser oracle artifact.
        let row_bytes = width as usize * 4;
        for top in 0..height as usize / 2 {
            let bottom = height as usize - 1 - top;
            let (before_bottom, from_bottom) = pixels.split_at_mut(bottom * row_bytes);
            before_bottom[top * row_bytes..(top + 1) * row_bytes]
                .swap_with_slice(&mut from_bottom[..row_bytes]);
        }
    }
    RgbaImage::new(width, height, pixels)?.write_png(&options.output)?;
    if let Some(adapter) = adapter {
        println!("adapter={adapter}");
    }
    println!(
        "backend={} frame={} size={}x{} output={}",
        options.backend,
        options.frame,
        width,
        height,
        options.output.display()
    );
    Ok(())
}

#[cfg(any(feature = "native-webgpu-exact", feature = "browser-webgpu-exact"))]
fn replay_native_webgpu(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    #[cfg(feature = "native-webgpu-exact")]
    {
        // Link-host only: this does not create or invoke the C++ Dawn renderer.
        nuxie_renderer_ffi::dawn_link_anchor();
    }
    let mode = match mode {
        "msaa" => nuxie_renderer::RenderMode::Msaa,
        "clockwise-atomic" => nuxie_renderer::RenderMode::ClockwiseAtomic,
        value => return Err(format!("unsupported exact WebGPU mode `{value}`").into()),
    };
    let mut factory = nuxie_renderer::NativeWebGpuFactory::new(width, height)?;
    let adapter = factory.adapter_name().to_owned();
    let mut frame = factory.begin_frame(clear, mode)?;
    stream.replay_frame(frame_index, &mut factory, &mut frame)?;
    Ok((frame.finish()?, Some(adapter)))
}

#[cfg(feature = "native-vulkan-exact")]
fn replay_native_vulkan(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let mode = match mode {
        "msaa" => nuxie_renderer::RenderMode::Msaa,
        "clockwise-atomic" => nuxie_renderer::RenderMode::ClockwiseAtomic,
        value => return Err(format!("unsupported exact Vulkan mode `{value}`").into()),
    };
    let mut factory = nuxie_renderer::NativeVulkanFactory::new(width, height)?;
    let adapter = factory.adapter_name().to_owned();
    let mut frame = factory.begin_frame(clear, mode)?;
    stream.replay_frame(frame_index, &mut factory, &mut frame)?;
    Ok((frame.finish()?, Some(adapter)))
}

fn validate_backend_mode(backend: &str, mode: &str) -> Result<(), String> {
    if !matches!(mode, "msaa" | "clockwise-atomic") {
        return Err(format!("unsupported renderer mode `{mode}`"));
    }
    if matches!(backend, "ffi-metal" | "rust-metal" | "rust-metal-atomic") && mode == "msaa" {
        return Err(
            "native Metal does not implement `msaa`; upstream Metal selects raster-order or atomic execution"
                .to_owned(),
        );
    }
    Ok(())
}

#[cfg(all(feature = "native-metal", target_os = "macos"))]
fn replay_native_metal(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let mut factory = nuxie_renderer::NativeMetalFactory::new_with_context_options(
        width,
        height,
        nuxie_renderer::NativeMetalContextOptions {
            shader_compilation_mode: nuxie_renderer::ShaderCompilationMode::AlwaysSynchronous,
            disable_framebuffer_reads: false,
            ..Default::default()
        },
    )?;
    let adapter = factory.adapter_name();
    let mut frame = factory.begin_frame(clear)?;
    stream.replay_frame(frame_index, &mut factory, &mut frame)?;
    Ok((frame.finish()?, Some(adapter)))
}

#[cfg(all(feature = "native-metal", target_os = "macos"))]
fn replay_native_metal_atomic(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let mode = match mode {
        "clockwise-atomic" => nuxie_renderer::RenderMode::ClockwiseAtomic,
        value => return Err(format!("unsupported native Metal mode `{value}`").into()),
    };
    let mut factory = nuxie_renderer::NativeMetalFactory::new_with_mode_and_context_options(
        width,
        height,
        mode,
        nuxie_renderer::NativeMetalContextOptions {
            shader_compilation_mode: nuxie_renderer::ShaderCompilationMode::AlwaysSynchronous,
            disable_framebuffer_reads: false,
            ..Default::default()
        },
    )?;
    let adapter = factory.adapter_name();
    let mut frame = factory.begin_frame(clear)?;
    stream.replay_frame(frame_index, &mut factory, &mut frame)?;
    Ok((frame.finish()?, Some(adapter)))
}

#[cfg(all(feature = "ffi", target_os = "macos"))]
fn replay_ffi_metal(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let factory = nuxie_renderer_ffi::FfiFactory::new_metal(width, height)?;
    let adapter = factory.adapter_name()?;
    let mut pixels = replay_ffi(stream, frame_index, factory, clear, mode)?;
    flip_rows(&mut pixels, width, height);
    Ok((pixels, Some(adapter)))
}

#[cfg(all(
    feature = "perf-dawn",
    any(target_os = "macos", target_os = "emscripten")
))]
fn replay_ffi_dawn(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let factory = nuxie_renderer_ffi::FfiFactory::new_dawn(width, height)?;
    let adapter = factory.adapter_name()?;
    let mut pixels = replay_ffi(stream, frame_index, factory, clear, mode)?;
    #[cfg(target_os = "emscripten")]
    {
        // Complete TestingWindowWGPU::endFrame -> TestHarness::savePNG, whose
        // source-owned WritePNGFile(flipY=true) reverses the rows a second time.
        flip_rows(&mut pixels, width, height);
    }
    Ok((pixels, Some(adapter)))
}

#[cfg(all(feature = "ffi-vulkan", target_os = "macos"))]
fn replay_ffi_vulkan(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let factory = nuxie_renderer_ffi::FfiFactory::new_vulkan(width, height)?;
    let adapter = factory.adapter_name()?;
    Ok((
        replay_ffi(stream, frame_index, factory, clear, mode)?,
        Some(adapter),
    ))
}

#[cfg(all(feature = "ffi-webgl2", target_os = "emscripten"))]
fn replay_ffi_webgl2(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<(Vec<u8>, Option<String>), Box<dyn Error>> {
    let factory = nuxie_renderer_ffi::FfiFactory::new_webgl2(width, height)?;
    let adapter = factory.adapter_name()?;
    let mut pixels = replay_ffi(stream, frame_index, factory, clear, mode)?;
    flip_rows(&mut pixels, width, height);
    Ok((pixels, Some(adapter)))
}

#[cfg(any(
    all(feature = "ffi", target_os = "macos"),
    all(feature = "perf-dawn", target_os = "emscripten"),
    all(feature = "ffi-webgl2", target_os = "emscripten")
))]
fn replay_ffi(
    stream: &RenderStream,
    frame_index: usize,
    mut factory: nuxie_renderer_ffi::FfiFactory,
    clear: u32,
    mode: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    use nuxie_renderer_ffi::FfiRenderMode;
    let mode = match mode {
        "msaa" => FfiRenderMode::Msaa,
        "clockwise-atomic" => FfiRenderMode::ClockwiseAtomic,
        value => return Err(format!("unsupported renderer mode `{value}`").into()),
    };
    let mut frame = factory.begin_frame_with_mode(clear, mode)?;
    stream.replay_frame(frame_index, &mut factory, &mut frame)?;
    frame.end();
    Ok(factory.read_pixels()?)
}

#[cfg(any(feature = "ffi", test))]
fn flip_rows(pixels: &mut [u8], width: u32, height: u32) {
    let row_bytes = width as usize * 4;
    for row in 0..height as usize / 2 {
        let opposite = height as usize - row - 1;
        let (before, after) = pixels.split_at_mut(opposite * row_bytes);
        before[row * row_bytes..(row + 1) * row_bytes].swap_with_slice(&mut after[..row_bytes]);
    }
}

fn clear_pixels(width: u32, height: u32, color: u32) -> Vec<u8> {
    let rgba = color.to_be_bytes();
    let count = (width as usize).saturating_mul(height as usize);
    rgba.into_iter()
        .cycle()
        .take(count.saturating_mul(4))
        .collect()
}

fn apply_command_limit(
    stream: &mut RenderStream,
    frame_index: usize,
    limit: Option<usize>,
) -> Result<(), String> {
    let Some(limit) = limit else {
        return Ok(());
    };
    let frame = stream
        .frames
        .get_mut(frame_index)
        .ok_or_else(|| format!("render stream has no frame {frame_index}"))?;
    frame.commands.truncate(limit);
    Ok(())
}

fn parse_options() -> Result<Options, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut stream = None;
    let mut output = None;
    let mut backend = "stub".to_owned();
    let mut frame = 0;
    let mut clear = None;
    let mut mode = "msaa".to_owned();
    let mut command_limit = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--stream" => stream = Some(PathBuf::from(args.next().ok_or(usage())?)),
            "--output" => output = Some(PathBuf::from(args.next().ok_or(usage())?)),
            "--backend" => backend = args.next().ok_or(usage())?,
            "--frame" => frame = args.next().ok_or(usage())?.parse()?,
            "--clear" => {
                let value = args.next().ok_or(usage())?;
                clear = Some(u32::from_str_radix(value.trim_start_matches("0x"), 16)?);
            }
            "--mode" => mode = args.next().ok_or(usage())?,
            "--command-limit" => command_limit = Some(args.next().ok_or(usage())?.parse()?),
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
        }
    }
    Ok(Options {
        stream: stream.ok_or(usage())?,
        output: output.ok_or(usage())?,
        backend,
        frame,
        clear,
        mode,
        command_limit,
    })
}

fn usage() -> &'static str {
    "usage: renderer-replay --stream FILE --output FILE [--backend stub|rust-vulkan-exact|rust-webgpu-exact|rust-metal|rust-metal-atomic|ffi-metal|ffi-dawn|ffi-vulkan|ffi-webgl2] [--mode msaa|clockwise-atomic] [--frame N] [--command-limit N] [--clear 0xRRGGBBAA]"
}

#[cfg(test)]
mod tests {
    use super::{apply_command_limit, clear_pixels, flip_rows, validate_backend_mode};
    use nuxie_render_stream::RenderStream;

    #[test]
    fn stub_uses_requested_rgba_clear_color() {
        assert_eq!(
            clear_pixels(2, 1, 0x11223344),
            [0x11, 0x22, 0x33, 0x44, 0x11, 0x22, 0x33, 0x44]
        );
    }

    #[test]
    fn flips_native_readback_to_top_left_origin() {
        let mut pixels = vec![1; 8];
        pixels.extend([2; 8]);
        flip_rows(&mut pixels, 2, 2);
        assert_eq!(&pixels[..8], &[2; 8]);
        assert_eq!(&pixels[8..], &[1; 8]);
    }

    #[test]
    fn command_limit_truncates_only_the_selected_frame() {
        let mut stream = RenderStream::parse(
            "rive-golden-stream-v1\nframeSize width=1 height=1\nsave\nrestore\nframe\nsave\nframe\n",
        )
        .unwrap();
        apply_command_limit(&mut stream, 0, Some(1)).unwrap();
        assert_eq!(stream.frames[0].commands.len(), 1);
        assert_eq!(stream.frames[1].commands.len(), 1);
        assert_eq!(
            apply_command_limit(&mut stream, 2, Some(1)).unwrap_err(),
            "render stream has no frame 2"
        );
    }

    #[test]
    fn native_metal_rejects_the_webgpu_msaa_mode_before_replay() {
        let error = validate_backend_mode("ffi-metal", "msaa").unwrap_err();
        assert!(error.contains("native Metal does not implement `msaa`"));
        assert!(validate_backend_mode("rust-metal", "msaa").is_err());
        assert!(validate_backend_mode("rust-metal-atomic", "msaa").is_err());
        validate_backend_mode("ffi-metal", "clockwise-atomic").unwrap();
        validate_backend_mode("rust-metal", "clockwise-atomic").unwrap();
        validate_backend_mode("rust-metal-atomic", "clockwise-atomic").unwrap();
        validate_backend_mode("ffi-dawn", "msaa").unwrap();
    }
}
