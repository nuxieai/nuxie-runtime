#[cfg(any(target_os = "ios", target_os = "macos"))]
use std::{env, path::PathBuf};

#[cfg(any(target_os = "ios", target_os = "macos"))]
use anyhow::{Context, Result, bail};

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args
        .next()
        .context("usage: apple-msl-catalog <generate|check> [root]")?;
    let root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    if args.next().is_some() {
        bail!("unexpected extra argument");
    }
    match command.as_str() {
        "generate" => apple_msl_catalog::generate(&root),
        "check" => apple_msl_catalog::check(&root),
        _ => bail!("unknown command {command:?}; expected generate or check"),
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn main() {
    eprintln!("apple-msl-catalog must run on an Apple host");
    std::process::exit(2);
}
