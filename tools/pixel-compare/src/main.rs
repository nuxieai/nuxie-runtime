use pixel_compare::{artifact, compare, RgbaImage, Tolerance};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let expected = PathBuf::from(args.next().ok_or(usage())?);
    let actual = PathBuf::from(args.next().ok_or(usage())?);
    let mut tolerance = Tolerance::EXACT;
    let mut artifact_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-channel-delta" => {
                tolerance.max_channel_delta = args.next().ok_or(usage())?.parse()?;
            }
            "--max-different-pixels" => {
                tolerance.max_different_pixels = args.next().ok_or(usage())?.parse()?;
            }
            "--artifact" => artifact_path = Some(PathBuf::from(args.next().ok_or(usage())?)),
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
        }
    }

    let expected_image = RgbaImage::read_png(&expected)?;
    let actual_image = RgbaImage::read_png(&actual)?;
    let report = compare(&expected_image, &actual_image, tolerance)?;
    println!(
        "within-tolerance={} different-pixels={} max-channel-delta={} size={}x{}",
        report.within_tolerance,
        report.different_pixels,
        report.max_channel_delta,
        report.width,
        report.height
    );
    if !report.within_tolerance {
        if let Some(path) = artifact_path {
            artifact(&expected_image, &actual_image)?.write_png(path)?;
        }
        std::process::exit(1);
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: pixel-compare <expected.png> <actual.png> [--max-channel-delta N] [--max-different-pixels N] [--artifact diff.png]"
}
