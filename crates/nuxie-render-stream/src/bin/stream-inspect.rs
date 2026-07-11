use nuxie_render_stream::RenderStream;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: stream-inspect <rive-golden-stream-v1 file>")?;
    let stream = RenderStream::parse(&fs::read_to_string(&path)?)?;
    let command_count: usize = stream.frames.iter().map(|frame| frame.commands.len()).sum();
    println!(
        "frames={} resources={} commands={} size={:?}",
        stream.frames.len(),
        stream.resources.len(),
        command_count,
        stream.frame_size
    );
    Ok(())
}
