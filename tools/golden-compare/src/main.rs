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
    let corpus = parse_corpus(&options.corpus)?;
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
    let mut failures = Vec::new();

    for entry in &corpus {
        *counts.entry(entry.status).or_default() += 1;
        match entry.status {
            Status::UnsupportedFeature => {
                println!(
                    "[unsupported-feature] {}: skipped ({})",
                    entry.id,
                    entry.features.join(", ")
                );
            }
            Status::NotYet | Status::Diverges | Status::Exact => {
                let file = resolve_asset_path(&entry.path, &options.rive_runtime_dir);
                match run_stream(&options.cpp_runner, entry, &file, &corpus_dir) {
                    Ok(cpp_stream) => {
                        println!(
                            "[{}] {}: c++ stream ok ({} bytes)",
                            entry.status,
                            entry.id,
                            cpp_stream.len()
                        );
                        if entry.status == Status::Exact {
                            match &options.rust_runner {
                                Some(rust_runner) => {
                                    let rust_stream =
                                        run_stream(rust_runner, entry, &file, &corpus_dir)?;
                                    if rust_stream != cpp_stream {
                                        failures.push(format!(
                                            "{}: exact stream differs from C++",
                                            entry.id
                                        ));
                                    }
                                }
                                None => failures.push(format!(
                                    "{}: status is exact but --rust-runner was not supplied",
                                    entry.id
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
        "golden-compare summary: entries={} exact={} diverges={} unsupported-feature={} not-yet={}",
        corpus.len(),
        exact,
        counts.get(&Status::Diverges).copied().unwrap_or(0),
        counts
            .get(&Status::UnsupportedFeature)
            .copied()
            .unwrap_or(0),
        counts.get(&Status::NotYet).copied().unwrap_or(0),
    );

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
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut corpus = PathBuf::from("corpus.toml");
        let mut cpp_runner = env::var_os("GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(default_cpp_runner);
        let mut rust_runner = None;
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
                "--help" | "-h" => {
                    println!(
                        "usage: golden-compare [--corpus corpus.toml] --cpp-runner <path> [--rust-runner <path>]"
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
    samples: Vec<f32>,
    status: Status,
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
            samples: vec![0.0],
            status: Status::NotYet,
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
        Ok(())
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
            "samples" => entry.samples = parse_float_array(value, line_number)?,
            "status" => entry.status = Status::parse(&parse_string(value, line_number)?)?,
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

fn run_stream(
    runner: &Path,
    entry: &CorpusEntry,
    file: &Path,
    corpus_dir: &Path,
) -> Result<String, String> {
    let mut command = Command::new(runner);
    command.arg("--file").arg(file);
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
    if !stdout.starts_with("rive-golden-stream-v1\n") {
        return Err(format!(
            "{} did not emit a rive-golden stream",
            runner.display()
        ));
    }
    Ok(stdout)
}

fn samples_csv(samples: &[f32]) -> String {
    samples
        .iter()
        .map(|sample| sample.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
