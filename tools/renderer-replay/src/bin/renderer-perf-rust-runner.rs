#[cfg(target_os = "macos")]
fn main() {
    if let Err(error) =
        renderer_replay::perf_runner::run(renderer_replay::perf_runner::BackendKind::RustWgpu)
    {
        eprintln!("renderer-perf-rust-runner error: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("renderer-perf-rust-runner requires macOS Metal");
    std::process::exit(1);
}
