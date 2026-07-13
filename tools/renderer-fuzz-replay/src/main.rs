use anyhow::{Context, Result, bail};
use nuxie_render_stream::RenderStream;
use pixel_compare::RgbaImage;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 64;
const CONTROL_COLOR: [u8; 4] = [0x22, 0xcc, 0x44, 0xff];
const CONTROL_RECT: Rect = Rect {
    left: 52,
    top: 52,
    right: 57,
    bottom: 57,
};
const CONTROL_FOOTPRINT: Rect = Rect {
    left: 47,
    top: 47,
    right: 63,
    bottom: 63,
};

#[derive(Clone, Copy)]
struct Rect {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

struct Options {
    replay: PathBuf,
    output_dir: PathBuf,
    timeout: Duration,
    mode: String,
}

struct Case {
    id: &'static str,
    name: &'static str,
    stream: String,
    expectation: PixelExpectation,
}

#[derive(Clone, Copy)]
enum PixelExpectation {
    Exact,
    Bounded {
        max_different_pixels: u64,
        max_channel_delta: u8,
    },
    NamedDelta {
        max_different_pixels: u64,
        max_channel_delta: u8,
    },
}

struct BackendOutput {
    image: RgbaImage,
    elapsed: Duration,
}

fn main() -> Result<()> {
    let options = parse_options()?;
    fs::create_dir_all(&options.output_dir).with_context(|| {
        format!(
            "create fuzz-replay output directory {}",
            options.output_dir.display()
        )
    })?;

    let cases = cases();
    let control = control_case();
    let control_stream_path = options.output_dir.join("control.stream");
    fs::write(&control_stream_path, &control.stream)
        .with_context(|| format!("write {}", control_stream_path.display()))?;
    let rust_control = run_backend(&options, &control, &control_stream_path, "rust-wgpu")?;
    let cpp_control = run_backend(&options, &control, &control_stream_path, "ffi-metal")?;
    for case in &cases {
        RenderStream::parse(&case.stream)
            .with_context(|| format!("generated stream {} is invalid", case.name))?;
        let stream_path = options.output_dir.join(format!("{}.stream", case.name));
        fs::write(&stream_path, &case.stream)
            .with_context(|| format!("write {}", stream_path.display()))?;

        let rust = run_backend(&options, case, &stream_path, "rust-wgpu")?;
        let cpp = run_backend(&options, case, &stream_path, "ffi-metal")?;
        require_matching_region(
            case,
            "rust-wgpu",
            &rust_control.image,
            &rust.image,
            CONTROL_FOOTPRINT,
        )?;
        require_matching_region(
            case,
            "ffi-metal",
            &cpp_control.image,
            &cpp.image,
            CONTROL_FOOTPRINT,
        )?;
        let (different_pixels, max_channel_delta) =
            compare_outside_control(&rust.image, &cpp.image, CONTROL_FOOTPRINT)?;
        let relation = require_pixel_expectation(case, different_pixels, max_channel_delta)?;
        println!(
            "case={} finding={} result=pass relation={} rust_ms={} cpp_ms={} non_control_different_pixels={} non_control_max_channel_delta={}",
            case.name,
            case.id,
            relation,
            rust.elapsed.as_millis(),
            cpp.elapsed.as_millis(),
            different_pixels,
            max_channel_delta,
        );
    }
    println!(
        "renderer-fuzz-replay: pass cases={} backends=2 mode={}",
        cases.len(),
        options.mode
    );
    Ok(())
}

fn require_pixel_expectation(
    case: &Case,
    different_pixels: u64,
    max_channel_delta: u8,
) -> Result<&'static str> {
    match case.expectation {
        PixelExpectation::Exact if different_pixels == 0 => Ok("exact"),
        PixelExpectation::Exact => bail!(
            "{} ({}) expected exact hostile-region parity, got {} pixels/max delta {}",
            case.name,
            case.id,
            different_pixels,
            max_channel_delta,
        ),
        PixelExpectation::Bounded {
            max_different_pixels,
            max_channel_delta: allowed_delta,
        } if different_pixels <= max_different_pixels && max_channel_delta <= allowed_delta => {
            Ok("bounded")
        }
        PixelExpectation::Bounded {
            max_different_pixels,
            max_channel_delta: allowed_delta,
        } => bail!(
            "{} ({}) exceeded its pixel boundary: {} pixels/max delta {}, allowed {}/{}",
            case.name,
            case.id,
            different_pixels,
            max_channel_delta,
            max_different_pixels,
            allowed_delta,
        ),
        PixelExpectation::NamedDelta { .. } if different_pixels == 0 => Ok("resolved"),
        PixelExpectation::NamedDelta {
            max_different_pixels,
            max_channel_delta: allowed_delta,
        } if different_pixels <= max_different_pixels && max_channel_delta <= allowed_delta => {
            Ok("named-delta")
        }
        PixelExpectation::NamedDelta {
            max_different_pixels,
            max_channel_delta: allowed_delta,
        } => bail!(
            "{} ({}) exceeded its named-delta boundary: {} pixels/max delta {}, allowed {}/{}",
            case.name,
            case.id,
            different_pixels,
            max_channel_delta,
            max_different_pixels,
            allowed_delta,
        ),
    }
}

fn run_backend(
    options: &Options,
    case: &Case,
    stream_path: &Path,
    backend: &str,
) -> Result<BackendOutput> {
    let output = options
        .output_dir
        .join(format!("{}-{backend}.png", case.name));
    let stdout_path = options
        .output_dir
        .join(format!("{}-{backend}.stdout", case.name));
    let stderr_path = options
        .output_dir
        .join(format!("{}-{backend}.stderr", case.name));
    match fs::remove_file(&output) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("remove stale {}", output.display()));
        }
    }
    let stdout =
        File::create(&stdout_path).with_context(|| format!("create {}", stdout_path.display()))?;
    let stderr =
        File::create(&stderr_path).with_context(|| format!("create {}", stderr_path.display()))?;

    let started = Instant::now();
    let mut child = Command::new(&options.replay)
        .arg("--stream")
        .arg(stream_path)
        .arg("--output")
        .arg(&output)
        .arg("--backend")
        .arg(backend)
        .arg("--mode")
        .arg(&options.mode)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| format!("start {backend} replay for {}", case.name))?;
    let status = wait_with_timeout(&mut child, options.timeout)?;
    let elapsed = started.elapsed();
    let Some(status) = status else {
        let _ = child.kill();
        let _ = child.wait();
        bail!(
            "{} ({}) timed out after {} ms; stderr: {}",
            case.name,
            backend,
            options.timeout.as_millis(),
            read_log(&stderr_path),
        );
    };
    if !status.success() {
        bail!(
            "{} ({}) exited with {}; stderr: {}",
            case.name,
            backend,
            display_status(status),
            read_log(&stderr_path),
        );
    }

    let image = RgbaImage::read_png(&output)
        .with_context(|| format!("{} ({backend}) emitted an invalid PNG", case.name))?;
    require_control_region(case, backend, &image)?;
    Ok(BackendOutput { image, elapsed })
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Result<Option<ExitStatus>> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().context("poll replay child")? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn require_control_region(case: &Case, backend: &str, image: &RgbaImage) -> Result<()> {
    if (image.width, image.height) != (WIDTH, HEIGHT) {
        bail!(
            "{} ({backend}) emitted {}x{}, expected {WIDTH}x{HEIGHT}",
            case.name,
            image.width,
            image.height,
        );
    }
    for y in CONTROL_RECT.top..CONTROL_RECT.bottom {
        for x in CONTROL_RECT.left..CONTROL_RECT.right {
            let offset = ((y * image.width + x) * 4) as usize;
            let actual = &image.pixels[offset..offset + 4];
            if actual != CONTROL_COLOR {
                bail!(
                    "{} ({backend}) corrupted finite control pixel ({x},{y}): expected {:02x?}, got {:02x?}",
                    case.name,
                    CONTROL_COLOR,
                    actual,
                );
            }
        }
    }
    Ok(())
}

fn require_matching_region(
    case: &Case,
    backend: &str,
    expected: &RgbaImage,
    actual: &RgbaImage,
    region: Rect,
) -> Result<()> {
    if (expected.width, expected.height) != (actual.width, actual.height) {
        bail!(
            "{} ({backend}) control baseline dimensions differ",
            case.name
        );
    }
    for y in region.top..region.bottom {
        for x in region.left..region.right {
            let offset = ((y * actual.width + x) * 4) as usize;
            let expected_pixel = &expected.pixels[offset..offset + 4];
            let actual_pixel = &actual.pixels[offset..offset + 4];
            if expected_pixel != actual_pixel {
                bail!(
                    "{} ({backend}) changed control-footprint pixel ({x},{y}): expected {:02x?}, got {:02x?}",
                    case.name,
                    expected_pixel,
                    actual_pixel,
                );
            }
        }
    }
    Ok(())
}

fn compare_outside_control(rust: &RgbaImage, cpp: &RgbaImage, ignored: Rect) -> Result<(u64, u8)> {
    if (rust.width, rust.height) != (cpp.width, cpp.height) {
        bail!("Rust and C++ fuzz replay dimensions differ");
    }
    let mut different_pixels = 0;
    let mut max_channel_delta = 0;
    for y in 0..rust.height {
        for x in 0..rust.width {
            if x >= ignored.left && x < ignored.right && y >= ignored.top && y < ignored.bottom {
                continue;
            }
            let offset = ((y * rust.width + x) * 4) as usize;
            let rust_pixel = &rust.pixels[offset..offset + 4];
            let cpp_pixel = &cpp.pixels[offset..offset + 4];
            let delta = rust_pixel
                .iter()
                .zip(cpp_pixel)
                .map(|(rust, cpp)| rust.abs_diff(*cpp))
                .max()
                .unwrap_or(0);
            if delta != 0 {
                different_pixels += 1;
                max_channel_delta = max_channel_delta.max(delta);
            }
        }
    }
    Ok((different_pixels, max_channel_delta))
}

fn parse_options() -> Result<Options> {
    let mut args = std::env::args().skip(1);
    let mut replay = None;
    let mut output_dir = PathBuf::from("target/renderer-fuzz-replay");
    let mut timeout = Duration::from_secs(20);
    let mut mode = String::from("clockwise-atomic");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--replay" => replay = Some(PathBuf::from(args.next().context(usage())?)),
            "--output-dir" => output_dir = PathBuf::from(args.next().context(usage())?),
            "--timeout-seconds" => {
                timeout = Duration::from_secs(args.next().context(usage())?.parse()?);
            }
            "--mode" => mode = args.next().context(usage())?,
            _ => bail!("unknown argument `{arg}`\n{}", usage()),
        }
    }
    if mode != "msaa" && mode != "clockwise-atomic" {
        bail!("unsupported renderer mode `{mode}`");
    }
    Ok(Options {
        replay: replay.context(usage())?,
        output_dir,
        timeout,
        mode,
    })
}

fn usage() -> &'static str {
    "usage: renderer-fuzz-replay --replay FILE [--output-dir DIR] [--timeout-seconds N] [--mode msaa|clockwise-atomic]"
}

fn read_log(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| format!("<could not read {}: {error}>", path.display()))
        .trim()
        .to_owned()
}

fn display_status(status: ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| String::from("signal"))
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            id: "R3-FZ-01",
            name: "nonfinite-transform",
            stream: nonfinite_transform_stream(),
            expectation: PixelExpectation::Exact,
        },
        Case {
            id: "R3-FZ-02",
            name: "degenerate-geometry",
            stream: degenerate_geometry_stream(),
            expectation: PixelExpectation::Exact,
        },
        Case {
            id: "R3-FZ-03",
            name: "absurd-stroke",
            stream: absurd_stroke_stream(),
            expectation: PixelExpectation::NamedDelta {
                max_different_pixels: 1_024,
                max_channel_delta: 255,
            },
        },
        Case {
            id: "R3-FZ-04",
            name: "deep-clips",
            stream: deep_clips_stream(),
            expectation: PixelExpectation::Bounded {
                max_different_pixels: 32,
                max_channel_delta: 1,
            },
        },
        Case {
            id: "R3-FZ-05",
            name: "hostile-gradients",
            stream: hostile_gradients_stream(),
            expectation: PixelExpectation::NamedDelta {
                max_different_pixels: 384,
                max_channel_delta: 255,
            },
        },
    ]
}

fn control_case() -> Case {
    Case {
        id: "R3-FZ-CONTROL",
        name: "control",
        stream: stream("", ""),
        expectation: PixelExpectation::Exact,
    }
}

fn stream(resources: &str, commands: &str) -> String {
    format!(
        "rive-golden-stream-v1\nframeSize width={WIDTH} height={HEIGHT}\nclearColor value=0x000000ff\n{resources}{commands}{}frame\n",
        control_draw()
    )
}

fn path(points: &str, verbs: &str) -> String {
    format!("{{id=1,fillRule=0,path={{verbs=[{verbs}],points=[{points}]}}}}")
}

fn paint(style: &str, thickness: &str, shader: u64, color: &str) -> String {
    format!(
        "{{id=1,style={style},color={color},thickness={thickness},join=0,cap=0,feather=0,blendMode=3,shader={shader}}}"
    )
}

fn draw(path: &str, paint: &str) -> String {
    format!("drawPath path={path} paint={paint}\n")
}

fn control_draw() -> String {
    draw(
        &path(
            "(48,48),(61,48),(61,61),(48,61)",
            "move,line,line,line,close",
        ),
        &paint("fill", "1", 0, "0xff22cc44"),
    )
}

fn hostile_triangle(paint_value: &str) -> String {
    draw(
        &path("(4,4),(28,4),(4,28)", "move,line,line,close"),
        paint_value,
    )
}

fn nonfinite_transform_stream() -> String {
    let fill = paint("fill", "1", 0, "0xff3355ff");
    let mut commands = String::new();
    for matrix in [
        "[NaN,0,0,1,0,0]",
        "[inf,0,0,1,0,0]",
        "[-inf,0,0,1,0,0]",
        "[3.4028235e38,0,0,3.4028235e38,0,0]",
    ] {
        commands.push_str("save\n");
        commands.push_str(&format!("transform matrix={matrix}\n"));
        commands.push_str(&hostile_triangle(&fill));
        commands.push_str("restore\n");
    }
    stream("", &commands)
}

fn degenerate_geometry_stream() -> String {
    let fill = paint("fill", "1", 0, "0xff3355ff");
    let mut commands = String::new();
    commands.push_str(&draw(&path("(8,8)", "move"), &fill));
    commands.push_str(&draw(&path("(12,12),(12,12)", "move,line"), &fill));
    commands.push_str(&draw(
        &path("(16,16),(16,16),(16,16)", "move,line,line,close"),
        &fill,
    ));
    stream("", &commands)
}

fn absurd_stroke_stream() -> String {
    let line = path("(4,4),(28,28)", "move,line");
    let mut commands = String::new();
    for thickness in ["0", "-3.4028235e38", "3.4028235e38", "NaN", "inf"] {
        commands.push_str(&draw(&line, &paint("stroke", thickness, 0, "0xff3355ff")));
    }
    stream("", &commands)
}

fn deep_clips_stream() -> String {
    let clip = path("(0,0),(32,0),(32,32),(0,32)", "move,line,line,line,close");
    let mut commands = String::new();
    for _ in 0..64 {
        commands.push_str("save\n");
        commands.push_str(&format!("clipPath path={clip}\n"));
    }
    commands.push_str(&hostile_triangle(&paint("fill", "1", 0, "0xff3355ff")));
    for _ in 0..64 {
        commands.push_str("restore\n");
    }
    stream("", &commands)
}

fn hostile_gradients_stream() -> String {
    let resources = [
        "makeLinearGradient id=1 start=(0,0) end=(32,0) stops=[]\n",
        "makeLinearGradient id=2 start=(0,0) end=(32,0) stops=[{color=0xff0000ff,stop=0.5},{color=0x0000ffff,stop=0.5}]\n",
        "makeLinearGradient id=3 start=(0,0) end=(32,0) stops=[{color=0xff0000ff,stop=0.8},{color=0x0000ffff,stop=0.2}]\n",
        "makeLinearGradient id=4 start=(0,0) end=(32,0) stops=[{color=0xff0000ff,stop=-1},{color=0x0000ffff,stop=0.5}]\n",
        "makeLinearGradient id=5 start=(0,0) end=(32,0) stops=[{color=0xff0000ff,stop=0.5},{color=0x0000ffff,stop=2}]\n",
        "makeLinearGradient id=6 start=(0,0) end=(32,0) stops=[{color=0xff0000ff,stop=0},{color=0x0000ffff,stop=NaN}]\n",
        "makeRadialGradient id=7 center=(16,16) radius=inf stops=[{color=0xff0000ff,stop=0},{color=0x0000ffff,stop=inf}]\n",
    ]
    .concat();
    let mut commands = String::new();
    for shader in 1..=7 {
        commands.push_str(&hostile_triangle(&paint("fill", "1", shader, "0xffffffff")));
    }
    stream(&resources, &commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_generated_case_parses_and_ends_with_control_draw() {
        for case in cases() {
            let stream = RenderStream::parse(&case.stream).unwrap();
            assert_eq!(stream.frame_size, Some((WIDTH, HEIGHT)), "{}", case.name);
            assert_eq!(stream.frames.len(), 1, "{}", case.name);
            assert!(case.stream.ends_with("frame\n"), "{}", case.name);
            assert!(case.stream.contains("color=0xff22cc44"), "{}", case.name);
        }
    }

    #[test]
    fn control_oracle_checks_every_pixel_in_reserved_region() {
        let mut image = RgbaImage::new(
            WIDTH,
            HEIGHT,
            CONTROL_COLOR
                .into_iter()
                .cycle()
                .take((WIDTH * HEIGHT * 4) as usize)
                .collect(),
        )
        .unwrap();
        let case = &cases()[0];
        require_control_region(case, "test", &image).unwrap();
        let offset = ((CONTROL_RECT.top * WIDTH + CONTROL_RECT.left) * 4) as usize;
        image.pixels[offset] ^= 1;
        assert!(require_control_region(case, "test", &image).is_err());
    }

    #[test]
    fn comparison_ignores_only_the_reserved_control_region() {
        let black = vec![0; (WIDTH * HEIGHT * 4) as usize];
        let mut changed = black.clone();
        let inside = ((CONTROL_RECT.top * WIDTH + CONTROL_RECT.left) * 4) as usize;
        changed[inside] = 255;
        let outside = ((CONTROL_RECT.top * WIDTH + CONTROL_RECT.left - 1) * 4) as usize;
        changed[outside] = 7;
        let black = RgbaImage::new(WIDTH, HEIGHT, black).unwrap();
        let changed = RgbaImage::new(WIDTH, HEIGHT, changed).unwrap();
        assert_eq!(
            compare_outside_control(&black, &changed, CONTROL_RECT).unwrap(),
            (1, 7)
        );
    }

    #[test]
    fn full_control_footprint_is_compared_to_backend_baseline() {
        let pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
        let expected = RgbaImage::new(WIDTH, HEIGHT, pixels.clone()).unwrap();
        let mut actual = RgbaImage::new(WIDTH, HEIGHT, pixels).unwrap();
        let case = &cases()[0];
        require_matching_region(case, "test", &expected, &actual, CONTROL_FOOTPRINT).unwrap();

        let offset = ((CONTROL_FOOTPRINT.top * WIDTH + CONTROL_FOOTPRINT.left) * 4) as usize;
        actual.pixels[offset] = 1;
        assert!(
            require_matching_region(case, "test", &expected, &actual, CONTROL_FOOTPRINT).is_err()
        );
    }

    #[test]
    fn pixel_expectations_fail_closed() {
        let exact = &cases()[0];
        assert_eq!(require_pixel_expectation(exact, 0, 0).unwrap(), "exact");
        assert!(require_pixel_expectation(exact, 1, 1).is_err());

        let bounded = &cases()[3];
        assert_eq!(
            require_pixel_expectation(bounded, 32, 1).unwrap(),
            "bounded"
        );
        assert!(require_pixel_expectation(bounded, 33, 1).is_err());
        assert!(require_pixel_expectation(bounded, 1, 2).is_err());

        let named = &cases()[2];
        assert_eq!(
            require_pixel_expectation(named, 1_024, 255).unwrap(),
            "named-delta"
        );
        assert!(require_pixel_expectation(named, 1_025, 255).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn child_wait_has_a_hard_deadline() {
        let mut child = Command::new("sh").arg("-c").arg("sleep 5").spawn().unwrap();
        let started = Instant::now();
        assert!(
            wait_with_timeout(&mut child, Duration::from_millis(30))
                .unwrap()
                .is_none()
        );
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
