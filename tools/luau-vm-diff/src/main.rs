//! `luau-vm-diff --corpus DIR [--filter TEXT] [--verbose]`
//!
//! Runs every `DIR/*.luau` case under every compiler profile on the vendored
//! VM and on `nuxie-luau-vm`, prints each disagreement, then checks that the
//! corpus reached every opcode the loader admits. Exit 0 when every case
//! agrees and coverage is complete, 1 otherwise, 2 on a usage error.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use luau_vm_diff::{OPCODE_NAMES, Outcome, REJECTED_OPCODES, UNEMITTED_OPCODES, opcodes, run_case};

fn main() -> ExitCode {
    let mut corpus: Option<PathBuf> = None;
    let mut filter: Option<String> = None;
    let mut verbose = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--corpus" => corpus = args.next().map(PathBuf::from),
            "--filter" => filter = args.next(),
            "--verbose" => verbose = true,
            other => {
                eprintln!("luau-vm-diff: unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(corpus) = corpus else {
        eprintln!("usage: luau-vm-diff --corpus DIR [--filter TEXT] [--verbose]");
        return ExitCode::from(2);
    };
    let mut cases: Vec<PathBuf> = match std::fs::read_dir(&corpus) {
        Ok(entries) => entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "luau"))
            .collect(),
        Err(error) => {
            eprintln!("luau-vm-diff: {}: {error}", corpus.display());
            return ExitCode::from(2);
        }
    };
    cases.sort();

    let mut failures = 0usize;
    let mut runs = 0usize;
    let mut versions = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for path in &cases {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if filter.as_ref().is_some_and(|f| !name.contains(f.as_str())) {
            continue;
        }
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                println!("FAIL {name}: {error}");
                failures += 1;
                continue;
            }
        };
        let results = match run_case(&source) {
            Ok(results) => results,
            Err(error) => {
                println!("FAIL {name}: compile: {error}");
                failures += 1;
                continue;
            }
        };
        for run in &results {
            runs += 1;
            versions.insert(run.version);
            for bytecode in &run.bytecodes {
                match opcodes(bytecode) {
                    Ok(ops) => seen.extend(ops),
                    Err(error) => {
                        println!("note {name} [{}]: opcodes: {error}", run.profile.name())
                    }
                }
            }
            if run.agrees() {
                if verbose {
                    println!("ok   {name} [{}] v{}", run.profile.name(), run.version);
                }
            } else {
                failures += 1;
                println!("FAIL {name} [{}] v{}", run.profile.name(), run.version);
                report(&run.reference, &run.nuxie);
            }
        }
    }

    let missing: Vec<&str> = OPCODE_NAMES
        .iter()
        .enumerate()
        .filter(|(op, name)| {
            !REJECTED_OPCODES.contains(name)
                && !UNEMITTED_OPCODES.contains(name)
                && !seen.contains(&u8::try_from(*op).unwrap_or(0))
        })
        .map(|(_, name)| *name)
        .collect();
    let versions: Vec<String> = versions.iter().map(u8::to_string).collect();
    println!(
        "luau-vm-diff: {runs} runs, {failures} disagreements, bytecode versions {}",
        versions.join(",")
    );
    if filter.is_none() {
        if missing.is_empty() {
            println!("luau-vm-diff: the corpus reaches every admitted opcode");
        } else {
            println!(
                "luau-vm-diff: the corpus does not reach {}",
                missing.join(" ")
            );
        }
    }
    if failures == 0 && (filter.is_some() || missing.is_empty()) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn report(reference: &Outcome, nuxie: &Outcome) {
    if reference.returns != nuxie.returns {
        println!("  returns  luaur: {:?}", reference.returns);
        println!("  returns  nuxie: {:?}", nuxie.returns);
    }
    if reference.error != nuxie.error {
        println!("  error    luaur: {:?}", reference.error);
        println!("  error    nuxie: {:?}", nuxie.error);
    }
    if reference.output != nuxie.output {
        let first = reference
            .output
            .iter()
            .zip(&nuxie.output)
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| reference.output.len().min(nuxie.output.len()));
        println!(
            "  output differs at line {} ({} vs {} lines)",
            first + 1,
            reference.output.len(),
            nuxie.output.len()
        );
        println!("  output   luaur: {:?}", reference.output.get(first));
        println!("  output   nuxie: {:?}", nuxie.output.get(first));
    }
}
