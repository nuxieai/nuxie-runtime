//! `luau-bench run`: run the merged upstream bench scripts in-process.
//! `luau-bench compare`: check a results file against a baseline.
//!
//! Exit status: 0 pass, 1 ratchet failure (a bench or the geomean over the
//! ceiling, a baseline bench missing, a failed script, or a loaded host
//! under `--max-load`), 2 usage or I/O error.

use luau_bench::vm;
use luau_bench::{
    BenchResult, Verdict, compare, parse_result_file, results_csv, results_json, script_benches,
};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage:
  luau-bench run --scripts <dir> [--vm luaur] [--filter <substring>] [--runs N]
                 [--json <file>] [--csv <file>] [--meta key=value]... [--max-load <1-min load>]
  luau-bench compare --baseline <file> --results <file> [--ceiling 1.10] [--slack-ms 0.25]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("run") => run(&args[1..]),
        Some("compare") => compare_command(&args[1..]),
        _ => Err(USAGE.to_string()),
    };
    match outcome {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(message) => {
            eprintln!("luau-bench: {message}");
            ExitCode::from(2)
        }
    }
}

struct Flags {
    values: BTreeMap<String, Vec<String>>,
}

impl Flags {
    fn parse(args: &[String], known: &[&str]) -> Result<Self, String> {
        let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut iter = args.iter();
        while let Some(flag) = iter.next() {
            let name = flag
                .strip_prefix("--")
                .filter(|name| known.contains(name))
                .ok_or_else(|| format!("unknown argument `{flag}`\n{USAGE}"))?;
            let value = iter
                .next()
                .ok_or_else(|| format!("`{flag}` needs a value"))?;
            values
                .entry(name.to_string())
                .or_default()
                .push(value.clone());
        }
        Ok(Self { values })
    }

    fn one(&self, name: &str) -> Option<&str> {
        self.values
            .get(name)
            .and_then(|values| values.last())
            .map(String::as_str)
    }

    fn required(&self, name: &str) -> Result<&str, String> {
        self.one(name)
            .ok_or_else(|| format!("--{name} is required\n{USAGE}"))
    }

    fn number(&self, name: &str, default: f64) -> Result<f64, String> {
        self.one(name).map_or(Ok(default), |text| {
            text.parse()
                .map_err(|_| format!("--{name}: not a number: {text}"))
        })
    }
}

fn run(args: &[String]) -> Result<bool, String> {
    let flags = Flags::parse(
        args,
        &[
            "scripts", "vm", "filter", "runs", "json", "csv", "meta", "max-load",
        ],
    )?;
    let scripts_dir = PathBuf::from(flags.required("scripts")?);
    let vm_name = flags.one("vm").unwrap_or("luaur");
    let vm = vm::by_name(vm_name).ok_or_else(|| {
        format!(
            "unknown --vm `{vm_name}` (known: {})",
            vm::VM_NAMES.join(", ")
        )
    })?;
    let runs = flags.number("runs", 1.0)? as usize;
    if runs == 0 {
        return Err("--runs must be at least 1".into());
    }
    let max_load = flags
        .one("max-load")
        .map(|_| flags.number("max-load", 0.0))
        .transpose()?;
    let filter = flags.one("filter");

    let mut scripts = Vec::new();
    for entry in std::fs::read_dir(&scripts_dir)
        .map_err(|error| format!("{}: {error}", scripts_dir.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        let Some(stem) = path
            .extension()
            .filter(|ext| *ext == "lua")
            .and(path.file_stem())
            .and_then(|stem| stem.to_str())
        else {
            continue;
        };
        if filter.is_none_or(|filter| stem.contains(filter)) {
            scripts.push((stem.to_string(), path.clone()));
        }
    }
    scripts.sort();
    if scripts.is_empty() {
        return Err(format!(
            "no matching .lua scripts in {} (run tests/luau-bench/merge.py first)",
            scripts_dir.display()
        ));
    }
    let sources = scripts
        .iter()
        .map(|(stem, path)| {
            std::fs::read_to_string(path)
                .map(|source| (stem.clone(), source))
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut meta = Map::new();
    for pair in flags.values.get("meta").into_iter().flatten() {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| format!("--meta wants key=value, got `{pair}`"))?;
        meta.insert(key.to_string(), Value::String(value.to_string()));
    }
    meta.insert("os".into(), json!(std::env::consts::OS));
    meta.insert("arch".into(), json!(std::env::consts::ARCH));
    meta.insert(
        "cpus".into(),
        json!(
            std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(0)
        ),
    );

    let mut benches: BTreeMap<String, BenchResult> = BTreeMap::new();
    let mut passes = Vec::new();
    let mut loaded_passes = 0;
    // Whole-suite passes, so a transient disturbance lands on one pass of
    // many benches instead of every pass of one bench.
    for pass in 1..=runs {
        let load_before = load_average();
        let started = Instant::now();
        let progress = std::io::stderr().is_terminal();
        for (stem, source) in &sources {
            if progress {
                eprint!("\r[pass {pass}/{runs}] {stem:<60}");
            }
            match vm.run_script(&format!("={stem}"), source) {
                Ok(lines) => {
                    let found = script_benches(stem, &lines);
                    if found.is_empty() {
                        benches
                            .entry(stem.clone())
                            .or_default()
                            .errors
                            .push("no report line printed".into());
                    }
                    for (id, report) in found {
                        let median = luau_bench::median(&report.times_ms).unwrap_or(f64::NAN);
                        benches.entry(id).or_default().pass_medians_ms.push(median);
                    }
                }
                Err(error) => benches.entry(stem.clone()).or_default().errors.push(error),
            }
        }
        let load_after = load_average();
        let seconds = started.elapsed().as_secs_f64();
        eprintln!(
            "{}[pass {pass}/{runs}] {} scripts in {seconds:.1}s, 1-min load {} -> {}{:40}",
            if progress { "\r" } else { "" },
            sources.len(),
            fmt_load(&load_before),
            fmt_load(&load_after),
            ""
        );
        let loaded = max_load.is_some_and(|limit| {
            [&load_before, &load_after]
                .iter()
                .any(|load| load.first().is_some_and(|one| *one > limit))
        });
        if loaded {
            loaded_passes += 1;
            eprintln!(
                "luau-bench: pass {pass}: 1-minute load average above --max-load {}",
                max_load.unwrap_or_default()
            );
        }
        passes.push(json!({ "pass": pass, "seconds": (seconds * 10.0).round() / 10.0, "load_before": load_before, "load_after": load_after, "over_max_load": loaded }));
    }

    print_run_table(vm.name(), runs, &benches);
    let document = results_json(vm.name(), runs, &meta, &passes, &benches);
    if let Some(path) = flags.one("json") {
        write(
            Path::new(path),
            &(serde_json::to_string_pretty(&document).map_err(|error| error.to_string())? + "\n"),
        )?;
        eprintln!("luau-bench: wrote {path}");
    }
    if let Some(path) = flags.one("csv") {
        write(Path::new(path), &results_csv(&benches))?;
        eprintln!("luau-bench: wrote {path}");
    }
    let failed = benches
        .values()
        .filter(|result| result.median_ms().is_none())
        .count();
    if failed > 0 {
        eprintln!("luau-bench: {failed} bench(es) failed");
    }
    if loaded_passes > 0 {
        eprintln!(
            "luau-bench: {loaded_passes} pass(es) ran on a loaded host; numbers are not usable as a baseline"
        );
    }
    Ok(failed == 0 && loaded_passes == 0)
}

fn print_run_table(vm: &str, runs: usize, benches: &BTreeMap<String, BenchResult>) {
    let width = benches.keys().map(String::len).max().unwrap_or(5).max(5);
    println!(
        "{:<width$} {:>10} {:>10} {:>10}",
        "bench", "median ms", "min pass", "max pass"
    );
    for (id, result) in benches {
        match result.median_ms() {
            Some(median) => {
                let min = result
                    .pass_medians_ms
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min);
                let max = result
                    .pass_medians_ms
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max);
                println!("{id:<width$} {median:>10.2} {min:>10.2} {max:>10.2}");
            }
            None => println!(
                "{id:<width$} {:>10} {}",
                "ERROR",
                result.errors.first().map(String::as_str).unwrap_or("")
            ),
        }
    }
    let ok: Vec<f64> = benches
        .values()
        .filter_map(BenchResult::median_ms)
        .collect();
    println!(
        "{vm}: {} benches, {} ok, geomean {} ms (median of {runs} pass(es), each the median of the script's 20 kept runs)",
        benches.len(),
        ok.len(),
        luau_bench::geomean(&ok).map_or("-".into(), |value| format!("{value:.3}"))
    );
}

fn compare_command(args: &[String]) -> Result<bool, String> {
    let flags = Flags::parse(args, &["baseline", "results", "ceiling", "slack-ms"])?;
    let ceiling = flags.number("ceiling", 1.10)?;
    let slack_ms = flags.number("slack-ms", 0.25)?;
    let baseline = read_results(flags.required("baseline")?)?;
    let results = read_results(flags.required("results")?)?;
    let comparison = compare(&baseline, &results, ceiling, slack_ms);

    let width = comparison
        .rows
        .iter()
        .map(|row| row.bench.len())
        .max()
        .unwrap_or(5)
        .max(5);
    let base_label = format!("{} base", baseline.vm);
    let result_label = format!("{} now", results.vm);
    println!(
        "{:<width$} {:>12} {:>12} {:>8}  verdict",
        "bench", base_label, result_label, "ratio"
    );
    let ms = |value: Option<f64>| value.map_or("-".to_string(), |value| format!("{value:.2}"));
    for row in &comparison.rows {
        let ratio = match (row.baseline_ms, row.result_ms) {
            (Some(base), Some(now)) => format!("{:.3}x", now / base),
            _ => "-".into(),
        };
        let verdict = match row.verdict {
            Verdict::Ok => "ok",
            Verdict::OverCeiling => "OVER CEILING",
            Verdict::Missing => "MISSING/FAILED",
            Verdict::New => "new (not judged)",
        };
        println!(
            "{:<width$} {:>12} {:>12} {:>8}  {verdict}",
            row.bench,
            ms(row.baseline_ms),
            ms(row.result_ms),
            ratio
        );
    }
    let over = comparison
        .rows
        .iter()
        .filter(|row| row.verdict == Verdict::OverCeiling)
        .count();
    let missing = comparison
        .rows
        .iter()
        .filter(|row| row.verdict == Verdict::Missing)
        .count();
    println!(
        "geomean {} over {} benches; ceiling {ceiling:.2}x per bench (and {slack_ms} ms absolute slack) and for the geomean; {over} over ceiling, {missing} missing",
        comparison
            .geomean_ratio
            .map_or("-".into(), |ratio| format!("{ratio:.3}x")),
        comparison.compared,
    );
    for key in ["host", "date", "commit"] {
        if let Some(value) = baseline.meta.get(key).and_then(Value::as_str) {
            println!("baseline {key}: {value}");
        }
    }
    println!(
        "{}",
        if comparison.passed {
            "luau-bench compare: PASS"
        } else {
            "luau-bench compare: FAIL"
        }
    );
    Ok(comparison.passed)
}

fn read_results(path: &str) -> Result<luau_bench::ResultFile, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    parse_result_file(&text).map_err(|error| format!("{path}: {error}"))
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    std::fs::write(path, contents).map_err(|error| format!("{}: {error}", path.display()))
}

/// 1, 5 and 15 minute load averages, from /proc/loadavg or `sysctl vm.loadavg`.
fn load_average() -> Vec<f64> {
    let text = std::fs::read_to_string("/proc/loadavg").ok().or_else(|| {
        std::process::Command::new("sysctl")
            .args(["-n", "vm.loadavg"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
    });
    text.map(|text| {
        text.split_whitespace()
            .filter_map(|field| field.trim_matches(['{', '}']).parse().ok())
            .take(3)
            .collect()
    })
    .unwrap_or_default()
}

fn fmt_load(load: &[f64]) -> String {
    load.first().map_or("?".into(), |one| format!("{one:.2}"))
}
