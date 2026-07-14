use pixel_compare::RgbaImage;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut artifact = None;
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => artifact = Some(PathBuf::from(args.next().ok_or(usage())?)),
            "--output" => output = Some(PathBuf::from(args.next().ok_or(usage())?)),
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
        }
    }
    let artifact = artifact.ok_or(usage())?;
    let output = output.ok_or(usage())?;
    let image = RgbaImage::read_riveabl(&artifact)?;
    image.write_png(&output)?;
    println!(
        "converted RIVEABL {}x{} artifact={} output={}",
        image.width,
        image.height,
        artifact.display(),
        output.display()
    );
    Ok(())
}

fn usage() -> &'static str {
    "usage: riveabl-to-png --artifact FILE --output FILE"
}
