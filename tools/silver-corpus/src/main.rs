use anyhow::Context;
use silver_corpus::{
    Lane, Status, compare_files, read_manifest, resolve_expected, validate_manifest,
};
use std::env;
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
    let mut probed = 0usize;
    let mut probe_exact = 0usize;
    for case in &manifest.cases {
        if options.lane.is_some_and(|lane| lane != case.lane) {
            continue;
        }
        selected += 1;
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
        "silver-corpus summary: entries={} provenanced={} runtime={} scripted={} selected={} cpp-baseline-exact={} cpp-rust-exact={} pending={} pending-scripted={} diverges={} unsupported-feature={} provenance-unknown={} operations={} bytes={}",
        summary.entries,
        summary.provenanced,
        summary.runtime,
        summary.scripted,
        selected,
        summary.cpp_baseline_exact,
        summary.cpp_rust_exact,
        summary.status(Status::Pending),
        summary.status(Status::PendingScripted),
        summary.status(Status::Diverges),
        summary.status(Status::UnsupportedFeature),
        summary.status(Status::ProvenanceUnknown),
        summary.operations,
        summary.bytes,
    );
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
}

impl Options {
    fn parse(args: Vec<String>) -> anyhow::Result<Self> {
        let mut manifest = PathBuf::from("silver-corpus.toml");
        let mut rive_runtime_dir = env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut lane = None;
        let mut rust_output_dir = None;
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
        })
    }
}

fn print_help() {
    println!(
        "usage:\n  silver-corpus validate [--manifest silver-corpus.toml] [--rive-runtime-dir PATH] [--lane runtime|scripted|unknown] [--rust-output-dir PATH]\n  silver-corpus compare <expected.sriv> <actual.sriv>"
    );
}
