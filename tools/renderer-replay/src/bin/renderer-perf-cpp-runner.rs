#[cfg(target_os = "macos")]
fn main() {
    if let Err(error) =
        renderer_replay::perf_runner::run(renderer_replay::perf_runner::BackendKind::CppDawn)
    {
        eprintln!("renderer-perf-cpp-runner error: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("renderer-perf-cpp-runner requires Dawn WebGPU on macOS Metal");
    std::process::exit(1);
}
