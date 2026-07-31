use anyhow::Context;
use silver_corpus::{
    Execution, Lane, Status, compare_files, compare_sriv, parse_sriv, read_manifest,
    resolve_expected, validate_manifest,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("silver-corpus error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("validate") | None => validate(Options::parse(args.collect())?),
        Some("compare") => {
            let expected = args.next().context("compare requires <expected.sriv>")?;
            let actual = args.next().context("compare requires <actual.sriv>")?;
            anyhow::ensure!(args.next().is_none(), "compare accepts exactly two paths");
            compare_files(Path::new(&expected), Path::new(&actual))?;
            println!("silver-corpus compare: exact");
            Ok(())
        }
        Some("-h" | "--help" | "help") => {
            print_help();
            Ok(())
        }
        Some(other) => anyhow::bail!("unknown command {other}; expected validate or compare"),
    }
}

fn validate(options: Options) -> anyhow::Result<()> {
    let manifest = read_manifest(&options.manifest)?;
    let summary = validate_manifest(&manifest, &options.rive_runtime_dir)?;

    let mut selected = 0usize;
    let mut executed = 0usize;
    let mut byte_exact = 0usize;
    let mut epsilon_exact = 0usize;
    let mut divergent = 0usize;
    let mut unsupported = 0usize;
    let mut probed = 0usize;
    let mut probe_exact = 0usize;
    for case in &manifest.cases {
        if options.lane.is_some_and(|lane| lane != case.lane) {
            continue;
        }
        if options.id.as_deref().is_some_and(|id| id != case.id) {
            continue;
        }
        selected += 1;
        if case.lane == Lane::Runtime {
            if case.status == Status::UnsupportedFeature {
                unsupported += 1;
                println!("[unsupported-feature] {}: {}", case.id, case.note);
                continue;
            }
            let execution = match Execution::run(case, &options.rive_runtime_dir) {
                Ok(execution) => execution,
                Err(error) => {
                    return Err(error).context(format!("{} action execution failed", case.id));
                }
            };
            executed += 1;
            let expected_path = resolve_expected(&options.rive_runtime_dir, case);
            let expected_bytes = fs::read(&expected_path)
                .with_context(|| format!("failed to read {}", expected_path.display()))?;
            let expected = parse_sriv(&expected_bytes)
                .with_context(|| format!("invalid expected {}", expected_path.display()))?;
            let actual = parse_sriv(execution.bytes())
                .with_context(|| format!("{} produced invalid SRIV", case.id))?;
            match compare_sriv(&expected, &actual) {
                Ok(()) => {
                    if case.status == Status::Diverges {
                        anyhow::bail!(
                            "{} is classified diverges but now compares exact; promote it",
                            case.id
                        );
                    }
                    if expected_bytes == execution.bytes() {
                        byte_exact += 1;
                        println!("[exact] {}: byte exact", case.id);
                    } else {
                        epsilon_exact += 1;
                        println!("[epsilon] {}: operation exact within epsilon", case.id);
                    }
                }
                Err(difference) if case.status == Status::Diverges => {
                    let recorded = case
                        .note
                        .split_once("first difference: ")
                        .and_then(|(_, difference)| difference.strip_suffix('.'))
                        .context("divergent manifest note has no recorded first difference")?;
                    anyhow::ensure!(
                        recorded == difference.to_string(),
                        "{} divergence changed: recorded {recorded}; actual {difference}",
                        case.id
                    );
                    divergent += 1;
                    println!("[divergent] {}: {}", case.id, difference);
                }
                Err(difference) => {
                    anyhow::bail!("{} exact entry diverged: {difference}", case.id);
                }
            }
        }
        if let Some(output_dir) = options.rust_output_dir.as_deref() {
            let actual = output_dir.join(format!("{}.sriv", case.id));
            if !actual.is_file() {
                if case.status == Status::Exact {
                    anyhow::bail!(
                        "{} exact Rust output is missing: {}",
                        case.id,
                        actual.display()
                    );
                }
                continue;
            }
            probed += 1;
            match compare_files(&resolve_expected(&options.rive_runtime_dir, case), &actual) {
                Ok(()) if case.status == Status::Diverges => {
                    anyhow::bail!(
                        "{} is classified diverges but now compares exact; promote it explicitly",
                        case.id
                    );
                }
                Ok(()) => {
                    probe_exact += 1;
                    println!("[{}] {}: cpp-rust exact", case.status, case.id);
                }
                Err(error) if case.status == Status::Diverges => {
                    println!("[diverges] {}: classified difference: {error:#}", case.id);
                }
                Err(error)
                    if matches!(
                        case.status,
                        Status::UnsupportedFeature | Status::ProvenanceUnknown
                    ) =>
                {
                    println!(
                        "[{}] {}: non-gating difference: {error:#}",
                        case.status, case.id
                    );
                }
                Err(error) if matches!(case.status, Status::Pending | Status::PendingScripted) => {
                    anyhow::bail!(
                        "{} produced an unclassified Rust-vs-C++ difference: {error:#}; \
                         classify it as diverges with a specific note",
                        case.id
                    );
                }
                Err(error) => {
                    return Err(error).context(format!("{} exact entry diverged", case.id));
                }
            }
        }
    }

    println!(
        "silver-corpus summary: entries={} provenanced={} runtime={} scripted={} selected={} executed={} cpp-baseline-exact={} cpp-rust-exact={} byte-exact={} epsilon={} divergent={} unsupported={} pending={} pending-scripted={} diverges={} unsupported-feature={} provenance-unknown={} operations={} bytes={}",
        summary.entries,
        summary.provenanced,
        summary.runtime,
        summary.scripted,
        selected,
        executed,
        summary.cpp_baseline_exact,
        byte_exact + epsilon_exact,
        byte_exact,
        epsilon_exact,
        divergent,
        unsupported,
        summary.status(Status::Pending),
        summary.status(Status::PendingScripted),
        summary.status(Status::Diverges),
        summary.status(Status::UnsupportedFeature),
        summary.status(Status::ProvenanceUnknown),
        summary.operations,
        summary.bytes,
    );
    println!(
        "silver-corpus lane-summary: lane={} selected={} byte-exact={} epsilon={} divergent={} unsupported={}",
        options
            .lane
            .map(|lane| lane.to_string())
            .unwrap_or_else(|| "all".to_owned()),
        selected,
        byte_exact,
        epsilon_exact,
        divergent,
        unsupported,
    );
    anyhow::ensure!(
        selected > 0,
        "no manifest entries matched the requested lane/id filters"
    );
    if options.id.is_none() && options.lane.is_none_or(|lane| lane == Lane::Runtime) {
        anyhow::ensure!(
            byte_exact + epsilon_exact >= manifest.corpus.min_cpp_rust_exact,
            "executed cpp-rust-exact={} is below ratchet {}",
            byte_exact + epsilon_exact,
            manifest.corpus.min_cpp_rust_exact
        );
    }
    if options.rust_output_dir.is_some() {
        println!("silver-corpus probe: rust-exact={probe_exact} probed={probed}");
    }
    Ok(())
}

#[derive(Debug)]
struct Options {
    manifest: PathBuf,
    rive_runtime_dir: PathBuf,
    lane: Option<Lane>,
    rust_output_dir: Option<PathBuf>,
    id: Option<String>,
}

impl Options {
    fn parse(args: Vec<String>) -> anyhow::Result<Self> {
        let mut manifest = PathBuf::from("silver-corpus.toml");
        let mut rive_runtime_dir = env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut lane = None;
        let mut rust_output_dir = None;
        let mut id = None;
        let mut index = 0;
        while index < args.len() {
            let option = &args[index];
            let value = |index: &mut usize| -> anyhow::Result<String> {
                *index += 1;
                args.get(*index)
                    .cloned()
                    .with_context(|| format!("{option} requires a value"))
            };
            match option.as_str() {
                "--manifest" => manifest = PathBuf::from(value(&mut index)?),
                "--rive-runtime-dir" => rive_runtime_dir = PathBuf::from(value(&mut index)?),
                "--rust-output-dir" => rust_output_dir = Some(PathBuf::from(value(&mut index)?)),
                "--id" => id = Some(value(&mut index)?),
                "--lane" => {
                    lane = Some(match value(&mut index)?.as_str() {
                        "runtime" => Lane::Runtime,
                        "scripted" => Lane::Scripted,
                        "unknown" => Lane::Unknown,
                        value => anyhow::bail!("invalid lane {value}"),
                    });
                }
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                other => anyhow::bail!("unknown validate option {other}"),
            }
            index += 1;
        }
        Ok(Self {
            manifest,
            rive_runtime_dir,
            lane,
            rust_output_dir,
            id,
        })
    }
}

fn print_help() {
    println!(
        "usage:\n  silver-corpus validate [--manifest silver-corpus.toml] [--rive-runtime-dir PATH] [--lane runtime|scripted|unknown] [--id ID] [--rust-output-dir PATH]\n  silver-corpus compare <expected.sriv> <actual.sriv>"
    );
}
