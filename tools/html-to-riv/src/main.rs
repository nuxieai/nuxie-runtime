use nuxie_html_to_riv::{CompileInput, compile};
use std::{error::Error, fs, path::PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: html-to-riv <input.json> <output.riv>".into());
    }
    let input: CompileInput = serde_json::from_slice(&fs::read(&args[0])?)?;
    let output = compile(&input)
        .map_err(|diagnostics| serde_json::to_string_pretty(&diagnostics).unwrap_or_default())?;
    let path = PathBuf::from(&args[1]);
    let map = serde_json::to_vec_pretty(&output.source_map)?;
    fs::write(path.with_extension("map.json"), map)?;
    fs::write(
        path.with_extension("requirements.json"),
        serde_json::to_vec_pretty(&output.runtime_requirements)?,
    )?;
    fs::write(path, output.riv)?;
    Ok(())
}
