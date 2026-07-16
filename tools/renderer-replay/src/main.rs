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
    mode: String,
    command_limit: Option<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = parse_options()?;
    let mut stream = RenderStream::parse(&fs::read_to_string(&options.stream)?)?;
    apply_command_limit(&mut stream, options.frame, options.command_limit)?;
    let (width, height) = stream
        .frame_size
        .ok_or("recorded stream does not declare frameSize")?;
    let clear = options.clear.or(stream.clear_color).unwrap_or(0);
    let pixels = match options.backend.as_str() {
        "stub" => clear_pixels(width, height, clear),
        "rust-wgpu" => replay_wgpu(&stream, options.frame, width, height, clear, &options.mode)?,
        #[cfg(all(feature = "ffi", target_os = "macos"))]
        "ffi-metal" => {
            replay_ffi_metal(&stream, options.frame, width, height, clear, &options.mode)?
        }
        #[cfg(all(feature = "perf-dawn", target_os = "macos"))]
        "ffi-dawn" => replay_ffi_dawn(&stream, options.frame, width, height, clear, &options.mode)?,
        backend => {
            return Err(format!(
                "backend `{backend}` is unavailable; use `stub`, `rust-wgpu`{}{}",
                if cfg!(all(feature = "ffi", target_os = "macos")) {
                    " or `ffi-metal`"
                } else {
                    ""
                },
                if cfg!(all(feature = "perf-dawn", target_os = "macos")) {
                    " or `ffi-dawn`"
                } else {
                    ""
                }
            )
            .into());
        }
    };
    RgbaImage::new(width, height, pixels)?.write_png(&options.output)?;
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

fn replay_wgpu(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mode = match mode {
        "msaa" => nuxie_renderer::RenderMode::Msaa,
        "clockwise-atomic" => nuxie_renderer::RenderMode::ClockwiseAtomic,
        value => return Err(format!("unsupported renderer mode `{value}`").into()),
    };
    let mut factory = nuxie_renderer::WgpuFactory::new_with_mode(width, height, mode)?;
    let mut frame = factory.begin_frame(clear);
    stream.replay_frame(frame_index, &mut factory, &mut frame)?;
    Ok(frame.finish()?)
}

#[cfg(all(feature = "ffi", target_os = "macos"))]
fn replay_ffi_metal(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let factory = nuxie_renderer_ffi::FfiFactory::new_metal(width, height)?;
    let mut pixels = replay_ffi(stream, frame_index, factory, clear, mode)?;
    flip_rows(&mut pixels, width, height);
    Ok(pixels)
}

#[cfg(all(feature = "perf-dawn", target_os = "macos"))]
fn replay_ffi_dawn(
    stream: &RenderStream,
    frame_index: usize,
    width: u32,
    height: u32,
    clear: u32,
    mode: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let factory = nuxie_renderer_ffi::FfiFactory::new_dawn(width, height)?;
    replay_ffi(stream, frame_index, factory, clear, mode)
}

#[cfg(all(feature = "ffi", target_os = "macos"))]
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
    "usage: renderer-replay --stream FILE --output FILE [--backend stub|rust-wgpu|ffi-metal|ffi-dawn] [--mode msaa|clockwise-atomic] [--frame N] [--command-limit N] [--clear 0xRRGGBBAA]"
}

#[cfg(test)]
mod tests {
    use super::{apply_command_limit, clear_pixels, flip_rows};
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
}
