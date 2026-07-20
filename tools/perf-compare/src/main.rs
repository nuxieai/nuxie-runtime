use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("perf-compare error: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1).collect())?;
    match &options.mode {
        Mode::Single(target) => run_single(&options, target),
        Mode::Corpus(corpus) => run_corpus(&options, corpus),
    }
}

fn run_single(options: &Options, target: &RunTarget) -> Result<(), String> {
    let (cpp, rust) = measure_runners(target, options)?;

    println!("perf-compare file={}", target.file.display());
    println!(
        "perf-compare samples={} iterations={} warmups={}",
        target.samples, options.iterations, options.warmups
    );
    print_metric(options.runner_benchmark, options.benchmark_repeat);
    print_aggregate_mode(options.aggregate);
    print_runner_order(options.runner_order);
    print_measurements("cpp", cpp.total);
    print_measurements("rust", rust.total);
    let file = FileResult {
        id: target.id.clone(),
        file: target.file.display().to_string(),
        segments: target.segment_count,
        cpp,
        rust,
    };
    let ratio = file.rust_over_cpp(options.aggregate);
    println!("rust_over_cpp={ratio:.3}");

    let aggregate = aggregate_results(std::slice::from_ref(&file), options.aggregate);
    write_json_report_if_requested(options, std::slice::from_ref(&file), &aggregate)?;
    check_max_ratio(ratio, options.max_ratio)
}

fn run_corpus(options: &Options, corpus: &Path) -> Result<(), String> {
    let entries = parse_corpus(corpus)?;
    let corpus_dir = corpus
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let targets = corpus_targets(entries, options, &corpus_dir)?;
    if targets.is_empty() {
        return Err(format!(
            "corpus {} has no selected exact entries",
            corpus.display()
        ));
    }

    let mut files = Vec::with_capacity(targets.len());
    println!("perf-compare corpus={}", corpus.display());
    println!(
        "perf-compare entries={} iterations={} warmups={}",
        targets.len(),
        options.iterations,
        options.warmups
    );
    print_metric(options.runner_benchmark, options.benchmark_repeat);
    print_aggregate_mode(options.aggregate);
    print_runner_order(options.runner_order);
    for target in &targets {
        let (cpp, rust) = measure_runners(target, options)?;
        let ratio = measurement_value(rust.total, options.aggregate).as_secs_f64()
            / measurement_value(cpp.total, options.aggregate).as_secs_f64();
        println!(
            "entry id={} segments={} cpp_median_ms={:.3} rust_median_ms={:.3} cpp_min_ms={:.3} rust_min_ms={:.3} rust_over_cpp={ratio:.3}",
            target.id,
            target.segment_count,
            millis(cpp.total.median),
            millis(rust.total.median),
            millis(cpp.total.min),
            millis(rust.total.min)
        );
        files.push(FileResult {
            id: target.id.clone(),
            file: target.file.display().to_string(),
            segments: target.segment_count,
            cpp,
            rust,
        });
    }

    let aggregate = aggregate_results(&files, options.aggregate);
    println!(
        "aggregate mode={} entries={} segments={} cpp_ms_sum={:.3} rust_ms_sum={:.3} rust_over_cpp={:.3}",
        aggregate.mode.as_str(),
        aggregate.entries,
        aggregate.segments,
        millis(aggregate.cpp_selected_sum()),
        millis(aggregate.rust_selected_sum()),
        aggregate.rust_over_cpp
    );
    write_json_report_if_requested(options, &files, &aggregate)?;
    check_max_ratio(aggregate.rust_over_cpp, options.max_ratio)
}

fn corpus_targets(
    entries: Vec<CorpusEntry>,
    options: &Options,
    corpus_dir: &Path,
) -> Result<Vec<RunTarget>, String> {
    let mut entries = entries
        .into_iter()
        .filter(|entry| entry.status == Status::Exact)
        .collect::<Vec<_>>();
    if let Some(corpus_ids) = &options.corpus_ids {
        let mut selected = Vec::with_capacity(corpus_ids.len());
        for id in corpus_ids {
            let Some(index) = entries.iter().position(|entry| &entry.id == id) else {
                return Err(format!(
                    "--corpus-ids entry {id} was not found among exact corpus entries"
                ));
            };
            selected.push(entries.remove(index));
        }
        entries = selected;
    } else if let Some(limit) = options.corpus_limit {
        entries.truncate(limit);
    }

    let mut targets = Vec::new();
    for entry in entries {
        if options.benchmark_repeat > 1 {
            targets.extend(RunTarget::from_corpus_entry_repeated_samples(
                &entry,
                &options.rive_runtime_dir,
            )?);
        } else {
            targets.push(RunTarget::from_corpus_entry(
                &entry,
                &options.rive_runtime_dir,
                corpus_dir,
            ));
        }
    }
    Ok(targets)
}

fn print_measurements(label: &str, measurements: Measurements) {
    println!(
        "{label} median_ms={:.3} min_ms={:.3} max_ms={:.3}",
        millis(measurements.median),
        millis(measurements.min),
        millis(measurements.max)
    );
}

fn metric_name(runner_benchmark: bool) -> &'static str {
    if runner_benchmark {
        "runner_hot_loop_ms"
    } else {
        "process_elapsed_ms"
    }
}

fn print_metric(runner_benchmark: bool, benchmark_repeat: usize) {
    println!("perf-compare metric={}", metric_name(runner_benchmark));
    if runner_benchmark {
        println!("perf-compare benchmark_repeat={benchmark_repeat}");
    }
}

fn print_aggregate_mode(mode: AggregateMode) {
    println!("perf-compare aggregate={}", mode.as_str());
}

fn print_runner_order(order: RunnerOrder) {
    println!("perf-compare runner_order={}", order.as_str());
}

/// Write the machine-readable report when `--json` was passed. Runs before
/// the max-ratio gate so threshold failures still leave an artifact behind.
fn write_json_report_if_requested(
    options: &Options,
    files: &[FileResult],
    aggregate: &Aggregate,
) -> Result<(), String> {
    let Some(path) = &options.json else {
        return Ok(());
    };
    let report = render_json_report(options, files, aggregate);
    std::fs::write(path, report)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    println!("perf-compare json={}", path.display());
    Ok(())
}

fn render_json_report(options: &Options, files: &[FileResult], aggregate: &Aggregate) -> String {
    let mut out = String::new();
    out.push('{');
    push_json_key(&mut out, "schema");
    push_json_string(&mut out, "rive-perf-compare-json-v1");
    out.push(',');
    push_json_key(&mut out, "metric");
    push_json_string(&mut out, metric_name(options.runner_benchmark));
    out.push(',');
    push_json_key(&mut out, "iterations");
    out.push_str(&options.iterations.to_string());
    out.push(',');
    push_json_key(&mut out, "warmups");
    out.push_str(&options.warmups.to_string());
    out.push(',');
    push_json_key(&mut out, "runner_order");
    push_json_string(&mut out, options.runner_order.as_str());
    out.push(',');
    if options.runner_benchmark {
        push_json_key(&mut out, "benchmark_repeat");
        out.push_str(&options.benchmark_repeat.to_string());
        out.push(',');
    }

    push_json_key(&mut out, "meta");
    out.push('{');
    for (index, (key, value)) in options.meta.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        push_json_string(&mut out, key);
        out.push(':');
        push_json_string(&mut out, value);
    }
    out.push_str("},");

    push_json_key(&mut out, "files");
    out.push('[');
    for (index, file) in files.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        push_file_result(&mut out, file, options.aggregate);
    }
    out.push_str("],");

    push_json_key(&mut out, "aggregate");
    out.push('{');
    push_json_key(&mut out, "entries");
    out.push_str(&aggregate.entries.to_string());
    out.push(',');
    push_json_key(&mut out, "segments");
    out.push_str(&aggregate.segments.to_string());
    out.push(',');
    push_json_key(&mut out, "mode");
    push_json_string(&mut out, aggregate.mode.as_str());
    out.push(',');
    push_json_key(&mut out, "cpp_median_ms_sum");
    push_json_number(&mut out, millis(aggregate.cpp_median_sum));
    out.push(',');
    push_json_key(&mut out, "rust_median_ms_sum");
    push_json_number(&mut out, millis(aggregate.rust_median_sum));
    out.push(',');
    push_json_key(&mut out, "cpp_min_ms_sum");
    push_json_number(&mut out, millis(aggregate.cpp_min_sum));
    out.push(',');
    push_json_key(&mut out, "rust_min_ms_sum");
    push_json_number(&mut out, millis(aggregate.rust_min_sum));
    out.push(',');
    push_json_key(&mut out, "cpp_selected_ms_sum");
    push_json_number(&mut out, millis(aggregate.cpp_selected_sum()));
    out.push(',');
    push_json_key(&mut out, "rust_selected_ms_sum");
    push_json_number(&mut out, millis(aggregate.rust_selected_sum()));
    out.push(',');
    push_json_key(&mut out, "rust_over_cpp");
    push_json_number(&mut out, aggregate.rust_over_cpp);
    out.push('}');

    out.push('}');
    out.push('\n');
    out
}

fn push_file_result(out: &mut String, file: &FileResult, aggregate: AggregateMode) {
    out.push('{');
    push_json_key(out, "id");
    push_json_string(out, &file.id);
    out.push(',');
    push_json_key(out, "file");
    push_json_string(out, &file.file);
    out.push(',');
    push_json_key(out, "segments");
    out.push_str(&file.segments.to_string());
    out.push(',');

    push_json_key(out, "runners");
    out.push('{');
    push_json_key(out, "cpp");
    push_runner_measurements(out, &file.cpp);
    out.push(',');
    push_json_key(out, "rust");
    push_runner_measurements(out, &file.rust);
    out.push_str("},");

    push_json_key(out, "rust_over_cpp");
    push_json_number(out, file.rust_over_cpp(aggregate));
    out.push(',');

    push_json_key(out, "rust_over_cpp_by_phase");
    out.push('{');
    push_json_key(out, "total");
    push_json_number(out, file.rust_over_cpp(aggregate));
    for (name, cpp_phase) in &file.cpp.phases {
        let Some((_, rust_phase)) = file
            .rust
            .phases
            .iter()
            .find(|(rust_name, _)| rust_name == name)
        else {
            continue;
        };
        out.push(',');
        push_json_key(out, name);
        push_json_number(
            out,
            measurement_value(*rust_phase, aggregate).as_secs_f64()
                / measurement_value(*cpp_phase, aggregate).as_secs_f64(),
        );
    }
    out.push('}');

    out.push('}');
}

fn push_runner_measurements(out: &mut String, measurements: &RunnerMeasurements) {
    out.push('{');
    push_json_key(out, "iterations");
    out.push_str(&measurements.iterations.to_string());
    out.push(',');
    push_json_key(out, "phases");
    out.push('{');
    push_json_key(out, "total");
    push_measurements(out, measurements.total);
    for (name, phase) in &measurements.phases {
        out.push(',');
        push_json_key(out, name);
        push_measurements(out, *phase);
    }
    out.push_str("}}");
}

fn push_measurements(out: &mut String, measurements: Measurements) {
    out.push('{');
    push_json_key(out, "median_ms");
    push_json_number(out, millis(measurements.median));
    out.push(',');
    push_json_key(out, "min_ms");
    push_json_number(out, millis(measurements.min));
    out.push(',');
    push_json_key(out, "max_ms");
    push_json_number(out, millis(measurements.max));
    out.push(',');
    push_json_key(out, "spread_ms");
    push_json_number(out, millis(measurements.spread()));
    out.push('}');
}

fn push_json_key(out: &mut String, key: &str) {
    push_json_string(out, key);
    out.push(':');
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// JSON has no NaN/Infinity literals; emit null for non-finite values.
fn push_json_number(out: &mut String, value: f64) {
    if value.is_finite() {
        out.push_str(&format!("{value:.6}"));
    } else {
        out.push_str("null");
    }
}

fn check_max_ratio(ratio: f64, max_ratio: Option<f64>) -> Result<(), String> {
    let Some(max_ratio) = max_ratio else {
        return Ok(());
    };
    if ratio <= max_ratio {
        println!("perf-threshold ok rust_over_cpp={ratio:.3} max_ratio={max_ratio:.3}");
        Ok(())
    } else {
        Err(format!(
            "perf threshold failed: rust_over_cpp={ratio:.3} max_ratio={max_ratio:.3}"
        ))
    }
}

#[derive(Debug, Clone)]
struct Options {
    cpp_runner: PathBuf,
    rust_runner: PathBuf,
    rive_runtime_dir: PathBuf,
    mode: Mode,
    iterations: usize,
    warmups: usize,
    runner_order: RunnerOrder,
    aggregate: AggregateMode,
    corpus_limit: Option<usize>,
    corpus_ids: Option<Vec<String>>,
    max_ratio: Option<f64>,
    runner_benchmark: bool,
    benchmark_repeat: usize,
    json: Option<PathBuf>,
    meta: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
enum Mode {
    Single(RunTarget),
    Corpus(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AggregateMode {
    Median,
    Min,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunnerOrder {
    CppFirst,
    RustFirst,
}

impl RunnerOrder {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "cpp-first" => Ok(Self::CppFirst),
            "rust-first" => Ok(Self::RustFirst),
            other => Err(format!("unknown runner order: {other}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::CppFirst => "cpp-first",
            Self::RustFirst => "rust-first",
        }
    }
}

impl AggregateMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "median" => Ok(Self::Median),
            "min" => Ok(Self::Min),
            other => Err(format!("unknown aggregate mode: {other}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Median => "median",
            Self::Min => "min",
        }
    }
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut cpp_runner = env::var_os("GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(default_cpp_runner);
        let mut rust_runner = env::var_os("RUST_GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/debug/rust-golden-runner"));
        let mut rive_runtime_dir = env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut file = None;
        let mut corpus = None;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut samples = "0".to_owned();
        let mut sample_count = 1usize;
        let mut iterations = 5usize;
        let mut warmups = 0usize;
        let mut runner_order = RunnerOrder::CppFirst;
        let mut aggregate = AggregateMode::Median;
        let mut corpus_limit = None;
        let mut corpus_ids = None;
        let mut max_ratio = None;
        let mut runner_benchmark = false;
        let mut benchmark_repeat = 1usize;
        let mut json = None;
        let mut meta = Vec::new();

        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            let mut value = |option: &str| -> Result<String, String> {
                index += 1;
                args.get(index)
                    .cloned()
                    .ok_or_else(|| format!("{option} requires a value"))
            };

            match arg.as_str() {
                "--cpp-runner" => cpp_runner = PathBuf::from(value(arg)?),
                "--rust-runner" => rust_runner = PathBuf::from(value(arg)?),
                "--rive-runtime-dir" => rive_runtime_dir = PathBuf::from(value(arg)?),
                "--file" => file = Some(PathBuf::from(value(arg)?)),
                "--corpus" => corpus = Some(PathBuf::from(value(arg)?)),
                "--corpus-limit" => {
                    corpus_limit = Some(parse_positive_usize(&value(arg)?, "--corpus-limit")?)
                }
                "--corpus-ids" => corpus_ids = Some(parse_csv_ids(&value(arg)?, "--corpus-ids")?),
                "--aggregate" => aggregate = AggregateMode::parse(&value(arg)?)?,
                "--runner-order" => runner_order = RunnerOrder::parse(&value(arg)?)?,
                "--max-ratio" => max_ratio = Some(parse_ratio(&value(arg)?)?),
                "--runner-benchmark" => runner_benchmark = true,
                "--benchmark-repeat" => {
                    benchmark_repeat = parse_positive_usize(&value(arg)?, "--benchmark-repeat")?
                }
                "--json" => json = Some(PathBuf::from(value(arg)?)),
                "--meta" => meta.push(parse_meta(&value(arg)?)?),
                "--artboard" => artboard = Some(value(arg)?),
                "--state-machine" => state_machine = Some(value(arg)?),
                "--input-script" => input_script = Some(PathBuf::from(value(arg)?)),
                "--samples" => {
                    let parsed = parse_samples_csv(&value(arg)?)?;
                    sample_count = parsed.count;
                    samples = parsed.csv;
                }
                "--iterations" => iterations = parse_positive_usize(&value(arg)?, "--iterations")?,
                "--warmups" => {
                    warmups = value(arg)?
                        .parse::<usize>()
                        .map_err(|_| "--warmups must be a non-negative integer".to_owned())?;
                }
                "--help" | "-h" => {
                    println!(
                        "usage: perf-compare (--file <path> | --corpus corpus.toml) [--samples 0,0.5] [--iterations N] [--warmups N] [--runner-order cpp-first|rust-first] [--aggregate median|min] [--corpus-limit N | --corpus-ids a,b] [--max-ratio N] [--runner-benchmark] [--benchmark-repeat N] [--json path] [--meta key=value ...] [--cpp-runner path] [--rust-runner path]"
                    );
                    std::process::exit(0);
                }
                other if !other.starts_with('-') && file.is_none() && corpus.is_none() => {
                    file = Some(PathBuf::from(other));
                }
                other => return Err(format!("unknown option: {other}")),
            }
            index += 1;
        }

        let mode = match (file, corpus) {
            (Some(file), None) => Mode::Single(RunTarget {
                id: "single".to_owned(),
                file,
                artboard,
                state_machine,
                input_script,
                samples,
                segment_count: sample_count,
            }),
            (None, Some(corpus)) => Mode::Corpus(corpus),
            (Some(_), Some(_)) => {
                return Err("choose either --file or --corpus, not both".to_owned());
            }
            (None, None) => return Err("missing --file <path> or --corpus <path>".to_owned()),
        };
        if benchmark_repeat > 1 && !runner_benchmark {
            return Err("--benchmark-repeat requires --runner-benchmark".to_owned());
        }
        if benchmark_repeat > 1
            && let Mode::Single(target) = &mode
        {
            validate_benchmark_repeat_target(target)?;
        }
        if corpus_limit.is_some() && corpus_ids.is_some() {
            return Err("choose either --corpus-limit or --corpus-ids, not both".to_owned());
        }

        Ok(Self {
            cpp_runner,
            rust_runner,
            rive_runtime_dir,
            mode,
            iterations,
            warmups,
            runner_order,
            aggregate,
            corpus_limit,
            corpus_ids,
            max_ratio,
            runner_benchmark,
            benchmark_repeat,
            json,
            meta,
        })
    }
}

/// Metadata is passed in (never computed here) so JSON output stays
/// deterministic for a given command line.
fn parse_meta(value: &str) -> Result<(String, String), String> {
    let Some((key, meta_value)) = value.split_once('=') else {
        return Err(format!("--meta expects key=value, got {value}"));
    };
    if key.is_empty() {
        return Err(format!("--meta expects a non-empty key, got {value}"));
    }
    Ok((key.to_owned(), meta_value.to_owned()))
}

fn parse_csv_ids(value: &str, option: &str) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    for part in value.split(',') {
        let id = part.trim();
        if id.is_empty() {
            return Err(format!("{option} must not contain empty ids"));
        }
        if ids.iter().any(|existing| existing == id) {
            return Err(format!("{option} contains duplicate id {id}"));
        }
        ids.push(id.to_owned());
    }
    if ids.is_empty() {
        return Err(format!("{option} must include at least one id"));
    }
    Ok(ids)
}

#[derive(Debug, Clone)]
struct RunTarget {
    id: String,
    file: PathBuf,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<PathBuf>,
    samples: String,
    segment_count: usize,
}

impl RunTarget {
    fn from_corpus_entry(entry: &CorpusEntry, rive_runtime_dir: &Path, corpus_dir: &Path) -> Self {
        Self {
            id: entry.id.clone(),
            file: resolve_asset_path(&entry.path, rive_runtime_dir),
            artboard: entry.artboard.clone(),
            state_machine: entry.state_machine.clone(),
            input_script: entry
                .input_script
                .as_deref()
                .map(|path| resolve_script_path(path, corpus_dir)),
            samples: samples_csv(&entry.samples),
            segment_count: entry.samples.len(),
        }
    }

    fn from_corpus_entry_repeated_samples(
        entry: &CorpusEntry,
        rive_runtime_dir: &Path,
    ) -> Result<Vec<Self>, String> {
        if entry.input_script.is_some() {
            return Err(format!(
                "--benchmark-repeat cannot be combined with input_script entry {}",
                entry.id
            ));
        }

        Ok(entry
            .samples
            .iter()
            .map(|sample| Self {
                id: format!("{}@{}", entry.id, sample),
                file: resolve_asset_path(&entry.path, rive_runtime_dir),
                artboard: entry.artboard.clone(),
                state_machine: entry.state_machine.clone(),
                input_script: None,
                samples: sample.to_string(),
                segment_count: 1,
            })
            .collect())
    }
}

#[derive(Debug, Clone, Copy)]
struct Measurements {
    min: Duration,
    median: Duration,
    max: Duration,
}

impl Measurements {
    fn spread(&self) -> Duration {
        self.max.saturating_sub(self.min)
    }
}

/// One runner invocation: the measured total plus any per-phase breakdown
/// (only available in `--runner-benchmark` mode).
#[derive(Debug, Clone)]
struct RunSample {
    total: Duration,
    phases: Vec<(&'static str, Duration)>,
}

/// Summary of every iteration for one runner on one target.
#[derive(Debug, Clone)]
struct RunnerMeasurements {
    total: Measurements,
    phases: Vec<(&'static str, Measurements)>,
    iterations: usize,
}

/// Per-file comparison result feeding the console report and `--json` output.
#[derive(Debug, Clone)]
struct FileResult {
    id: String,
    file: String,
    segments: usize,
    cpp: RunnerMeasurements,
    rust: RunnerMeasurements,
}

impl FileResult {
    fn rust_over_cpp(&self, mode: AggregateMode) -> f64 {
        measurement_value(self.rust.total, mode).as_secs_f64()
            / measurement_value(self.cpp.total, mode).as_secs_f64()
    }
}

#[derive(Debug, Clone)]
struct Aggregate {
    entries: usize,
    segments: usize,
    mode: AggregateMode,
    cpp_median_sum: Duration,
    rust_median_sum: Duration,
    cpp_min_sum: Duration,
    rust_min_sum: Duration,
    rust_over_cpp: f64,
}

impl Aggregate {
    fn cpp_selected_sum(&self) -> Duration {
        match self.mode {
            AggregateMode::Median => self.cpp_median_sum,
            AggregateMode::Min => self.cpp_min_sum,
        }
    }

    fn rust_selected_sum(&self) -> Duration {
        match self.mode {
            AggregateMode::Median => self.rust_median_sum,
            AggregateMode::Min => self.rust_min_sum,
        }
    }
}

fn measurement_value(measurements: Measurements, mode: AggregateMode) -> Duration {
    match mode {
        AggregateMode::Median => measurements.median,
        AggregateMode::Min => measurements.min,
    }
}

fn aggregate_results(files: &[FileResult], mode: AggregateMode) -> Aggregate {
    let cpp_median_sum = files.iter().map(|file| file.cpp.total.median).sum();
    let rust_median_sum: Duration = files.iter().map(|file| file.rust.total.median).sum();
    let cpp_min_sum = files.iter().map(|file| file.cpp.total.min).sum();
    let rust_min_sum: Duration = files.iter().map(|file| file.rust.total.min).sum();
    let cpp_selected_sum = match mode {
        AggregateMode::Median => cpp_median_sum,
        AggregateMode::Min => cpp_min_sum,
    };
    let rust_selected_sum = match mode {
        AggregateMode::Median => rust_median_sum,
        AggregateMode::Min => rust_min_sum,
    };
    Aggregate {
        entries: files.len(),
        segments: files.iter().map(|file| file.segments).sum(),
        mode,
        cpp_median_sum,
        rust_median_sum,
        cpp_min_sum,
        rust_min_sum,
        rust_over_cpp: rust_selected_sum.as_secs_f64() / cpp_selected_sum.as_secs_f64(),
    }
}

fn measure_runner(
    label: &str,
    runner: &Path,
    target: &RunTarget,
    options: &Options,
) -> Result<RunnerMeasurements, String> {
    if options.benchmark_repeat > 1 {
        validate_benchmark_repeat_target(target)?;
    }
    for warmup in 0..options.warmups {
        run_once(label, runner, target, options, warmup + 1, true)?;
    }

    let mut samples = Vec::with_capacity(options.iterations);
    for iteration in 0..options.iterations {
        samples.push(run_once(
            label,
            runner,
            target,
            options,
            iteration + 1,
            false,
        )?);
    }
    Ok(summarize_samples(&samples))
}

fn measure_runners(
    target: &RunTarget,
    options: &Options,
) -> Result<(RunnerMeasurements, RunnerMeasurements), String> {
    match options.runner_order {
        RunnerOrder::CppFirst => {
            let cpp = measure_runner("cpp", &options.cpp_runner, target, options)?;
            let rust = measure_runner("rust", &options.rust_runner, target, options)?;
            Ok((cpp, rust))
        }
        RunnerOrder::RustFirst => {
            let rust = measure_runner("rust", &options.rust_runner, target, options)?;
            let cpp = measure_runner("cpp", &options.cpp_runner, target, options)?;
            Ok((cpp, rust))
        }
    }
}

fn summarize_samples(samples: &[RunSample]) -> RunnerMeasurements {
    let total = measurements_summary(samples.iter().map(|sample| sample.total).collect());
    let phase_names = samples
        .first()
        .map(|sample| {
            sample
                .phases
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let phases = phase_names
        .into_iter()
        .map(|name| {
            let durations = samples
                .iter()
                .map(|sample| {
                    sample
                        .phases
                        .iter()
                        .find(|(phase, _)| *phase == name)
                        .map_or(Duration::ZERO, |(_, duration)| *duration)
                })
                .collect();
            (name, measurements_summary(durations))
        })
        .collect();
    RunnerMeasurements {
        total,
        phases,
        iterations: samples.len(),
    }
}

fn run_once(
    label: &str,
    runner: &Path,
    target: &RunTarget,
    options: &Options,
    iteration: usize,
    warmup: bool,
) -> Result<RunSample, String> {
    let mut command = runner_command(
        runner,
        target,
        options.runner_benchmark,
        options.benchmark_repeat,
    );
    let start = Instant::now();
    let output = command
        .output()
        .map_err(|error| format!("failed to run {label} runner {}: {error}", runner.display()))?;
    let elapsed = start.elapsed();
    if !output.status.success() {
        return Err(format!(
            "{label} runner {} exited with {} for {}\n{}",
            runner.display(),
            output.status,
            target.id,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if options.runner_benchmark {
        let benchmark = parse_benchmark_output(&output.stdout).map_err(|error| {
            let kind = if warmup { "warmup" } else { "iteration" };
            format!(
                "{label} runner {} did not emit a benchmark for {} on {kind} {iteration}: {error}",
                runner.display(),
                target.id
            )
        })?;
        let total = benchmark
            .total
            .unwrap_or_else(|| benchmark.phases.iter().map(|(_, duration)| *duration).sum());
        let phases = benchmark.phases;
        return Ok(RunSample { total, phases });
    }
    if !output.stdout.starts_with(b"rive-golden-stream-v1\n") {
        let kind = if warmup { "warmup" } else { "iteration" };
        return Err(format!(
            "{label} runner {} did not emit a rive-golden stream for {} on {kind} {iteration}",
            runner.display(),
            target.id
        ));
    }
    Ok(RunSample {
        total: elapsed,
        phases: Vec::new(),
    })
}

fn runner_command(
    runner: &Path,
    target: &RunTarget,
    benchmark: bool,
    benchmark_repeat: usize,
) -> Command {
    let mut command = Command::new(runner);
    command.arg("--file").arg(&target.file);
    if let Some(artboard) = &target.artboard {
        command.arg("--artboard").arg(artboard);
    }
    if let Some(state_machine) = &target.state_machine {
        command.arg("--state-machine").arg(state_machine);
    }
    if let Some(input_script) = &target.input_script {
        command.arg("--input-script").arg(input_script);
    }
    command.arg("--samples").arg(&target.samples);
    if benchmark {
        command.arg("--benchmark");
        if benchmark_repeat > 1 {
            command
                .arg("--benchmark-repeat")
                .arg(benchmark_repeat.to_string());
        }
    }
    command
}

fn validate_benchmark_repeat_target(target: &RunTarget) -> Result<(), String> {
    if target.input_script.is_some() {
        return Err("--benchmark-repeat cannot be combined with --input-script".to_owned());
    }
    if target.segment_count != 1 {
        return Err("--benchmark-repeat requires exactly one sample".to_owned());
    }
    Ok(())
}

/// Hot-loop phases reported by the golden runners' `--benchmark` mode, in
/// stable output order.
const BENCHMARK_PHASES: [(&str, &str); 4] = [
    ("advance", "advance_ms"),
    ("input", "input_ms"),
    ("prepare", "prepare_ms"),
    ("draw", "draw_ms"),
];

#[derive(Debug)]
struct BenchmarkOutput {
    total: Option<Duration>,
    phases: Vec<(&'static str, Duration)>,
}

fn parse_benchmark_output(stdout: &[u8]) -> Result<BenchmarkOutput, String> {
    let text = std::str::from_utf8(stdout).map_err(|error| format!("invalid utf8: {error}"))?;
    let mut lines = text.lines();
    if lines.next() != Some("rive-golden-benchmark-v1") {
        return Err("missing rive-golden-benchmark-v1 header".to_owned());
    }

    let mut total = None;
    let mut durations: [Option<Duration>; BENCHMARK_PHASES.len()] = [None; BENCHMARK_PHASES.len()];
    for line in lines {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key == "total_ms" {
            total = Some(parse_millis(value, key)?);
            continue;
        }
        let Some(index) = BENCHMARK_PHASES
            .iter()
            .position(|(_, phase_key)| *phase_key == key)
        else {
            continue;
        };
        durations[index] = Some(parse_millis(value, key)?);
    }

    let phases = BENCHMARK_PHASES
        .iter()
        .zip(durations)
        .map(|((name, key), duration)| {
            duration
                .map(|duration| (*name, duration))
                .ok_or_else(|| format!("missing {key}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(BenchmarkOutput { total, phases })
}

fn parse_millis(value: &str, key: &str) -> Result<Duration, String> {
    let millis = value
        .parse::<f64>()
        .map_err(|error| format!("invalid {key} {value}: {error}"))?;
    if !millis.is_finite() || millis < 0.0 {
        return Err(format!("invalid {key} {value}"));
    }
    Ok(Duration::from_secs_f64(millis / 1000.0))
}

fn measurements_summary(mut measurements: Vec<Duration>) -> Measurements {
    measurements.sort();
    let min = measurements[0];
    let max = measurements[measurements.len() - 1];
    let median = measurements[measurements.len() / 2];
    Measurements { min, median, max }
}

#[derive(Debug, Clone)]
struct ParsedSamples {
    csv: String,
    count: usize,
}

fn parse_samples_csv(value: &str) -> Result<ParsedSamples, String> {
    let samples = value
        .split(',')
        .map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return Err("empty sample in --samples".to_owned());
            }
            trimmed
                .parse::<f32>()
                .map_err(|_| format!("invalid sample {trimmed}"))?;
            Ok(trimmed.to_owned())
        })
        .collect::<Result<Vec<_>, String>>()?;
    if samples.is_empty() {
        return Err("--samples must include at least one sample".to_owned());
    }
    Ok(ParsedSamples {
        count: samples.len(),
        csv: samples.join(","),
    })
}

#[derive(Debug, Clone)]
struct CorpusEntry {
    id: String,
    path: String,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<String>,
    samples: Vec<f32>,
    status: Status,
}

impl CorpusEntry {
    fn new() -> Self {
        Self {
            id: String::new(),
            path: String::new(),
            artboard: None,
            state_machine: None,
            input_script: None,
            samples: vec![0.0],
            status: Status::NotYet,
        }
    }

    fn validate(&self, line: usize) -> Result<(), String> {
        if self.id.is_empty() {
            return Err(format!("entry before line {line} is missing id"));
        }
        if self.path.is_empty() {
            return Err(format!("entry {} is missing path", self.id));
        }
        if self.samples.is_empty() {
            return Err(format!(
                "entry {} must include at least one sample",
                self.id
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Exact,
    UnsupportedFeature,
    Diverges,
    NotYet,
}

impl Status {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "exact" => Ok(Self::Exact),
            "unsupported-feature" => Ok(Self::UnsupportedFeature),
            "diverges" => Ok(Self::Diverges),
            "not-yet" => Ok(Self::NotYet),
            other => Err(format!("unknown corpus status: {other}")),
        }
    }
}

fn parse_corpus(path: &Path) -> Result<Vec<CorpusEntry>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut entries = Vec::new();
    let mut current = None::<CorpusEntry>;

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[file]]" {
            if let Some(entry) = current.take() {
                entry.validate(line_number)?;
                entries.push(entry);
            }
            current = Some(CorpusEntry::new());
            continue;
        }

        let Some(entry) = current.as_mut() else {
            return Err(format!("line {line_number}: expected [[file]] before keys"));
        };
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_number}: expected key = value"))?;
        match key.trim() {
            "id" => entry.id = parse_string(value.trim(), line_number)?,
            "path" => entry.path = parse_string(value.trim(), line_number)?,
            "artboard" => entry.artboard = Some(parse_string(value.trim(), line_number)?),
            "state_machine" => entry.state_machine = Some(parse_string(value.trim(), line_number)?),
            "input_script" => entry.input_script = Some(parse_string(value.trim(), line_number)?),
            "samples" => entry.samples = parse_float_array(value.trim(), line_number)?,
            "status" => entry.status = Status::parse(&parse_string(value.trim(), line_number)?)?,
            _ => {}
        }
    }

    if let Some(entry) = current.take() {
        entry.validate(text.lines().count() + 1)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_string(value: &str, line: usize) -> Result<String, String> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'"' || bytes[bytes.len() - 1] != b'"' {
        return Err(format!("line {line}: expected quoted string"));
    }
    Ok(value[1..value.len() - 1].replace("\\\"", "\""))
}

fn parse_float_array(value: &str, line: usize) -> Result<Vec<f32>, String> {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(format!("line {line}: expected float array"));
    };
    let mut values = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        values.push(
            part.parse::<f32>()
                .map_err(|error| format!("line {line}: invalid float {part}: {error}"))?,
        );
    }
    Ok(values)
}

fn resolve_asset_path(path: &str, rive_runtime_dir: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        rive_runtime_dir.join(path)
    }
}

fn resolve_script_path(path: &str, corpus_dir: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        corpus_dir.join(path)
    }
}

fn samples_csv(samples: &[f32]) -> String {
    samples
        .iter()
        .map(|sample| sample.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_positive_usize(value: &str, option: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("{option} must be a positive integer"))?;
    if parsed == 0 {
        return Err(format!("{option} must be greater than 0"));
    }
    Ok(parsed)
}

fn parse_ratio(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| "--max-ratio must be a finite positive number".to_owned())?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err("--max-ratio must be a finite positive number".to_owned());
    }
    Ok(parsed)
}

fn default_cpp_runner() -> PathBuf {
    let os = match env::consts::OS {
        "macos" => "macosx",
        "windows" => "windows",
        _ => "linux",
    };
    PathBuf::from(format!(
        "tools/golden-runner/build/{os}/bin/debug/rive_golden_runner"
    ))
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_required_file_and_optional_values() {
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--samples".to_owned(),
            "0, 0.5".to_owned(),
            "--iterations".to_owned(),
            "3".to_owned(),
            "--warmups".to_owned(),
            "2".to_owned(),
            "--artboard".to_owned(),
            "Main".to_owned(),
        ])
        .expect("parse options");

        let Mode::Single(target) = options.mode else {
            panic!("expected single target");
        };
        assert_eq!(target.file, PathBuf::from("fixture.riv"));
        assert_eq!(target.samples, "0,0.5");
        assert_eq!(target.segment_count, 2);
        assert_eq!(options.iterations, 3);
        assert_eq!(options.warmups, 2);
        assert_eq!(target.artboard.as_deref(), Some("Main"));
        assert!(!options.runner_benchmark);
    }

    #[test]
    fn parses_corpus_mode_and_threshold() {
        let options = Options::parse(vec![
            "--corpus".to_owned(),
            "corpus.toml".to_owned(),
            "--corpus-limit".to_owned(),
            "7".to_owned(),
            "--max-ratio".to_owned(),
            "1.5".to_owned(),
            "--runner-benchmark".to_owned(),
            "--benchmark-repeat".to_owned(),
            "11".to_owned(),
            "--runner-order".to_owned(),
            "rust-first".to_owned(),
        ])
        .expect("parse options");

        assert!(matches!(options.mode, Mode::Corpus(_)));
        assert_eq!(options.corpus_limit, Some(7));
        assert_eq!(options.max_ratio, Some(1.5));
        assert!(options.runner_benchmark);
        assert_eq!(options.benchmark_repeat, 11);
        assert_eq!(options.runner_order, RunnerOrder::RustFirst);
    }

    #[test]
    fn parses_aggregate_mode_and_corpus_ids() {
        let options = Options::parse(vec![
            "--corpus".to_owned(),
            "corpus.toml".to_owned(),
            "--aggregate".to_owned(),
            "min".to_owned(),
            "--corpus-ids".to_owned(),
            "ai_assitant, spotify_kids_demo".to_owned(),
        ])
        .expect("parse options");

        assert_eq!(options.aggregate, AggregateMode::Min);
        assert_eq!(
            options.corpus_ids,
            Some(vec![
                "ai_assitant".to_owned(),
                "spotify_kids_demo".to_owned(),
            ])
        );
    }

    #[test]
    fn rejects_ambiguous_corpus_selection() {
        let error = Options::parse(vec![
            "--corpus".to_owned(),
            "corpus.toml".to_owned(),
            "--corpus-limit".to_owned(),
            "5".to_owned(),
            "--corpus-ids".to_owned(),
            "ai_assitant".to_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("choose either --corpus-limit or --corpus-ids"));
    }

    #[test]
    fn rejects_unknown_aggregate_mode() {
        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--aggregate".to_owned(),
            "fastest-ish".to_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("unknown aggregate mode"));
    }

    #[test]
    fn rejects_unknown_runner_order() {
        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--runner-order".to_owned(),
            "fastest-first".to_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("unknown runner order"));
    }

    #[test]
    fn rejects_zero_iterations() {
        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--iterations".to_owned(),
            "0".to_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("greater than 0"));
    }

    #[test]
    fn rejects_benchmark_repeat_without_runner_benchmark() {
        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--benchmark-repeat".to_owned(),
            "2".to_owned(),
        ])
        .unwrap_err();

        assert!(error.contains("requires --runner-benchmark"));
    }

    #[test]
    fn benchmark_repeat_is_passed_to_runner() {
        let target = RunTarget {
            id: "single".to_owned(),
            file: PathBuf::from("fixture.riv"),
            artboard: None,
            state_machine: None,
            input_script: None,
            samples: "0".to_owned(),
            segment_count: 1,
        };
        let command = runner_command(Path::new("runner"), &target, true, 17);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|args| args[0] == "--benchmark-repeat" && args[1] == "17")
        );
    }

    #[test]
    fn benchmark_repeat_requires_single_sample_without_inputs() {
        let mut target = RunTarget {
            id: "single".to_owned(),
            file: PathBuf::from("fixture.riv"),
            artboard: None,
            state_machine: None,
            input_script: None,
            samples: "0,0.5".to_owned(),
            segment_count: 2,
        };
        let error = validate_benchmark_repeat_target(&target).unwrap_err();
        assert!(error.contains("exactly one sample"));

        target.samples = "0".to_owned();
        target.segment_count = 1;
        target.input_script = Some(PathBuf::from("input.json"));
        let error = validate_benchmark_repeat_target(&target).unwrap_err();
        assert!(error.contains("cannot be combined"));
    }

    #[test]
    fn parses_exact_corpus_entries() {
        let path = env::temp_dir().join(format!("perf-compare-corpus-{}.toml", std::process::id()));
        fs::write(
            &path,
            r#"
[[file]]
id = "first"
path = "tests/unit_tests/assets/first.riv"
samples = [0.0, 0.25]
status = "exact"

[[file]]
id = "second"
path = "tests/unit_tests/assets/second.riv"
input_script = "tests/input_scripts/click.txt"
status = "unsupported-feature"
"#,
        )
        .expect("write corpus");

        let entries = parse_corpus(&path).expect("parse corpus");
        fs::remove_file(path).ok();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "first");
        assert_eq!(entries[0].samples, vec![0.0, 0.25]);
        assert_eq!(entries[0].status, Status::Exact);
        assert_eq!(
            entries[1].input_script.as_deref(),
            Some("tests/input_scripts/click.txt")
        );
    }

    #[test]
    fn corpus_benchmark_repeat_expands_samples_after_file_limit() {
        let path = env::temp_dir().join(format!(
            "perf-compare-repeat-corpus-{}.toml",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"
[[file]]
id = "first"
path = "tests/unit_tests/assets/first.riv"
samples = [0.0, 0.25]
status = "exact"

[[file]]
id = "second"
path = "tests/unit_tests/assets/second.riv"
samples = [0.5]
status = "exact"

[[file]]
id = "third"
path = "tests/unit_tests/assets/third.riv"
samples = [0.75]
status = "exact"
"#,
        )
        .expect("write corpus");

        let options = Options::parse(vec![
            "--corpus".to_owned(),
            path.display().to_string(),
            "--corpus-limit".to_owned(),
            "2".to_owned(),
            "--runner-benchmark".to_owned(),
            "--benchmark-repeat".to_owned(),
            "11".to_owned(),
        ])
        .expect("parse options");
        let entries = parse_corpus(&path).expect("parse corpus");
        let targets =
            corpus_targets(entries, &options, path.parent().unwrap()).expect("corpus targets");
        fs::remove_file(path).ok();

        assert_eq!(
            targets
                .iter()
                .map(|target| target.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first@0", "first@0.25", "second@0.5"]
        );
        assert_eq!(
            targets
                .iter()
                .map(|target| target.samples.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "0.25", "0.5"]
        );
        assert!(targets.iter().all(|target| target.segment_count == 1));
    }

    #[test]
    fn corpus_ids_select_exact_entries_in_requested_order() {
        let path = env::temp_dir().join(format!(
            "perf-compare-corpus-ids-{}.toml",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"
[[file]]
id = "first"
path = "tests/unit_tests/assets/first.riv"
samples = [0.0]
status = "exact"

[[file]]
id = "second"
path = "tests/unit_tests/assets/second.riv"
samples = [0.5]
status = "exact"

[[file]]
id = "parked"
path = "tests/unit_tests/assets/parked.riv"
samples = [0.75]
status = "unsupported-feature"
"#,
        )
        .expect("write corpus");

        let options = Options::parse(vec![
            "--corpus".to_owned(),
            path.display().to_string(),
            "--corpus-ids".to_owned(),
            "second,first".to_owned(),
        ])
        .expect("parse options");
        let entries = parse_corpus(&path).expect("parse corpus");
        let targets =
            corpus_targets(entries, &options, path.parent().unwrap()).expect("corpus targets");

        assert_eq!(
            targets
                .iter()
                .map(|target| target.id.as_str())
                .collect::<Vec<_>>(),
            vec!["second", "first"]
        );

        let entries = parse_corpus(&path).expect("parse corpus");
        let missing = Options::parse(vec![
            "--corpus".to_owned(),
            path.display().to_string(),
            "--corpus-ids".to_owned(),
            "parked".to_owned(),
        ])
        .expect("parse options");
        let error = corpus_targets(entries, &missing, path.parent().unwrap()).unwrap_err();
        fs::remove_file(path).ok();

        assert!(error.contains("was not found among exact corpus entries"));
    }

    #[test]
    fn corpus_benchmark_repeat_rejects_input_script_entries() {
        let path = env::temp_dir().join(format!(
            "perf-compare-repeat-input-corpus-{}.toml",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"
[[file]]
id = "scripted"
path = "tests/unit_tests/assets/scripted.riv"
input_script = "inputs/scripted.json"
samples = [0.0]
status = "exact"
"#,
        )
        .expect("write corpus");

        let options = Options::parse(vec![
            "--corpus".to_owned(),
            path.display().to_string(),
            "--runner-benchmark".to_owned(),
            "--benchmark-repeat".to_owned(),
            "11".to_owned(),
        ])
        .expect("parse options");
        let entries = parse_corpus(&path).expect("parse corpus");
        let error = corpus_targets(entries, &options, path.parent().unwrap()).unwrap_err();
        fs::remove_file(path).ok();

        assert!(error.contains("cannot be combined with input_script entry scripted"));
    }

    #[test]
    fn summarizes_measurements_by_sorted_median() {
        let summary = measurements_summary(vec![
            Duration::from_millis(30),
            Duration::from_millis(10),
            Duration::from_millis(20),
        ]);
        assert_eq!(summary.min, Duration::from_millis(10));
        assert_eq!(summary.median, Duration::from_millis(20));
        assert_eq!(summary.max, Duration::from_millis(30));
    }

    #[test]
    fn aggregate_mode_selects_threshold_statistic() {
        let file = file_result();
        let median = aggregate_results(std::slice::from_ref(&file), AggregateMode::Median);
        let min = aggregate_results(std::slice::from_ref(&file), AggregateMode::Min);

        assert_eq!(median.cpp_selected_sum(), Duration::from_millis(8));
        assert_eq!(median.rust_selected_sum(), Duration::from_millis(10));
        assert_eq!(median.rust_over_cpp, 1.25);
        assert_eq!(min.cpp_selected_sum(), Duration::from_millis(6));
        assert_eq!(min.rust_selected_sum(), Duration::from_millis(9));
        assert!((min.rust_over_cpp - 1.5).abs() < 1e-12);
    }

    #[test]
    fn parses_runner_benchmark_hot_loop_phases() {
        let benchmark = parse_benchmark_output(
            b"rive-golden-benchmark-v1\nelapsed_ms=99\ntotal_ms=7.75\nadvance_ms=1.5\ninput_ms=0.25\nprepare_ms=2.0\ndraw_ms=4.25\nbookkeeping_ms=91\nsegments=2\n",
        )
        .expect("parse benchmark phases");
        assert_eq!(benchmark.total, Some(Duration::from_micros(7_750)));
        assert_eq!(
            benchmark.phases,
            vec![
                ("advance", Duration::from_micros(1_500)),
                ("input", Duration::from_micros(250)),
                ("prepare", Duration::from_micros(2_000)),
                ("draw", Duration::from_micros(4_250)),
            ]
        );
    }

    #[test]
    fn rejects_runner_benchmark_without_phase_duration() {
        let error = parse_benchmark_output(
            b"rive-golden-benchmark-v1\nelapsed_ms=12.5\nadvance_ms=1\ninput_ms=0\nprepare_ms=0\nsegments=2\n",
        )
        .unwrap_err();
        assert!(error.contains("missing draw_ms"));
    }

    #[test]
    fn parses_json_and_meta_options() {
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--json".to_owned(),
            "out.json".to_owned(),
            "--meta".to_owned(),
            "git_sha=abc123".to_owned(),
            "--meta".to_owned(),
            "build_profile=release".to_owned(),
        ])
        .expect("parse options");

        assert_eq!(options.json, Some(PathBuf::from("out.json")));
        assert_eq!(
            options.meta,
            vec![
                ("git_sha".to_owned(), "abc123".to_owned()),
                ("build_profile".to_owned(), "release".to_owned()),
            ]
        );
    }

    #[test]
    fn rejects_meta_without_key_value_shape() {
        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--meta".to_owned(),
            "no-equals-sign".to_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("--meta expects key=value"));
    }

    fn sample(advance: u64, input: u64, prepare: u64, draw: u64) -> RunSample {
        let phases = vec![
            ("advance", Duration::from_millis(advance)),
            ("input", Duration::from_millis(input)),
            ("prepare", Duration::from_millis(prepare)),
            ("draw", Duration::from_millis(draw)),
        ];
        RunSample {
            total: phases.iter().map(|(_, duration)| *duration).sum(),
            phases,
        }
    }

    fn file_result() -> FileResult {
        FileResult {
            id: "single".to_owned(),
            file: "fixture \"quoted\".riv".to_owned(),
            segments: 2,
            cpp: summarize_samples(&[sample(1, 0, 2, 3), sample(2, 0, 2, 4), sample(1, 0, 2, 5)]),
            rust: summarize_samples(&[sample(2, 0, 3, 4), sample(2, 0, 3, 5), sample(2, 0, 3, 6)]),
        }
    }

    /// Minimal JSON validator: verifies the report parses as a single JSON
    /// value without relying on external crates.
    fn assert_valid_json(text: &str) {
        fn skip_whitespace(bytes: &[u8], mut index: usize) -> usize {
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            index
        }

        fn parse_value(bytes: &[u8], index: usize) -> Result<usize, String> {
            let index = skip_whitespace(bytes, index);
            match bytes.get(index) {
                Some(b'{') => parse_container(bytes, index + 1, b'}', true),
                Some(b'[') => parse_container(bytes, index + 1, b']', false),
                Some(b'"') => parse_string(bytes, index),
                Some(_) => parse_scalar(bytes, index),
                None => Err("unexpected end of input".to_owned()),
            }
        }

        fn parse_container(
            bytes: &[u8],
            mut index: usize,
            close: u8,
            is_object: bool,
        ) -> Result<usize, String> {
            index = skip_whitespace(bytes, index);
            if bytes.get(index) == Some(&close) {
                return Ok(index + 1);
            }
            loop {
                if is_object {
                    index = parse_string(bytes, skip_whitespace(bytes, index))?;
                    index = skip_whitespace(bytes, index);
                    if bytes.get(index) != Some(&b':') {
                        return Err(format!("expected ':' at byte {index}"));
                    }
                    index += 1;
                }
                index = parse_value(bytes, index)?;
                index = skip_whitespace(bytes, index);
                match bytes.get(index) {
                    Some(b',') => index += 1,
                    Some(byte) if *byte == close => return Ok(index + 1),
                    other => return Err(format!("unexpected {other:?} at byte {index}")),
                }
            }
        }

        fn parse_string(bytes: &[u8], index: usize) -> Result<usize, String> {
            if bytes.get(index) != Some(&b'"') {
                return Err(format!("expected string at byte {index}"));
            }
            let mut index = index + 1;
            while let Some(byte) = bytes.get(index) {
                match byte {
                    b'\\' => index += 2,
                    b'"' => return Ok(index + 1),
                    _ => index += 1,
                }
            }
            Err("unterminated string".to_owned())
        }

        fn parse_scalar(bytes: &[u8], index: usize) -> Result<usize, String> {
            let end = (index..bytes.len())
                .find(|&position| {
                    matches!(bytes[position], b',' | b'}' | b']')
                        || bytes[position].is_ascii_whitespace()
                })
                .unwrap_or(bytes.len());
            let token = std::str::from_utf8(&bytes[index..end]).map_err(|e| e.to_string())?;
            if token == "true"
                || token == "false"
                || token == "null"
                || token.parse::<f64>().is_ok()
            {
                Ok(end)
            } else {
                Err(format!("invalid scalar {token:?} at byte {index}"))
            }
        }

        let bytes = text.as_bytes();
        let index = parse_value(bytes, 0).expect("report must be valid JSON");
        assert_eq!(
            skip_whitespace(bytes, index),
            bytes.len(),
            "trailing content after JSON value"
        );
    }

    #[test]
    fn renders_valid_json_report_with_phases_and_meta() {
        let file = file_result();
        let aggregate = aggregate_results(std::slice::from_ref(&file), AggregateMode::Median);
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--runner-benchmark".to_owned(),
            "--iterations".to_owned(),
            "3".to_owned(),
            "--meta".to_owned(),
            "git_sha=abc123".to_owned(),
            "--meta".to_owned(),
            "build_profile=release".to_owned(),
            "--meta".to_owned(),
            "timestamp=2026-07-07T00:00:00Z".to_owned(),
        ])
        .expect("parse options");

        let report = render_json_report(&options, std::slice::from_ref(&file), &aggregate);
        assert_valid_json(&report);

        assert!(report.contains("\"schema\":\"rive-perf-compare-json-v1\""));
        assert!(report.contains("\"runner_order\":\"cpp-first\""));
        assert!(report.contains("\"metric\":\"runner_hot_loop_ms\""));
        // benchmark_repeat rides along whenever the hot-loop metric is used.
        assert!(report.contains("\"benchmark_repeat\":1"));
        assert!(report.contains("\"git_sha\":\"abc123\""));
        assert!(report.contains("\"build_profile\":\"release\""));
        assert!(report.contains("\"timestamp\":\"2026-07-07T00:00:00Z\""));
        assert!(report.contains("\"fixture \\\"quoted\\\".riv\""));
        // Per-phase stats and ratios for every hot-loop phase plus the total.
        for phase in ["total", "advance", "input", "prepare", "draw"] {
            assert!(
                report.contains(&format!("\"{phase}\":{{\"median_ms\":")),
                "missing phase stats for {phase}"
            );
        }
        assert!(report.contains("\"rust_over_cpp_by_phase\":{\"total\":"));
        assert!(report.contains("\"aggregate\":{\"entries\":1,\"segments\":2,"));
        assert!(report.contains("\"mode\":\"median\""));
        assert!(report.contains("\"cpp_min_ms_sum\":6.000000"));
        assert!(report.contains("\"rust_min_ms_sum\":9.000000"));
        assert!(report.contains("\"cpp_selected_ms_sum\":8.000000"));
        assert!(report.contains("\"rust_selected_ms_sum\":10.000000"));
        // cpp totals: 6,8,8 -> median 8; rust totals: 9,10,11 -> median 10.
        assert!(report.contains("\"rust_over_cpp\":1.250000"));
        // Zero-duration input phase must not produce NaN.
        assert!(!report.contains("NaN"));
        assert!(report.contains("\"input\":null") || report.contains("\"input\":0.000000"));
    }

    #[test]
    fn deterministic_report_for_identical_inputs() {
        let file = file_result();
        let aggregate = aggregate_results(std::slice::from_ref(&file), AggregateMode::Median);
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--meta".to_owned(),
            "timestamp=fixed".to_owned(),
        ])
        .expect("parse options");
        let first = render_json_report(&options, std::slice::from_ref(&file), &aggregate);
        let second = render_json_report(&options, std::slice::from_ref(&file), &aggregate);
        assert_eq!(first, second);
    }
}
