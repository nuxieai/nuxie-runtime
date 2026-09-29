fn main() {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if let Some(source) = std::env::args().nth(1) {
        video_qualification::run_metal_benchmark(&source);
        return;
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/video");
        video_qualification::run_metal_parity(&format!("{fixtures}/parity-opaque.mp4"));
        video_qualification::run_metal_proof(&format!("{fixtures}/red-blue-audio.mp4"));
    }
}
