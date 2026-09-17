fn main() {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if let Some(source) = std::env::args().nth(1) {
        video_qualification::run_metal_benchmark(&source);
        return;
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    video_qualification::run_metal_proof(&format!(
        "{}/../../crates/nuxie-video-host/tests/fixtures/red-blue-audio.mp4",
        env!("CARGO_MANIFEST_DIR")
    ));
}
