fn main() {
    println!("cargo:rerun-if-changed=src/apple/player.m");
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if matches!(target.as_str(), "macos" | "ios" | "tvos" | "visionos") {
        cc::Build::new()
            .file("src/apple/player.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .compile("nuxie_video_apple");
        for framework in [
            "Foundation",
            "AVFoundation",
            "CoreMedia",
            "CoreVideo",
            "QuartzCore",
        ] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
}
