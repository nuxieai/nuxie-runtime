use std::collections::BTreeMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("golden-compare error: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1).collect())?;
    let mut corpus = parse_corpus(&options.corpus)?;
    if let Some(milestone) = options.milestone.as_deref() {
        corpus.retain(|entry| entry.milestone.as_deref() == Some(milestone));
    }
    if let Some(status) = options.status {
        corpus
            .retain(|entry| entry.effective_status(options.verify_scripted_diagnostics) == status);
    }
    if !options.ids.is_empty() {
        corpus.retain(|entry| options.ids.iter().any(|id| id == &entry.id));
    }
    if corpus.is_empty() {
        return Err(format!(
            "corpus {} contains no [[file]] entries",
            options.corpus.display()
        ));
    }

    let corpus_dir = options
        .corpus
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut counts = BTreeMap::<Status, usize>::new();
    let mut exact_segments = 0usize;
    let mut not_yet_rust_probed = 0usize;
    let mut not_yet_rust_exact = 0usize;
    let mut parked_by_milestone = BTreeMap::<String, usize>::new();
    let mut failures = Vec::new();

    for entry in &corpus {
        let status = entry.effective_status(options.verify_scripted_diagnostics);
        *counts.entry(status).or_default() += 1;
        if status == Status::Exact && entry.verification != VerificationMode::RejectsMalformed {
            exact_segments += entry.samples.len();
        }
        if status == Status::UnsupportedFeature {
            let bucket = entry
                .milestone
                .clone()
                .unwrap_or_else(|| "untagged".to_owned());
            *parked_by_milestone.entry(bucket).or_default() += 1;
        }
        if entry.requires_scripted_runner() && !options.verify_scripted_diagnostics {
            println!(
                "[{}] {}: skipped (requires scripted runners)",
                status, entry.id
            );
            continue;
        }
        match status {
            Status::UnsupportedFeature => {
                if options.verify_unsupported_cpp {
                    let file = resolve_asset_path(&entry.path, &options.rive_runtime_dir);
                    match run_stream(
                        &options.cpp_runner,
                        entry,
                        &file,
                        &corpus_dir,
                        RunnerKind::Cpp,
                        options.verify_scripted_diagnostics,
                    ) {
                        Ok(cpp_stream) => println!(
                            "[unsupported-feature] {}: c++ stream ok ({} bytes)",
                            entry.id,
                            cpp_stream.len()
                        ),
                        Err(error) => failures.push(format!("{}: {error}", entry.id)),
                    }
                }
                println!(
                    "[unsupported-feature] {}: skipped ({})",
                    entry.id,
                    entry.features.join(", ")
                );
                let unsupported_feature = entry.rust_runner_unsupported_feature().or_else(|| {
                    options
                        .verify_scripted_diagnostics
                        .then(|| entry.scripted_rust_runner_unsupported_feature())
                        .flatten()
                });
                if let Some(feature) = unsupported_feature {
                    match &options.rust_runner {
                        Some(rust_runner) => {
                            let file = resolve_asset_path(&entry.path, &options.rive_runtime_dir);
                            match run_unsupported_diagnostic(
                                rust_runner,
                                entry,
                                &file,
                                &corpus_dir,
                                feature,
                                options.verify_scripted_diagnostics,
                            ) {
                                Ok(()) => println!(
                                    "[unsupported-feature] {}: rust diagnostic ok ({feature})",
                                    entry.id
                                ),
                                Err(error) => failures.push(format!("{}: {error}", entry.id)),
                            }
                        }
                        None => failures.push(format!(
                            "{}: unsupported feature {feature} requires --rust-runner to verify diagnostic",
                            entry.id
                        )),
                    }
                }
            }
            Status::Exact if entry.verification == VerificationMode::RejectsMalformed => {
                let file = resolve_asset_path(&entry.path, &options.rive_runtime_dir);
                let expected_rust_error = entry
                    .import_error_feature()
                    .expect("rejects-malformed entries are validated before execution");

                match run_malformed_rejection(
                    &options.cpp_runner,
                    entry,
                    &file,
                    &corpus_dir,
                    RunnerKind::Cpp,
                    expected_rust_error,
                    options.verify_scripted_diagnostics,
                ) {
                    Ok(()) => println!(
                        "[exact] {}: c++ rejected malformed import as expected",
                        entry.id
                    ),
                    Err(error) => failures.push(format!("{}: {error}", entry.id)),
                }

                match &options.rust_runner {
                    Some(rust_runner) => match run_malformed_rejection(
                        rust_runner,
                        entry,
                        &file,
                        &corpus_dir,
                        RunnerKind::Rust,
                        expected_rust_error,
                        options.verify_scripted_diagnostics,
                    ) {
                        Ok(()) => println!(
                            "[exact] {}: rust rejected the same malformed import as expected",
                            entry.id
                        ),
                        Err(error) => failures.push(format!("{}: {error}", entry.id)),
                    },
                    None => failures.push(format!(
                        "{}: rejects-malformed verification requires --rust-runner",
                        entry.id
                    )),
                }
            }
            Status::NotYet | Status::Diverges | Status::Exact => {
                let file = resolve_asset_path(&entry.path, &options.rive_runtime_dir);
                match run_stream(
                    &options.cpp_runner,
                    entry,
                    &file,
                    &corpus_dir,
                    RunnerKind::Cpp,
                    options.verify_scripted_diagnostics,
                ) {
                    Ok(cpp_stream) => {
                        println!(
                            "[{}] {}: c++ stream ok ({} bytes)",
                            status,
                            entry.id,
                            cpp_stream.len()
                        );
                        let run_rust = status == Status::Exact
                            || (status == Status::Diverges && options.verify_divergent_rust)
                            || (status == Status::NotYet && options.probe_not_yet_rust);
                        if run_rust {
                            match &options.rust_runner {
                                Some(rust_runner) => {
                                    match run_stream(
                                        rust_runner,
                                        entry,
                                        &file,
                                        &corpus_dir,
                                        RunnerKind::Rust,
                                        options.verify_scripted_diagnostics,
                                    ) {
                                        Ok(rust_stream) if status == Status::Diverges => println!(
                                            "[diverges] {}: rust stream ok ({} bytes)",
                                            entry.id,
                                            rust_stream.len()
                                        ),
                                        Ok(rust_stream) if status == Status::NotYet => {
                                            not_yet_rust_probed += 1;
                                            if let Some(difference) = entry
                                                .verification
                                                .stream_difference(&rust_stream, &cpp_stream)
                                            {
                                                println!(
                                                    "[not-yet] {}: rust differs from C++ under {} verification: {difference}",
                                                    entry.id, entry.verification
                                                );
                                            } else {
                                                not_yet_rust_exact += 1;
                                                println!(
                                                    "[not-yet] {}: rust stream is exact ({} bytes)",
                                                    entry.id,
                                                    rust_stream.len()
                                                );
                                            }
                                        }
                                        Ok(rust_stream) => {
                                            if let Some(difference) = entry
                                                .verification
                                                .stream_difference(&rust_stream, &cpp_stream)
                                            {
                                                failures.push(format!(
                                                    "{}: stream differs from C++ under {} verification: {difference}",
                                                    entry.id, entry.verification
                                                ));
                                            }
                                        }
                                        Err(error) if status == Status::NotYet => {
                                            not_yet_rust_probed += 1;
                                            println!(
                                                "[not-yet] {}: rust runner failed: {error}",
                                                entry.id
                                            );
                                        }
                                        Err(error) => failures.push(format!("{}: {error}", entry.id)),
                                    }
                                }
                                None => failures.push(format!(
                                    "{}: rust verification was requested but --rust-runner was not supplied",
                                    entry.id,
                                )),
                            }
                        }
                    }
                    Err(error) => failures.push(format!("{}: {error}", entry.id)),
                }
            }
        }
    }

    let exact = counts.get(&Status::Exact).copied().unwrap_or(0);
    println!(
        "golden-compare summary: entries={} exact={} exact-segments={} diverges={} unsupported-feature={} not-yet={}",
        corpus.len(),
        exact,
        exact_segments,
        counts.get(&Status::Diverges).copied().unwrap_or(0),
        counts
            .get(&Status::UnsupportedFeature)
            .copied()
            .unwrap_or(0),
        counts.get(&Status::NotYet).copied().unwrap_or(0),
    );
    if options.probe_not_yet_rust {
        println!(
            "golden-compare not-yet probe: rust-exact={} probed={}",
            not_yet_rust_exact, not_yet_rust_probed
        );
    }
    if !parked_by_milestone.is_empty() {
        let breakdown = parked_by_milestone
            .iter()
            .map(|(milestone, count)| format!("{milestone}={count}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!("golden-compare parked: {breakdown}");
    }

    if failures.is_empty() {
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("failure: {failure}");
        }
        Err(format!("{} corpus entries failed", failures.len()))
    }
}

#[derive(Debug)]
struct Options {
    corpus: PathBuf,
    cpp_runner: PathBuf,
    rust_runner: Option<PathBuf>,
    rive_runtime_dir: PathBuf,
    milestone: Option<String>,
    status: Option<Status>,
    ids: Vec<String>,
    verify_unsupported_cpp: bool,
    verify_divergent_rust: bool,
    probe_not_yet_rust: bool,
    verify_scripted_diagnostics: bool,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut corpus = PathBuf::from("corpus.toml");
        let mut cpp_runner = env::var_os("GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(default_cpp_runner);
        let mut rust_runner = None;
        let mut milestone = None;
        let mut status = None;
        let mut ids = Vec::new();
        let mut verify_unsupported_cpp = false;
        let mut verify_divergent_rust = false;
        let mut probe_not_yet_rust = false;
        let mut verify_scripted_diagnostics = false;
        let mut rive_runtime_dir = env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));

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
                "--corpus" => corpus = PathBuf::from(value(arg)?),
                "--cpp-runner" => cpp_runner = PathBuf::from(value(arg)?),
                "--rust-runner" => rust_runner = Some(PathBuf::from(value(arg)?)),
                "--rive-runtime-dir" => rive_runtime_dir = PathBuf::from(value(arg)?),
                "--milestone" => milestone = Some(value(arg)?),
                "--status" => status = Some(Status::parse(&value(arg)?)?),
                "--id" => ids.push(value(arg)?),
                "--verify-unsupported-cpp" => verify_unsupported_cpp = true,
                "--verify-divergent-rust" => verify_divergent_rust = true,
                "--probe-not-yet-rust" => probe_not_yet_rust = true,
                "--verify-scripted-diagnostics" => verify_scripted_diagnostics = true,
                "--help" | "-h" => {
                    println!(
                        "usage: golden-compare [--corpus corpus.toml] [--milestone name] [--status exact|diverges|unsupported-feature|not-yet] [--id corpus-id]... [--verify-unsupported-cpp] [--verify-divergent-rust] [--probe-not-yet-rust] [--verify-scripted-diagnostics] --cpp-runner <path> [--rust-runner <path>]"
                    );
                    std::process::exit(0);
                }
                other => return Err(format!("unknown option: {other}")),
            }
            index += 1;
        }

        Ok(Self {
            corpus,
            cpp_runner,
            rust_runner,
            rive_runtime_dir,
            milestone,
            status,
            ids,
            verify_unsupported_cpp,
            verify_divergent_rust,
            probe_not_yet_rust,
            verify_scripted_diagnostics,
        })
    }
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

#[derive(Debug, Clone)]
struct CorpusEntry {
    id: String,
    path: String,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<String>,
    rust_execute_scripts: bool,
    samples: Vec<f32>,
    status: Status,
    verification: VerificationMode,
    milestone: Option<String>,
    features: Vec<String>,
}

impl CorpusEntry {
    fn new() -> Self {
        Self {
            id: String::new(),
            path: String::new(),
            artboard: None,
            state_machine: None,
            input_script: None,
            rust_execute_scripts: false,
            samples: vec![0.0],
            status: Status::NotYet,
            verification: VerificationMode::Exact,
            milestone: None,
            features: Vec::new(),
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
        for pair in self.samples.windows(2) {
            if pair[1] < pair[0] {
                return Err(format!("entry {} samples must be sorted", self.id));
            }
        }
        if self.verification == VerificationMode::RejectsMalformed {
            if self.status != Status::Exact {
                return Err(format!(
                    "entry {} uses rejects-malformed verification but is not exact",
                    self.id
                ));
            }
            if self.import_error_feature().is_none_or(str::is_empty) {
                return Err(format!(
                    "entry {} uses rejects-malformed verification without an import-error feature",
                    self.id
                ));
            }
        }
        Ok(())
    }

    fn import_error_feature(&self) -> Option<&str> {
        self.features
            .iter()
            .find_map(|feature| feature.strip_prefix("import-error:"))
    }

    fn rust_runner_unsupported_feature(&self) -> Option<&str> {
        self.features
            .iter()
            .find_map(|feature| feature.strip_prefix("rust-runner-unsupported:"))
    }

    fn scripted_rust_runner_unsupported_feature(&self) -> Option<&str> {
        self.features
            .iter()
            .find_map(|feature| feature.strip_prefix("scripted-rust-runner-unsupported:"))
    }

    fn requires_scripted_runner(&self) -> bool {
        self.rust_execute_scripts || self.has_scripted_runner_only_feature()
    }

    fn executes_scripts_in_rust(&self) -> bool {
        self.rust_execute_scripts || self.has_scripted_runner_only_feature()
    }

    fn has_scripted_runner_only_feature(&self) -> bool {
        self.features
            .iter()
            .any(|feature| feature == "scripted-runner-only")
    }

    fn effective_status(&self, scripted: bool) -> Status {
        if scripted
            && self
                .features
                .iter()
                .any(|feature| feature == "scripted-status:exact")
        {
            Status::Exact
        } else {
            self.status
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Exact,
    Diverges,
    UnsupportedFeature,
    NotYet,
}

impl Status {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "exact" => Ok(Self::Exact),
            "diverges" => Ok(Self::Diverges),
            "unsupported-feature" => Ok(Self::UnsupportedFeature),
            "not-yet" => Ok(Self::NotYet),
            other => Err(format!("unknown corpus status: {other}")),
        }
    }
}

impl Display for Status {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Status::Exact => "exact",
            Status::Diverges => "diverges",
            Status::UnsupportedFeature => "unsupported-feature",
            Status::NotYet => "not-yet",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum VerificationMode {
    Exact,
    Tolerant(f64),
    Structural,
    RejectsMalformed,
}

impl VerificationMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "exact" => Ok(Self::Exact),
            "structural" => Ok(Self::Structural),
            "rejects-malformed" => Ok(Self::RejectsMalformed),
            _ => {
                let Some(inner) = value
                    .strip_prefix("tolerant(")
                    .and_then(|value| value.strip_suffix(')'))
                else {
                    return Err(format!("unknown corpus verification mode: {value}"));
                };
                let epsilon = inner
                    .parse::<f64>()
                    .map_err(|error| format!("invalid tolerant epsilon {inner}: {error}"))?;
                if !epsilon.is_finite() || epsilon.is_sign_negative() {
                    return Err(format!(
                        "tolerant epsilon must be finite and non-negative: {inner}"
                    ));
                }
                Ok(Self::Tolerant(epsilon))
            }
        }
    }

    #[cfg(test)]
    fn streams_match(self, left: &str, right: &str) -> bool {
        self.stream_difference(left, right).is_none()
    }

    fn stream_difference(self, left: &str, right: &str) -> Option<String> {
        match self {
            Self::Exact => stream_difference_with_epsilon(left, right, GOLDEN_FLOAT_EPSILON),
            Self::Tolerant(epsilon) => stream_difference_with_epsilon(left, right, epsilon),
            Self::Structural => {
                stream_difference_with_epsilon(left, right, STRUCTURAL_FLOAT_EPSILON)
            }
            Self::RejectsMalformed => Some(
                "rejects-malformed verifies runner failures rather than render streams".to_owned(),
            ),
        }
    }
}

impl Display for VerificationMode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => formatter.write_str("exact"),
            Self::Tolerant(epsilon) => write!(formatter, "tolerant({epsilon})"),
            Self::Structural => formatter.write_str("structural"),
            Self::RejectsMalformed => formatter.write_str("rejects-malformed"),
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
        let key = key.trim();
        let value = value.trim();

        match key {
            "id" => entry.id = parse_string(value, line_number)?,
            "path" => entry.path = parse_string(value, line_number)?,
            "artboard" => entry.artboard = Some(parse_string(value, line_number)?),
            "state_machine" => entry.state_machine = Some(parse_string(value, line_number)?),
            "input_script" => entry.input_script = Some(parse_string(value, line_number)?),
            "rust_execute_scripts" => entry.rust_execute_scripts = parse_bool(value, line_number)?,
            "samples" => entry.samples = parse_float_array(value, line_number)?,
            "status" => entry.status = Status::parse(&parse_string(value, line_number)?)?,
            "verification" => {
                entry.verification = VerificationMode::parse(&parse_string(value, line_number)?)?
            }
            "milestone" => entry.milestone = Some(parse_string(value, line_number)?),
            "features" => entry.features = parse_string_array(value, line_number)?,
            other => return Err(format!("line {line_number}: unknown key {other}")),
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
    let mut out = String::new();
    let mut escaped = false;
    for ch in value[1..value.len() - 1].chars() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    if escaped {
        return Err(format!("line {line}: dangling string escape"));
    }
    Ok(out)
}

fn parse_string_array(value: &str, line: usize) -> Result<Vec<String>, String> {
    let inner = array_inner(value, line)?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|part| parse_string(part.trim(), line))
        .collect()
}

fn parse_float_array(value: &str, line: usize) -> Result<Vec<f32>, String> {
    let inner = array_inner(value, line)?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f32>()
                .map_err(|error| format!("line {line}: invalid sample {}: {error}", part.trim()))
        })
        .collect()
}

fn parse_bool(value: &str, line: usize) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("line {line}: expected true or false")),
    }
}

fn array_inner(value: &str, line: usize) -> Result<&str, String> {
    let value = value.trim();
    if !value.starts_with('[') || !value.ends_with(']') {
        return Err(format!("line {line}: expected array"));
    }
    Ok(&value[1..value.len() - 1])
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

const GOLDEN_FLOAT_EPSILON: f64 = 1.3e-4;
const STRUCTURAL_FLOAT_EPSILON: f64 = GOLDEN_FLOAT_EPSILON;

#[cfg(test)]
fn streams_equivalent(left: &str, right: &str) -> bool {
    streams_equivalent_with_epsilon(left, right, GOLDEN_FLOAT_EPSILON)
}

#[cfg(test)]
fn streams_equivalent_with_epsilon(left: &str, right: &str, epsilon: f64) -> bool {
    stream_difference_with_epsilon(left, right, epsilon).is_none()
}

fn stream_difference_with_epsilon(left: &str, right: &str, epsilon: f64) -> Option<String> {
    if left == right {
        return None;
    }
    if left.ends_with('\n') != right.ends_with('\n') {
        return Some(format!(
            "stream newline termination differs (rust ends with newline: {}, c++ ends with newline: {})",
            left.ends_with('\n'),
            right.ends_with('\n')
        ));
    }

    let mut left_lines = left.lines();
    let mut right_lines = right.lines();
    let mut line_number = 1usize;
    loop {
        match (left_lines.next(), right_lines.next()) {
            (Some(left), Some(right)) if line_equivalent(left, right, epsilon) => {}
            (Some(left), Some(right)) => {
                return Some(format!(
                    "line {line_number}: rust `{}` vs c++ `{}`",
                    summarize_stream_line(left),
                    summarize_stream_line(right)
                ));
            }
            (Some(left), None) => {
                return Some(format!(
                    "line {line_number}: rust has extra `{}`",
                    summarize_stream_line(left)
                ));
            }
            (None, Some(right)) => {
                return Some(format!(
                    "line {line_number}: c++ has extra `{}`",
                    summarize_stream_line(right)
                ));
            }
            (None, None) => return None,
        }
        line_number += 1;
    }
}

fn summarize_stream_line(line: &str) -> String {
    const MAX_LEN: usize = 240;
    if line.len() <= MAX_LEN {
        return line.to_owned();
    }
    format!("{}...", &line[..MAX_LEN])
}

fn line_equivalent(left: &str, right: &str, epsilon: f64) -> bool {
    if let Some(equivalent) = buffer_data_line_equivalent(left, right, epsilon) {
        return equivalent;
    }

    let left_bytes = left.as_bytes();
    let right_bytes = right.as_bytes();
    let mut left_index = 0usize;
    let mut right_index = 0usize;

    while left_index < left_bytes.len() && right_index < right_bytes.len() {
        if number_starts_at(left_bytes, left_index) && number_starts_at(right_bytes, right_index) {
            let left_end = number_end(left_bytes, left_index);
            let right_end = number_end(right_bytes, right_index);
            let Ok(left_number) = left[left_index..left_end].parse::<f64>() else {
                return false;
            };
            let Ok(right_number) = right[right_index..right_end].parse::<f64>() else {
                return false;
            };
            if (left_number - right_number).abs() > epsilon {
                return false;
            }
            left_index = left_end;
            right_index = right_end;
            continue;
        }

        if left_bytes[left_index] != right_bytes[right_index] {
            return false;
        }
        left_index += 1;
        right_index += 1;
    }

    left_index == left_bytes.len() && right_index == right_bytes.len()
}

fn buffer_data_line_equivalent(left: &str, right: &str, epsilon: f64) -> Option<bool> {
    if !left.starts_with("bufferData ") || !right.starts_with("bufferData ") {
        return None;
    }

    let (left_prefix, left_hex) = left.split_once(" data=")?;
    let (right_prefix, right_hex) = right.split_once(" data=")?;
    if left_prefix != right_prefix {
        return Some(false);
    }

    let buffer_type = buffer_data_type(left_prefix)?;
    if buffer_type != 1 {
        return Some(left_hex == right_hex);
    }

    Some(vertex_buffer_hex_equivalent(left_hex, right_hex, epsilon))
}

fn buffer_data_type(prefix: &str) -> Option<u8> {
    let marker = " type=";
    let start = prefix.find(marker)? + marker.len();
    let end = prefix[start..]
        .find(' ')
        .map(|offset| start + offset)
        .unwrap_or(prefix.len());
    prefix[start..end].parse().ok()
}

fn vertex_buffer_hex_equivalent(left_hex: &str, right_hex: &str, epsilon: f64) -> bool {
    let Some(left) = decode_hex_bytes(left_hex) else {
        return false;
    };
    let Some(right) = decode_hex_bytes(right_hex) else {
        return false;
    };
    if left.len() != right.len() || left.len() % 4 != 0 {
        return false;
    }

    left.chunks_exact(4)
        .zip(right.chunks_exact(4))
        .all(|(left, right)| {
            let left = f32::from_le_bytes([left[0], left[1], left[2], left[3]]);
            let right = f32::from_le_bytes([right[0], right[1], right[2], right[3]]);
            left.to_bits() == right.to_bits()
                || (f64::from(left) - f64::from(right)).abs() <= epsilon
        })
}

fn decode_hex_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let bytes = hex.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = hex_nibble(chunk[0])?;
        let low = hex_nibble(chunk[1])?;
        decoded.push((high << 4) | low);
    }
    Some(decoded)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn number_starts_at(bytes: &[u8], index: usize) -> bool {
    let byte = bytes[index];
    if byte.is_ascii_digit() {
        return true;
    }
    if byte == b'-' || byte == b'+' {
        return bytes
            .get(index + 1)
            .is_some_and(|next| next.is_ascii_digit() || *next == b'.');
    }
    byte == b'.'
        && bytes
            .get(index + 1)
            .is_some_and(|next| next.is_ascii_digit())
}

fn number_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    if matches!(bytes.get(index), Some(b'-' | b'+')) {
        index += 1;
    }

    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }

    if matches!(bytes.get(index), Some(b'.')) {
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_start = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'-' | b'+')) {
            index += 1;
        }
        let digits_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if digits_start == index {
            return exponent_start;
        }
    }

    index
}

fn unsupported_diagnostic_matches(stderr: &str, expected_feature: &str) -> bool {
    stderr.contains(&format!("unsupported: {expected_feature}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_can_filter_and_probe_not_yet_entries() {
        let options = Options::parse(vec![
            "--cpp-runner".to_owned(),
            "cpp-runner".to_owned(),
            "--rust-runner".to_owned(),
            "rust-runner".to_owned(),
            "--status".to_owned(),
            "not-yet".to_owned(),
            "--id".to_owned(),
            "one".to_owned(),
            "--id".to_owned(),
            "two".to_owned(),
            "--probe-not-yet-rust".to_owned(),
        ])
        .unwrap();

        assert_eq!(options.status, Some(Status::NotYet));
        assert_eq!(options.ids, ["one", "two"]);
        assert!(options.probe_not_yet_rust);
        assert_eq!(options.rust_runner, Some(PathBuf::from("rust-runner")));
    }

    #[test]
    fn scripted_status_override_only_applies_in_scripted_mode() {
        let mut entry = CorpusEntry::new();
        entry.status = Status::Diverges;
        entry.features.push("scripted-status:exact".to_owned());

        assert_eq!(entry.effective_status(false), Status::Diverges);
        assert_eq!(entry.effective_status(true), Status::Exact);
    }

    #[test]
    fn scripted_lane_forces_rust_script_execution() {
        let entry = CorpusEntry::new();

        let scripted_rust = stream_command(
            Path::new("rust-runner"),
            &entry,
            Path::new("fixture.riv"),
            Path::new("corpus"),
            RunnerKind::Rust,
            true,
        );
        assert!(
            scripted_rust
                .get_args()
                .any(|argument| argument == "--execute-scripts")
        );

        let normal_rust = stream_command(
            Path::new("rust-runner"),
            &entry,
            Path::new("fixture.riv"),
            Path::new("corpus"),
            RunnerKind::Rust,
            false,
        );
        assert!(
            normal_rust
                .get_args()
                .all(|argument| argument != "--execute-scripts")
        );

        let scripted_cpp = stream_command(
            Path::new("cpp-runner"),
            &entry,
            Path::new("fixture.riv"),
            Path::new("corpus"),
            RunnerKind::Cpp,
            true,
        );
        assert!(
            scripted_cpp
                .get_args()
                .all(|argument| argument != "--execute-scripts")
        );
    }

    #[test]
    fn stream_comparison_allows_float_epsilon() {
        assert!(streams_equivalent(
            "drawPath points=[(-15.2626038,-125)]\n",
            "drawPath points=[(-15.2626648,-125)]\n"
        ));
    }

    #[test]
    fn stream_comparison_allows_local_path_float_cancellation() {
        assert!(streams_equivalent(
            "drawPath points=[(-7.31272936,-2.03849483)]\n",
            "drawPath points=[(-7.31272936,-2.03837299)]\n"
        ));
        assert!(!streams_equivalent(
            "drawPath points=[(-7.31272936,-2.03849483)]\n",
            "drawPath points=[(-7.31272936,-2.03825000)]\n"
        ));
    }

    #[test]
    fn stream_comparison_allows_vertex_buffer_float_epsilon() {
        assert!(streams_equivalent(
            &format!(
                "bufferData id=7 type=1 size=8 data={}\n",
                f32_hex(&[1.0, -2.0])
            ),
            &format!(
                "bufferData id=7 type=1 size=8 data={}\n",
                f32_hex(&[1.00005, -2.00005])
            )
        ));
        assert!(!streams_equivalent(
            &format!(
                "bufferData id=7 type=1 size=8 data={}\n",
                f32_hex(&[1.0, -2.0])
            ),
            &format!(
                "bufferData id=7 type=1 size=8 data={}\n",
                f32_hex(&[1.001, -2.0])
            )
        ));
    }

    #[test]
    fn stream_comparison_keeps_index_buffer_data_exact() {
        assert!(!streams_equivalent(
            "bufferData id=3 type=0 size=2 data=0001\n",
            "bufferData id=3 type=0 size=2 data=0002\n"
        ));
    }

    #[test]
    fn stream_comparison_rejects_structural_differences() {
        assert!(!streams_equivalent("drawPath id=2\n", "clipPath id=2\n"));
        assert!(!streams_equivalent("drawPath id=2\n", "drawPath id=3\n"));
        assert!(!streams_equivalent("drawPath id=2\n", "drawPath id=2"));
    }

    #[test]
    fn verification_mode_parses_declared_modes() {
        assert_eq!(
            VerificationMode::parse("exact").unwrap(),
            VerificationMode::Exact
        );
        assert_eq!(
            VerificationMode::parse("structural").unwrap(),
            VerificationMode::Structural
        );
        assert_eq!(
            VerificationMode::parse("tolerant(0.25)").unwrap(),
            VerificationMode::Tolerant(0.25)
        );
        assert_eq!(
            VerificationMode::parse("rejects-malformed").unwrap(),
            VerificationMode::RejectsMalformed
        );
        assert!(VerificationMode::parse("tolerant(-0.1)").is_err());
        assert!(VerificationMode::parse("tolerant(nan)").is_err());
        assert!(VerificationMode::parse("loose").is_err());
    }

    #[test]
    fn tolerant_verification_uses_declared_epsilon() {
        let left = "drawPath points=[(0,0)]\n";
        let right = "drawPath points=[(0.004,0)]\n";

        assert!(!VerificationMode::Exact.streams_match(left, right));
        assert!(!VerificationMode::Tolerant(0.001).streams_match(left, right));
        assert!(VerificationMode::Tolerant(0.005).streams_match(left, right));
        assert!(
            !VerificationMode::Tolerant(0.005)
                .streams_match("drawPath points=[(0,0)]\n", "clipPath points=[(0,0)]\n")
        );
    }

    #[test]
    fn unsupported_diagnostic_must_match_manifest_feature() {
        assert!(unsupported_diagnostic_matches(
            "rust-golden-runner error: unsupported: scroll-constraints in Rust golden runner\n",
            "scroll-constraints"
        ));
        assert!(!unsupported_diagnostic_matches(
            "rust-golden-runner error: unsupported: data-binding-nested-child in Rust golden runner\n",
            "scroll-constraints"
        ));
    }

    #[test]
    fn malformed_rejection_requires_the_expected_failure_category() {
        let cpp_error = "golden-runner error: bad riv file; import result=2\n";
        let rust_error = concat!(
            "rust-golden-runner error: failed to import runtime file: ",
            "drawable object 13 (Shape) has invalid blendModeValue 5\n"
        );
        let expected = "drawable-object-13-Shape-has-invalid-blendModeValue-5";

        assert!(
            validate_malformed_rejection(RunnerKind::Cpp, false, Some(1), "", cpp_error, expected)
                .is_ok()
        );
        assert!(
            validate_malformed_rejection(
                RunnerKind::Rust,
                false,
                Some(1),
                "",
                rust_error,
                expected,
            )
            .is_ok()
        );

        assert!(
            validate_malformed_rejection(
                RunnerKind::Cpp,
                false,
                Some(1),
                "",
                "golden-runner error: bad riv file; import result=1\n",
                expected,
            )
            .is_err()
        );
        assert!(
            validate_malformed_rejection(
                RunnerKind::Rust,
                false,
                Some(1),
                "",
                rust_error,
                "a-different-import-error",
            )
            .is_err()
        );
        assert!(
            validate_malformed_rejection(
                RunnerKind::Rust,
                false,
                Some(101),
                "",
                "thread panicked\n",
                expected,
            )
            .is_err()
        );
        assert!(
            validate_malformed_rejection(RunnerKind::Cpp, false, None, "", cpp_error, expected,)
                .is_err()
        );
        assert!(
            validate_malformed_rejection(
                RunnerKind::Cpp,
                false,
                Some(1),
                "rive-golden-stream-v1\n",
                cpp_error,
                expected,
            )
            .is_err()
        );
        assert!(
            validate_malformed_rejection(RunnerKind::Cpp, true, Some(0), "", "", expected,)
                .is_err()
        );
    }

    #[test]
    fn corpus_parser_accepts_verification_mode() {
        let path = std::env::temp_dir().join(format!(
            "golden-compare-verification-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(
            &path,
            r#"
[[file]]
id = "layoutish"
path = "tests/unit_tests/assets/layoutish.riv"
samples = [0.0]
status = "exact"
verification = "tolerant(0.25)"
rust_execute_scripts = true
features = []

[[file]]
id = "malformed"
path = "tests/unit_tests/assets/malformed.riv"
samples = [0.0]
status = "exact"
verification = "rejects-malformed"
features = ["import-error:invalid-object"]

[[file]]
id = "scripted"
path = "tests/unit_tests/assets/scripted.riv"
samples = [0.0]
status = "exact"
features = ["scripted-runner-only", "scripted-status:exact"]
"#,
        )
        .unwrap();

        let entries = parse_corpus(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].verification, VerificationMode::Tolerant(0.25));
        assert!(entries[0].rust_execute_scripts);
        assert!(entries[0].requires_scripted_runner());
        assert!(entries[0].executes_scripts_in_rust());
        assert!(!entries[1].rust_execute_scripts);
        assert_eq!(entries[1].verification, VerificationMode::RejectsMalformed);
        assert!(!entries[1].requires_scripted_runner());
        assert!(!entries[1].executes_scripts_in_rust());
        assert!(!entries[2].rust_execute_scripts);
        assert!(entries[2].requires_scripted_runner());
        assert!(entries[2].executes_scripts_in_rust());
    }

    fn f32_hex(values: &[f32]) -> String {
        let mut hex = String::new();
        for value in values {
            for byte in value.to_le_bytes() {
                hex.push_str(&format!("{byte:02x}"));
            }
        }
        hex
    }
}

fn run_stream(
    runner: &Path,
    entry: &CorpusEntry,
    file: &Path,
    corpus_dir: &Path,
    runner_kind: RunnerKind,
    scripted_lane: bool,
) -> Result<String, String> {
    let mut command = stream_command(runner, entry, file, corpus_dir, runner_kind, scripted_lane);
    let output = command
        .output()
        .map_err(|error| format!("failed to run {}: {error}", runner.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{} exited with {}\n{}",
            runner.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("{} emitted non-utf8 stream: {error}", runner.display()))?;
    let Some(stream_start) = stdout.find("rive-golden-stream-v1\n") else {
        return Err(format!(
            "{} did not emit a rive-golden stream",
            runner.display()
        ));
    };
    Ok(stdout[stream_start..].to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunnerKind {
    Cpp,
    Rust,
}

fn run_malformed_rejection(
    runner: &Path,
    entry: &CorpusEntry,
    file: &Path,
    corpus_dir: &Path,
    runner_kind: RunnerKind,
    expected_rust_error: &str,
    scripted_lane: bool,
) -> Result<(), String> {
    let mut command = stream_command(runner, entry, file, corpus_dir, runner_kind, scripted_lane);
    let output = command
        .output()
        .map_err(|error| format!("failed to run {}: {error}", runner.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    validate_malformed_rejection(
        runner_kind,
        output.status.success(),
        output.status.code(),
        &stdout,
        &stderr,
        expected_rust_error,
    )
    .map_err(|reason| {
        format!(
            "{} did not produce the expected malformed-import rejection: {reason}\n{}",
            runner.display(),
            stderr
        )
    })
}

fn validate_malformed_rejection(
    runner_kind: RunnerKind,
    success: bool,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
    expected_rust_error: &str,
) -> Result<(), String> {
    if success {
        return Err("runner succeeded".to_owned());
    }
    if exit_code != Some(1) {
        return Err(match exit_code {
            Some(code) => format!("runner exited with unexpected code {code}"),
            None => "runner terminated without an exit code (for example, by signal)".to_owned(),
        });
    }
    if stdout.contains("rive-golden-stream-v1\n") {
        return Err("runner emitted a golden stream before failing".to_owned());
    }

    let matches_expected_import_failure = match runner_kind {
        RunnerKind::Cpp => cpp_malformed_diagnostic_matches(stderr),
        RunnerKind::Rust => rust_malformed_diagnostic_matches(stderr, expected_rust_error),
    };
    if !matches_expected_import_failure {
        return Err("runner emitted a non-import failure diagnostic".to_owned());
    }
    Ok(())
}

fn cpp_malformed_diagnostic_matches(stderr: &str) -> bool {
    const MARKER: &str = "bad riv file; import result=";
    stderr.lines().any(|line| {
        line.split_once(MARKER)
            .and_then(|(_, result)| result.split_whitespace().next())
            .is_some_and(|result| result == "2")
    })
}

fn rust_malformed_diagnostic_matches(stderr: &str, expected_error: &str) -> bool {
    stderr.contains("rust-golden-runner error:")
        && stderr.contains("failed to import runtime file:")
        && !expected_error.is_empty()
        && normalize_import_error(stderr)
            .to_ascii_lowercase()
            .contains(&expected_error.to_ascii_lowercase())
}

fn normalize_import_error(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn run_unsupported_diagnostic(
    runner: &Path,
    entry: &CorpusEntry,
    file: &Path,
    corpus_dir: &Path,
    expected_feature: &str,
    scripted_lane: bool,
) -> Result<(), String> {
    let mut command = stream_command(
        runner,
        entry,
        file,
        corpus_dir,
        RunnerKind::Rust,
        scripted_lane,
    );
    let output = command
        .output()
        .map_err(|error| format!("failed to run {}: {error}", runner.display()))?;
    if output.status.success() {
        return Err(format!(
            "{} succeeded; expected unsupported diagnostic",
            runner.display()
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.contains("unsupported:") {
        return Err(format!(
            "{} did not emit an unsupported diagnostic\n{}",
            runner.display(),
            stderr
        ));
    }
    if !unsupported_diagnostic_matches(&stderr, expected_feature) {
        let expected_marker = format!("unsupported: {expected_feature}");
        return Err(format!(
            "{} emitted the wrong unsupported diagnostic; expected {expected_marker:?}\n{}",
            runner.display(),
            stderr
        ));
    }

    Ok(())
}

fn stream_command(
    runner: &Path,
    entry: &CorpusEntry,
    file: &Path,
    corpus_dir: &Path,
    runner_kind: RunnerKind,
    scripted_lane: bool,
) -> Command {
    let mut command = Command::new(runner);
    command.arg("--file").arg(file);
    // A scripted C++ runner always imports through WITH_RIVE_SCRIPTING, even
    // for files without ScriptAsset objects. Mirror that build/runtime mode on
    // the Rust side for the entire scripted lane. Per-entry flags only decide
    // which files require the scripted lane when running the normal target.
    if runner_kind == RunnerKind::Rust && (scripted_lane || entry.executes_scripts_in_rust()) {
        command.arg("--execute-scripts");
    }
    if let Some(artboard) = &entry.artboard {
        command.arg("--artboard").arg(artboard);
    }
    if let Some(state_machine) = &entry.state_machine {
        command.arg("--state-machine").arg(state_machine);
    }
    command.arg("--samples").arg(samples_csv(&entry.samples));
    if let Some(input_script) = &entry.input_script {
        command
            .arg("--input-script")
            .arg(resolve_script_path(input_script, corpus_dir));
    }
    command
}

fn samples_csv(samples: &[f32]) -> String {
    samples
        .iter()
        .map(|sample| sample.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
