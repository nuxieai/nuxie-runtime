//! `luau-vm-diff --corpus DIR [--snapshots DIR] [--record | --against-reference]
//! [--filter TEXT] [--verbose]`
//!
//! Runs every `DIR/*.luau` case under every compiler profile on
//! `nuxie-luau-vm` and compares each outcome with its committed snapshot
//! (`DIR/snapshots/<case>.<profile>.txt`, [`luau_vm_diff::snapshot`]), then
//! checks that the corpus reached every opcode the loader admits.
//!
//! The snapshots were recorded from the vendored VM, the differential's
//! original reference. Modes that need it:
//!
//! - `--record` writes every snapshot from the vendored VM's outcome.
//! - `--against-reference` runs both VMs and checks that the vendored VM's
//!   outcome equals its snapshot and that the safe VM's equals the vendored
//!   VM's (the proof that the snapshots froze the live reference).
//!
//! Exit 0 when every run agrees, no snapshot is missing or stale, and
//! coverage is complete; 1 otherwise; 2 on a usage error.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use luau_vm_diff::{
    OPCODE_NAMES, REJECTED_OPCODES, UNEMITTED_OPCODES, opcodes, run_case,
    snapshot_file_name,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// The safe VM against the snapshots.
    Snapshots,
    /// Write the snapshots from the vendored VM.
    Record,
    /// Both VMs live, and the vendored VM against the snapshots.
    AgainstReference,
}

fn main() -> ExitCode {
    let mut corpus: Option<PathBuf> = None;
    let mut snapshots: Option<PathBuf> = None;
    let mut filter: Option<String> = None;
    let mut verbose = false;
    let mut mode = Mode::Snapshots;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--corpus" => corpus = args.next().map(PathBuf::from),
            "--snapshots" => snapshots = args.next().map(PathBuf::from),
            "--filter" => filter = args.next(),
            "--verbose" => verbose = true,
            "--record" => mode = Mode::Record,
            "--against-reference" => mode = Mode::AgainstReference,
            other => {
                eprintln!("luau-vm-diff: unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(corpus) = corpus else {
        eprintln!(
            "usage: luau-vm-diff --corpus DIR [--snapshots DIR] [--record | --against-reference] [--filter TEXT] [--verbose]"
        );
        return ExitCode::from(2);
    };
    let snapshots = snapshots.unwrap_or_else(|| corpus.join("snapshots"));
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
    if mode == Mode::Record
        && let Err(error) = std::fs::create_dir_all(&snapshots)
    {
        eprintln!("luau-vm-diff: {}: {error}", snapshots.display());
        return ExitCode::from(2);
    }

    let mut failures = 0usize;
    let mut runs = 0usize;
    let mut recorded = 0usize;
    let mut versions = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut expected_files = BTreeSet::new();
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
        let results = match run_case(&source, mode != Mode::Snapshots) {
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
            let file_name = snapshot_file_name(&name, run.profile);
            let file = snapshots.join(&file_name);
            expected_files.insert(file_name);
            let label = format!("{name} [{}] v{}", run.profile.name(), run.version);
            let nuxie = run.nuxie_snapshot();
            match mode {
                Mode::Record => {
                    let Some(reference) = run.reference_snapshot() else {
                        println!("FAIL {label}: the vendored VM did not run");
                        failures += 1;
                        continue;
                    };
                    if let Err(error) = std::fs::write(&file, reference) {
                        println!("FAIL {label}: {}: {error}", file.display());
                        failures += 1;
                        continue;
                    }
                    recorded += 1;
                }
                Mode::Snapshots => {
                    if !check(&label, &file, "nuxie", &nuxie, verbose) {
                        failures += 1;
                    }
                }
                Mode::AgainstReference => {
                    let reference = run.reference_snapshot().unwrap_or_default();
                    let mut agrees = check(&label, &file, "luaur", &reference, verbose);
                    if reference != nuxie {
                        println!("FAIL {label}: the safe VM disagrees with the vendored VM");
                        report_lines(&reference, "luaur", &nuxie, "nuxie");
                        agrees = false;
                    }
                    if !agrees {
                        failures += 1;
                    }
                }
            }
        }
    }

    if filter.is_none() && mode != Mode::Record {
        failures += stale_snapshots(&snapshots, &expected_files);
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
    let against = match mode {
        Mode::Snapshots => "safe VM against the snapshots",
        Mode::Record => "snapshots recorded from the vendored VM",
        Mode::AgainstReference => "both VMs against the snapshots and each other",
    };
    println!(
        "luau-vm-diff: {runs} runs ({against}), {failures} disagreements, bytecode versions {}",
        versions.join(",")
    );
    if mode == Mode::Record {
        println!(
            "luau-vm-diff: wrote {recorded} snapshots to {}",
            snapshots.display()
        );
    }
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

/// Compares `actual` (the outcome of the VM named `who`) with the snapshot
/// in `file`; prints the difference and returns false when they differ or
/// the snapshot cannot be read.
fn check(label: &str, file: &Path, who: &str, actual: &str, verbose: bool) -> bool {
    match std::fs::read_to_string(file) {
        Ok(expected) if expected == actual => {
            if verbose {
                println!("ok   {label}");
            }
            true
        }
        Ok(expected) => {
            println!("FAIL {label}: {who} differs from {}", file.display());
            report_lines(&expected, "snapshot", actual, who);
            if verbose {
                println!("  full output snapshot: {:?}", printed(&expected));
                println!("  full output {who}: {:?}", printed(actual));
            }
            false
        }
        Err(error) => {
            println!("FAIL {label}: no snapshot {}: {error}", file.display());
            false
        }
    }
}

/// The `print` lines of a snapshot, as captured (still escaped).
fn printed(snapshot: &str) -> Vec<&str> {
    snapshot
        .lines()
        .filter_map(|line| line.strip_prefix("print \""))
        .map(|line| line.strip_suffix('"').unwrap_or(line))
        .collect()
}

/// The first line where two snapshots differ.
fn report_lines(expected: &str, expected_name: &str, actual: &str, actual_name: &str) {
    let expected: Vec<&str> = expected.lines().collect();
    let actual: Vec<&str> = actual.lines().collect();
    let first = expected
        .iter()
        .zip(&actual)
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    println!(
        "  differs at line {} ({} vs {} lines)",
        first + 1,
        expected.len(),
        actual.len()
    );
    println!("  {expected_name:>8}: {:?}", expected.get(first));
    println!("  {actual_name:>8}: {:?}", actual.get(first));
}

/// Snapshot files in `dir` that no case and profile produced: a renamed or
/// deleted case left them behind. Returns how many it reported.
fn stale_snapshots(dir: &Path, expected: &BTreeSet<String>) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut stale: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".txt") && !expected.contains(name))
        .collect();
    stale.sort();
    for name in &stale {
        println!("FAIL stale snapshot {}", dir.join(name).display());
    }
    stale.len()
}
