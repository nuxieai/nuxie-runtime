use nuxie_html_to_riv::{CompileInput, Diagnostic, compile};
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
    let input: CompileInput = serde_json::from_slice(&fs::read(&args[0])?).map_err(|error| {
        serde_json::to_string(&vec![Diagnostic::new("invalid-request", "request", error.to_string())])
            .expect("serializable request diagnostic")
    })?;
    let output = compile(&input).map_err(|diagnostics| {
        serde_json::to_string(&diagnostics).expect("serializable compiler diagnostics")
    })?;
    let path = PathBuf::from(&args[1]);
    let map = serde_json::to_vec_pretty(&output.source_map)?;
    // Compiler success produces ordinary Rive bytes plus optional authoring
    // metadata. Loading the Rive file must not depend on the source map.
    fs::write(path.with_extension("map.json"), map)?;
    fs::write(path, output.riv)?;
    Ok(())
}
