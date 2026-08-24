#[cfg(target_os = "emscripten")]
fn main() {
    use nuxie_renderer::{NativeWebGpuFactory, RenderMode};

    let factory = match NativeWebGpuFactory::new(64, 64) {
        Ok(factory) => factory,
        Err(error) => {
            eprintln!("exact WebGPU factory failed: {error}");
            std::process::exit(1);
        }
    };
    println!("exact WebGPU adapter: {}", factory.adapter_name());
    let frame = match factory.begin_frame(0xff10_2030, RenderMode::Msaa) {
        Ok(frame) => frame,
        Err(error) => {
            eprintln!("exact WebGPU begin frame failed: {error}");
            std::process::exit(2);
        }
    };
    match frame.finish() {
        Ok(pixels)
            if pixels.len() == 64 * 64 * 4
                && pixels
                    .chunks_exact(4)
                    .all(|pixel| pixel == [0x10, 0x20, 0x30, 0xff]) =>
        {
            println!("exact WebGPU browser root passed");
        }
        Ok(pixels) if pixels.len() != 64 * 64 * 4 => {
            eprintln!("exact WebGPU pixel length mismatch: {}", pixels.len());
            std::process::exit(3);
        }
        Ok(pixels) => {
            eprintln!(
                "exact WebGPU clear mismatch: first pixel {:?}",
                pixels.get(..4)
            );
            std::process::exit(3);
        }
        Err(error) => {
            eprintln!("exact WebGPU finish frame failed: {error}");
            std::process::exit(4);
        }
    }
}

#[cfg(not(target_os = "emscripten"))]
fn main() {}
