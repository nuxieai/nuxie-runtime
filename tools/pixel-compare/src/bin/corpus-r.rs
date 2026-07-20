use pixel_compare::{
    artifact, compare, validate_reference_identities, DiffReport, ReferenceIdentity, RgbaImage,
    Tolerance,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::{atomic::AtomicBool, atomic::Ordering, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_REPLAY_TIMEOUT_SECONDS: u64 = 60;

#[derive(Debug, Deserialize)]
struct Manifest {
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    stream: PathBuf,
    reference: PathBuf,
    status: String,
    #[serde(default)]
    frame: usize,
    max_channel_delta: u8,
    max_different_pixels: u64,
    gated: Option<String>,
    mode: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse()?;
    let dynamic_run_provenance = DynamicRunProvenance::load(&options)?;
    let manifest: Manifest = toml::from_str(&fs::read_to_string(&options.manifest)?)?;
    let reference_base = std::env::current_dir()?;
    validate_entry_ids(&manifest.entry)?;
    if options.dynamic_reference().is_none() {
        validate_reference_identity(&reference_base, &manifest.entry)?;
    }
    let entries = selected_entries(&manifest.entry, &options.probe_gated)?;
    fs::create_dir_all(&options.output_dir)?;
    let mut counts = Counts::default();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    if let Some(reference) = options.dynamic_reference() {
        writeln!(
            stdout,
            "renderer-corpus-oracle reference-replay={} reference-backend={} candidate-replay={} candidate-backend={}",
            reference.replay.display(),
            reference.backend,
            options.replay.display(),
            options.backend,
        )?;
    }
    run_bounded(
        &entries,
        options.jobs,
        |entry| {
            run_entry(
                entry,
                &options,
                &reference_base,
                dynamic_run_provenance.as_ref(),
            )
        },
        |execution| execution.outcome.is_err(),
        |index, execution| {
            emit_entry(
                entries[index],
                execution,
                !options.probe_gated.is_empty(),
                &options.output_dir,
                &mut counts,
                &mut stdout,
                &mut stderr,
            )
        },
    )?;

    if !options.probe_gated.is_empty() {
        writeln!(
            stdout,
            "renderer-corpus-probe passes={} byte-exact={} diverges={} total={}",
            counts.exact,
            counts.byte_exact,
            counts.diverges,
            entries.len()
        )?;
        return Ok(());
    }
    writeln!(
        stdout,
        "renderer-corpus exact={} byte-exact={} diverges={} gated={} total={}",
        counts.exact,
        counts.byte_exact,
        counts.diverges,
        counts.gated,
        entries.len()
    )?;
    if options.expect_all_fail {
        validate_stub_baseline(&counts)?;
    } else if manifest
        .entry
        .iter()
        .filter(|entry| entry.status == "exact")
        .count()
        != counts.exact
        || counts.diverges != 0
    {
        return Err("renderer corpus ratchet failed".into());
    }
    Ok(())
}

#[derive(Debug)]
enum EntryOutcome {
    Gated {
        diagnostic: String,
    },
    Compared {
        report: DiffReport,
        byte_exact: bool,
        reference_is_transparent_blank: bool,
        adapter_check: Option<AdapterCheck>,
    },
}

#[derive(Clone, Copy, Debug)]
enum AdapterCheck {
    Matched,
    CandidateUnreported,
    ReferenceUnreported,
    Unreported,
}

impl AdapterCheck {
    fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::CandidateUnreported => "candidate-unreported",
            Self::ReferenceUnreported => "reference-unreported",
            Self::Unreported => "unreported",
        }
    }
}

#[derive(Debug, Default)]
struct ChildDiagnostics {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Debug)]
struct ReplayError {
    message: String,
    diagnostics: ChildDiagnostics,
}

#[derive(Debug)]
struct EntryExecution {
    diagnostics: ChildDiagnostics,
    outcome: Result<EntryOutcome, String>,
}

#[derive(Default)]
struct Counts {
    exact: usize,
    byte_exact: usize,
    transparent_blank_byte_exact: usize,
    diverges: usize,
    gated: usize,
}

fn validate_stub_baseline(counts: &Counts) -> Result<(), String> {
    let nonblank_byte_exact = counts
        .byte_exact
        .checked_sub(counts.transparent_blank_byte_exact)
        .ok_or_else(|| "stub baseline byte-exact counts are inconsistent".to_owned())?;
    if nonblank_byte_exact != 0 {
        return Err(format!(
            "stub baseline unexpectedly byte-matched {nonblank_byte_exact} nonblank references"
        ));
    }

    if counts.diverges == 0 {
        return Err("stub baseline has no tolerance-divergent references".to_owned());
    }
    Ok(())
}

fn run_entry(
    entry: &Entry,
    options: &Options,
    reference_base: &Path,
    dynamic_run_provenance: Option<&DynamicRunProvenance>,
) -> EntryExecution {
    if entry.status == "gated" && options.probe_gated.is_empty() {
        return EntryExecution {
            diagnostics: ChildDiagnostics::default(),
            outcome: Ok(EntryOutcome::Gated {
                diagnostic: entry.gated.as_deref().unwrap_or("no diagnostic").to_owned(),
            }),
        };
    }
    let dynamic_reference = options.dynamic_reference();
    let reference = dynamic_reference.map_or_else(
        || resolve_reference(reference_base, entry),
        |_| {
            Ok(options
                .output_dir
                .join(format!("{}-reference.png", entry.id)))
        },
    );
    let reference = match reference {
        Ok(reference) => reference,
        Err(error) => return EntryExecution::failed(error),
    };
    let actual = options.output_dir.join(format!("{}.png", entry.id));
    let mut diagnostics = ChildDiagnostics::default();
    let reference_output = if let Some(reference_replay) = dynamic_reference {
        let output = match run_replay(reference_replay, entry, &reference, options.replay_timeout) {
            Ok(output) => output,
            Err(error) => return EntryExecution::replay_failed(error),
        };
        diagnostics.append(&output);
        if !output.status.success() {
            return EntryExecution {
                diagnostics,
                outcome: Err(format!("reference renderer replay failed for {}", entry.id)),
            };
        }
        Some(output)
    } else {
        None
    };
    let candidate_replay = ReplayInvocation {
        replay: &options.replay,
        backend: &options.backend,
    };
    let output = match run_replay(candidate_replay, entry, &actual, options.replay_timeout) {
        Ok(output) => output,
        Err(error) => {
            diagnostics.merge(error.diagnostics);
            return EntryExecution {
                diagnostics,
                outcome: Err(error.message),
            };
        }
    };
    diagnostics.append(&output);
    let adapter_check = match (dynamic_reference, reference_output.as_ref()) {
        (Some(reference_replay), Some(reference_output)) if output.status.success() => {
            match prepare_dynamic_reference_report(DynamicComparison {
                entry,
                output_dir: &options.output_dir,
                reference_replay,
                candidate_replay,
                reference_png: &reference,
                candidate_png: &actual,
                reference_output,
                candidate_output: &output,
                run_provenance: dynamic_run_provenance
                    .expect("dynamic replay provenance must be loaded"),
            }) {
                Ok(adapter_check) => Some(adapter_check),
                Err(error) => {
                    return EntryExecution {
                        diagnostics,
                        outcome: Err(error),
                    };
                }
            }
        }
        _ => None,
    };
    let outcome = compare_entry(
        entry,
        &reference,
        &actual,
        &options.output_dir,
        output.status.success(),
        adapter_check,
    );
    EntryExecution {
        diagnostics,
        outcome,
    }
}

#[derive(Clone, Copy)]
struct ReplayInvocation<'a> {
    replay: &'a Path,
    backend: &'a str,
}

fn run_replay(
    invocation: ReplayInvocation<'_>,
    entry: &Entry,
    output: &Path,
    timeout: Duration,
) -> Result<Output, ReplayError> {
    let stream = path_str(&entry.stream)
        .map_err(|error| ReplayError::without_diagnostics(error.to_string()))?;
    let output =
        path_str(output).map_err(|error| ReplayError::without_diagnostics(error.to_string()))?;
    let mut child = Command::new(invocation.replay)
        .args(["--stream", stream])
        .args(["--output", output])
        .args(["--backend", invocation.backend])
        .args(["--frame", &entry.frame.to_string()])
        .args(["--mode", &entry.mode])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            ReplayError::without_diagnostics(format!(
                "failed to launch renderer replay `{}` for entry `{}` with backend `{}`: {error}",
                invocation.replay.display(),
                entry.id,
                invocation.backend
            ))
        })?;
    let stdout = child
        .stdout
        .take()
        .expect("piped renderer stdout must be available");
    let stderr = child
        .stderr
        .take()
        .expect("piped renderer stderr must be available");
    // Drain both pipes while the replay runs. Waiting first can deadlock when a
    // verbose child fills either OS pipe buffer before it exits.
    let stdout_reader = thread::spawn(move || read_to_end(stdout));
    let stderr_reader = thread::spawn(move || read_to_end(stderr));

    let wait_result = wait_with_timeout(&mut child, timeout);
    let lifecycle_error = match &wait_result {
        Ok(Some(_)) => None,
        Ok(None) => terminate_and_reap(&mut child).err(),
        Err(_) => terminate_and_reap(&mut child).err(),
    };
    let (diagnostics, capture_error) = collect_child_diagnostics(stdout_reader, stderr_reader);

    match wait_result {
        Ok(Some(status)) => {
            if let Some(error) = capture_error {
                return Err(ReplayError {
                    message: format!(
                        "failed to collect renderer replay output for entry `{}` with backend `{}`: {error}",
                        entry.id, invocation.backend
                    ),
                    diagnostics,
                });
            }
            Ok(Output {
                status,
                stdout: diagnostics.stdout,
                stderr: diagnostics.stderr,
            })
        }
        Ok(None) => Err(ReplayError {
            message: format!(
                "renderer replay timed out after {} for entry `{}` with backend `{}`{}",
                display_duration(timeout),
                entry.id,
                invocation.backend,
                failure_details(lifecycle_error, capture_error)
            ),
            diagnostics,
        }),
        Err(error) => Err(ReplayError {
            message: format!(
                "failed while waiting for renderer replay for entry `{}` with backend `{}`: {error}{}",
                entry.id,
                invocation.backend,
                failure_details(lifecycle_error, capture_error)
            ),
            diagnostics,
        }),
    }
}

fn read_to_end(mut stream: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn collect_child_diagnostics(
    stdout_reader: thread::JoinHandle<io::Result<Vec<u8>>>,
    stderr_reader: thread::JoinHandle<io::Result<Vec<u8>>>,
) -> (ChildDiagnostics, Option<String>) {
    let stdout = join_reader(stdout_reader, "stdout");
    let stderr = join_reader(stderr_reader, "stderr");
    let mut errors = Vec::new();
    let stdout = stdout.unwrap_or_else(|error| {
        errors.push(error);
        Vec::new()
    });
    let stderr = stderr.unwrap_or_else(|error| {
        errors.push(error);
        Vec::new()
    });
    (
        ChildDiagnostics { stdout, stderr },
        (!errors.is_empty()).then(|| errors.join("; ")),
    )
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<Vec<u8>>>,
    stream_name: &str,
) -> Result<Vec<u8>, String> {
    reader
        .join()
        .map_err(|_| format!("{stream_name} reader panicked"))?
        .map_err(|error| format!("could not read {stream_name}: {error}"))
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> io::Result<Option<ExitStatus>> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if started.elapsed() >= timeout {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_and_reap(child: &mut Child) -> Result<(), String> {
    let kill_error = child.kill().err();
    let wait_error = child.wait().err();
    match (kill_error, wait_error) {
        (_, None) => Ok(()),
        (None, Some(wait_error)) => Err(format!("failed to reap child: {wait_error}")),
        (Some(kill_error), Some(wait_error)) => Err(format!(
            "failed to kill child: {kill_error}; failed to reap child: {wait_error}"
        )),
    }
}

fn display_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{seconds} second{}", if seconds == 1 { "" } else { "s" })
}

fn failure_details(lifecycle_error: Option<String>, capture_error: Option<String>) -> String {
    let mut details = Vec::new();
    if let Some(error) = lifecycle_error {
        details.push(format!("cleanup failed: {error}"));
    }
    if let Some(error) = capture_error {
        details.push(format!("output capture failed: {error}"));
    }
    if details.is_empty() {
        String::new()
    } else {
        format!("; {}", details.join("; "))
    }
}

impl ChildDiagnostics {
    fn append(&mut self, output: &Output) {
        self.stdout.extend_from_slice(&output.stdout);
        self.stderr.extend_from_slice(&output.stderr);
    }

    fn merge(&mut self, other: Self) {
        self.stdout.extend(other.stdout);
        self.stderr.extend(other.stderr);
    }
}

impl ReplayError {
    fn without_diagnostics(message: String) -> Self {
        Self {
            message,
            diagnostics: ChildDiagnostics::default(),
        }
    }
}

#[derive(Serialize)]
struct DynamicProvenance<'a> {
    provenance_schema: u8,
    oracle: &'static str,
    case_id: &'a str,
    stream: String,
    stream_sha256: String,
    frame: usize,
    mode: &'a str,
    max_channel_delta: u8,
    max_different_pixels: u64,
    reference_replay: String,
    reference_replay_sha256: &'a str,
    reference_backend: &'a str,
    reference_output: String,
    reference_png_sha256: String,
    reference_adapter: &'a str,
    candidate_replay: String,
    candidate_replay_sha256: &'a str,
    candidate_backend: &'a str,
    candidate_output: String,
    candidate_png_sha256: String,
    candidate_adapter: &'a str,
    adapter_check: &'static str,
}

struct DynamicComparison<'a> {
    entry: &'a Entry,
    output_dir: &'a Path,
    reference_replay: ReplayInvocation<'a>,
    candidate_replay: ReplayInvocation<'a>,
    reference_png: &'a Path,
    candidate_png: &'a Path,
    reference_output: &'a std::process::Output,
    candidate_output: &'a std::process::Output,
    run_provenance: &'a DynamicRunProvenance,
}

fn prepare_dynamic_reference_report(
    comparison: DynamicComparison<'_>,
) -> Result<AdapterCheck, String> {
    let DynamicComparison {
        entry,
        output_dir,
        reference_replay,
        candidate_replay,
        reference_png,
        candidate_png,
        reference_output,
        candidate_output,
        run_provenance,
    } = comparison;
    let reference_adapter = reported_adapter(&reference_output.stdout)?;
    let candidate_adapter = reported_adapter(&candidate_output.stdout)?;
    let (adapter_check, adapter_error) = match (&reference_adapter, &candidate_adapter) {
        (Some(reference), Some(candidate)) if reference == candidate => {
            (Some(AdapterCheck::Matched), None)
        }
        (Some(reference), Some(candidate)) => (
            None,
            Some(format!(
                "renderer adapter mismatch for {}: reference `{reference}`, candidate `{candidate}`",
                entry.id
            )),
        ),
        (Some(_), None) => (
            Some(AdapterCheck::CandidateUnreported),
            Some(format!(
                "renderer adapter identity missing for {}: candidate did not report `adapter=`",
                entry.id
            )),
        ),
        (None, Some(_)) => (
            Some(AdapterCheck::ReferenceUnreported),
            Some(format!(
                "renderer adapter identity missing for {}: reference did not report `adapter=`",
                entry.id
            )),
        ),
        (None, None) => (
            Some(AdapterCheck::Unreported),
            Some(format!(
                "renderer adapter identity missing for {}: neither replay reported `adapter=`",
                entry.id
            )),
        ),
    };
    let provenance = output_dir.join(format!("{}.provenance.toml", entry.id));
    let record = DynamicProvenance {
        provenance_schema: 1,
        oracle: "same-runner-replay",
        case_id: &entry.id,
        stream: entry.stream.display().to_string(),
        stream_sha256: sha256_file(&entry.stream)?,
        frame: entry.frame,
        mode: &entry.mode,
        max_channel_delta: entry.max_channel_delta,
        max_different_pixels: entry.max_different_pixels,
        reference_replay: reference_replay.replay.display().to_string(),
        reference_replay_sha256: &run_provenance.reference_replay_sha256,
        reference_backend: reference_replay.backend,
        reference_output: reference_png.display().to_string(),
        reference_png_sha256: sha256_file(reference_png)?,
        reference_adapter: reference_adapter.as_deref().unwrap_or("unreported"),
        candidate_replay: candidate_replay.replay.display().to_string(),
        candidate_replay_sha256: &run_provenance.candidate_replay_sha256,
        candidate_backend: candidate_replay.backend,
        candidate_output: candidate_png.display().to_string(),
        candidate_png_sha256: sha256_file(candidate_png)?,
        candidate_adapter: candidate_adapter.as_deref().unwrap_or("unreported"),
        adapter_check: adapter_check.map_or("mismatch", AdapterCheck::as_str),
    };
    fs::write(
        &provenance,
        toml::to_string_pretty(&record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if let Some(error) = adapter_error {
        return Err(error);
    }
    let adapter_check = adapter_check.expect("adapter reports retain a provenance status");
    debug_assert!(matches!(adapter_check, AdapterCheck::Matched));
    Ok(adapter_check)
}

struct DynamicRunProvenance {
    reference_replay_sha256: String,
    candidate_replay_sha256: String,
}

impl DynamicRunProvenance {
    fn load(options: &Options) -> Result<Option<Self>, String> {
        let Some(reference) = options.dynamic_reference() else {
            return Ok(None);
        };
        let candidate_replay_sha256 = sha256_file(&options.replay)?;
        let reference_replay_sha256 = if reference.replay == options.replay {
            candidate_replay_sha256.clone()
        } else {
            sha256_file(reference.replay)?
        };
        Ok(Some(Self {
            reference_replay_sha256,
            candidate_replay_sha256,
        }))
    }
}

fn sha256_file(path: &Path) -> Result<String, String> {
    fs::read(path)
        .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
        .map_err(|error| format!("failed to hash {}: {error}", path.display()))
}

fn reported_adapter(stdout: &[u8]) -> Result<Option<String>, String> {
    let stdout = String::from_utf8_lossy(stdout);
    let mut adapters = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("adapter="));
    let adapter = adapters
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if adapters.next().is_some() {
        return Err("renderer replay reported more than one `adapter=` line".to_owned());
    }
    Ok(adapter.map(str::to_owned))
}

impl EntryExecution {
    fn failed(error: String) -> Self {
        Self {
            diagnostics: ChildDiagnostics::default(),
            outcome: Err(error),
        }
    }

    fn replay_failed(error: ReplayError) -> Self {
        Self {
            diagnostics: error.diagnostics,
            outcome: Err(error.message),
        }
    }
}

fn compare_entry(
    entry: &Entry,
    reference: &Path,
    actual: &Path,
    output_dir: &Path,
    replay_succeeded: bool,
    adapter_check: Option<AdapterCheck>,
) -> Result<EntryOutcome, String> {
    if !replay_succeeded {
        return Err(format!("renderer replay failed for {}", entry.id));
    }
    let expected = RgbaImage::read_png(reference).map_err(|error| error.to_string())?;
    let actual_image = RgbaImage::read_png(actual).map_err(|error| error.to_string())?;
    let report = compare(
        &expected,
        &actual_image,
        Tolerance {
            max_channel_delta: entry.max_channel_delta,
            max_different_pixels: entry.max_different_pixels,
        },
    )
    .map_err(|error| error.to_string())?;
    if !report.within_tolerance {
        let artifact_path = output_dir.join(format!("{}-diff.png", entry.id));
        artifact(&expected, &actual_image)
            .and_then(|artifact| artifact.write_png(&artifact_path))
            .map_err(|error| error.to_string())?;
    }
    Ok(EntryOutcome::Compared {
        report,
        byte_exact: expected == actual_image,
        reference_is_transparent_blank: expected.pixels.iter().all(|channel| *channel == 0),
        adapter_check,
    })
}

fn resolve_reference(base: &Path, entry: &Entry) -> Result<PathBuf, String> {
    let physical = if entry.reference.is_absolute() {
        entry.reference.clone()
    } else {
        base.join(&entry.reference)
    };
    if physical.is_file() {
        Ok(physical)
    } else {
        Err(format!(
            "renderer reference `{}` is missing and has no existing approved target",
            entry.reference.display()
        ))
    }
}

fn emit_child_diagnostics(
    diagnostics: &ChildDiagnostics,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<()> {
    stdout.write_all(&diagnostics.stdout)?;
    stdout.flush()?;
    stderr.write_all(&diagnostics.stderr)?;
    stderr.flush()
}

fn emit_entry(
    entry: &Entry,
    execution: EntryExecution,
    probing: bool,
    output_dir: &Path,
    counts: &mut Counts,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<(), String> {
    emit_child_diagnostics(&execution.diagnostics, stdout, stderr)
        .map_err(|error| error.to_string())?;
    match execution.outcome? {
        EntryOutcome::Gated { diagnostic } => {
            counts.gated += 1;
            writeln!(stdout, "gated {}: {diagnostic}", entry.id)
                .map_err(|error| error.to_string())?;
        }
        EntryOutcome::Compared {
            report,
            byte_exact,
            reference_is_transparent_blank,
            adapter_check,
        } if report.within_tolerance => {
            counts.exact += 1;
            counts.byte_exact += usize::from(byte_exact);
            counts.transparent_blank_byte_exact +=
                usize::from(byte_exact && reference_is_transparent_blank);
            writeln!(
                stdout,
                "{} {}: byte-exact={} different-pixels={} max-channel-delta={}",
                if probing { "probe-pass" } else { "exact" },
                entry.id,
                byte_exact,
                report.different_pixels,
                report.max_channel_delta
            )
            .map_err(|error| error.to_string())?;
            emit_dynamic_reference(entry, output_dir, adapter_check, stdout)?;
        }
        EntryOutcome::Compared {
            report,
            adapter_check,
            ..
        } => {
            counts.diverges += 1;
            let artifact_path = output_dir.join(format!("{}-diff.png", entry.id));
            writeln!(
                stdout,
                "{} {}: different-pixels={} max-channel-delta={} artifact={}",
                if probing {
                    "probe-diverges"
                } else {
                    "diverges"
                },
                entry.id,
                report.different_pixels,
                report.max_channel_delta,
                artifact_path.display()
            )
            .map_err(|error| error.to_string())?;
            emit_dynamic_reference(entry, output_dir, adapter_check, stdout)?;
        }
    }
    stdout.flush().map_err(|error| error.to_string())
}

fn emit_dynamic_reference(
    entry: &Entry,
    output_dir: &Path,
    adapter_check: Option<AdapterCheck>,
    stdout: &mut impl Write,
) -> Result<(), String> {
    let Some(adapter_check) = adapter_check else {
        return Ok(());
    };
    let reference = output_dir.join(format!("{}-reference.png", entry.id));
    let provenance = output_dir.join(format!("{}.provenance.toml", entry.id));
    writeln!(
        stdout,
        "same-runner-reference reference={} provenance={} adapter-check={}",
        reference.display(),
        provenance.display(),
        adapter_check.as_str(),
    )
    .map_err(|error| error.to_string())
}

fn run_bounded<T, R, F, P, G, E>(
    entries: &[T],
    jobs: usize,
    run: F,
    is_failure: P,
    mut emit: G,
) -> Result<(), E>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
    P: Fn(&R) -> bool + Sync,
    G: FnMut(usize, R) -> Result<(), E>,
{
    assert!(jobs > 0, "jobs must be positive");
    if entries.is_empty() {
        return Ok(());
    }

    enum WorkerMessage<R> {
        Completed {
            worker: usize,
            index: usize,
            result: R,
        },
        Panicked {
            index: usize,
        },
    }

    let worker_count = jobs.min(entries.len());
    let stop = AtomicBool::new(false);
    let (terminal_error, terminal_panic) = thread::scope(|scope| {
        let (result_tx, result_rx) = mpsc::channel::<WorkerMessage<R>>();
        let mut task_senders = Vec::with_capacity(worker_count);
        for worker in 0..worker_count {
            let (task_tx, task_rx) = mpsc::channel::<Option<usize>>();
            task_senders.push(task_tx);
            let result_tx = result_tx.clone();
            let stop = &stop;
            let run = &run;
            let is_failure = &is_failure;
            scope.spawn(move || {
                while let Ok(Some(index)) = task_rx.recv() {
                    match catch_unwind(AssertUnwindSafe(|| {
                        let result = run(&entries[index]);
                        let failed = is_failure(&result);
                        (result, failed)
                    })) {
                        Ok((result, failed)) => {
                            if failed {
                                stop.store(true, Ordering::Release);
                            }
                            if result_tx
                                .send(WorkerMessage::Completed {
                                    worker,
                                    index,
                                    result,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(_) => {
                            stop.store(true, Ordering::Release);
                            let _ = result_tx.send(WorkerMessage::Panicked { index });
                            break;
                        }
                    }
                }
            });
        }
        drop(result_tx);

        let mut idle_workers = (0..worker_count).rev().collect::<Vec<_>>();
        let mut in_flight = 0usize;
        let mut next_to_schedule = 0usize;
        let mut next_to_emit = 0usize;
        let mut pending = BTreeMap::new();
        let mut terminal_error = None;
        let mut terminal_panic = None;

        let dispatch = |idle_workers: &mut Vec<usize>,
                        in_flight: &mut usize,
                        next_to_schedule: &mut usize,
                        pending_len: usize| {
            while !stop.load(Ordering::Acquire)
                && *next_to_schedule < entries.len()
                && *in_flight + pending_len < jobs
                && !idle_workers.is_empty()
            {
                let worker = idle_workers.pop().unwrap();
                task_senders[worker].send(Some(*next_to_schedule)).unwrap();
                *next_to_schedule += 1;
                *in_flight += 1;
            }
        };

        dispatch(
            &mut idle_workers,
            &mut in_flight,
            &mut next_to_schedule,
            pending.len(),
        );
        while in_flight != 0 {
            let message = result_rx
                .recv()
                .expect("every scheduled worker must report completion or panic");
            let (worker, index, result) = match message {
                WorkerMessage::Completed {
                    worker,
                    index,
                    result,
                } => (worker, index, result),
                WorkerMessage::Panicked { index } => {
                    in_flight -= 1;
                    stop.store(true, Ordering::Release);
                    terminal_panic.get_or_insert(index);
                    pending.clear();
                    continue;
                }
            };
            in_flight -= 1;
            idle_workers.push(worker);

            if terminal_error.is_none() {
                pending.insert(index, result);
                while let Some(result) = pending.remove(&next_to_emit) {
                    let index = next_to_emit;
                    next_to_emit += 1;
                    if let Err(error) = emit(index, result) {
                        stop.store(true, Ordering::Release);
                        terminal_error = Some(error);
                        pending.clear();
                        break;
                    }
                }
            }

            if terminal_error.is_none() {
                dispatch(
                    &mut idle_workers,
                    &mut in_flight,
                    &mut next_to_schedule,
                    pending.len(),
                );
            }
        }

        for sender in task_senders {
            let _ = sender.send(None);
        }
        (terminal_error, terminal_panic)
    });

    if let Some(index) = terminal_panic {
        panic!("bounded worker panicked while processing entry {index}");
    }
    terminal_error.map_or(Ok(()), Err)
}

fn selected_entries<'a>(
    entries: &'a [Entry],
    probe_gated: &[String],
) -> Result<Vec<&'a Entry>, String> {
    if probe_gated.is_empty() {
        return Ok(entries.iter().collect());
    }
    let mut selected = Vec::with_capacity(probe_gated.len());
    for id in probe_gated {
        if selected.iter().any(|entry: &&Entry| entry.id == *id) {
            return Err(format!("duplicate --probe-gated id `{id}`"));
        }
        let entry = entries
            .iter()
            .find(|entry| entry.id == *id)
            .ok_or_else(|| format!("no manifest entry has id `{id}`"))?;
        if entry.status != "gated" {
            return Err(format!(
                "--probe-gated entry `{id}` has status `{}`",
                entry.status
            ));
        }
        selected.push(entry);
    }
    Ok(selected)
}

fn validate_entry_ids(entries: &[Entry]) -> Result<(), String> {
    let mut ids = HashMap::with_capacity(entries.len());
    let mut output_names = HashSet::with_capacity(entries.len() * 2);
    for entry in entries {
        if entry.id.is_empty()
            || !entry
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return Err(format!("unsafe manifest entry id `{}`", entry.id));
        }
        if let Some(previous) = ids.insert(entry.id.to_lowercase(), entry.id.as_str()) {
            if previous == entry.id {
                return Err(format!("duplicate manifest entry id `{}`", entry.id));
            }
            return Err(format!(
                "manifest entry ids `{previous}` and `{}` collide case-insensitively",
                entry.id
            ));
        }
        for output_name in [
            format!("{}.png", entry.id),
            format!("{}-diff.png", entry.id),
            format!("{}-reference.png", entry.id),
            format!("{}.provenance.toml", entry.id),
        ] {
            if !output_names.insert(output_name.to_lowercase()) {
                return Err(format!(
                    "manifest entry id `{}` collides in the renderer output namespace",
                    entry.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_reference_identity(base: &Path, entries: &[Entry]) -> Result<(), String> {
    let mut owners = HashMap::<PathBuf, (&str, &Path)>::new();
    for entry in entries {
        let physical = if entry.reference.is_absolute() {
            entry.reference.clone()
        } else {
            base.join(&entry.reference)
        };
        let physical = physical.canonicalize().unwrap_or(physical);
        if let Some((previous_id, previous_reference)) =
            owners.insert(physical.clone(), (&entry.id, &entry.reference))
        {
            if previous_reference != entry.reference {
                return Err(format!(
                    "manifest entries {previous_id} ({}) and {} ({}) resolve to the same physical reference {}; reference paths must remain unique",
                    previous_reference.display(),
                    entry.id,
                    entry.reference.display(),
                    physical.display(),
                ));
            }
        }
    }

    validate_reference_identities(
        base,
        entries.iter().map(|entry| ReferenceIdentity {
            id: &entry.id,
            stream: &entry.stream,
            frame: entry.frame,
            mode: &entry.mode,
            reference: &entry.reference,
        }),
    )
}

struct Options {
    manifest: PathBuf,
    replay: PathBuf,
    backend: String,
    output_dir: PathBuf,
    jobs: usize,
    expect_all_fail: bool,
    probe_gated: Vec<String>,
    reference_replay: Option<PathBuf>,
    reference_backend: Option<String>,
    replay_timeout: Duration,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn Error>> {
        Self::parse_args(std::env::args().skip(1))
    }

    fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let mut manifest = PathBuf::from("corpus-r.toml");
        let mut replay = PathBuf::from("target/debug/renderer-replay");
        let mut backend = "rust-wgpu".to_owned();
        let mut output_dir = PathBuf::from("target/renderer-corpus");
        let mut jobs = 1;
        let mut expect_all_fail = false;
        let mut probe_gated = Vec::new();
        let mut reference_replay = None;
        let mut reference_backend = None;
        let mut replay_timeout = Duration::from_secs(DEFAULT_REPLAY_TIMEOUT_SECONDS);
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => manifest = PathBuf::from(args.next().ok_or(usage())?),
                "--replay" => replay = PathBuf::from(args.next().ok_or(usage())?),
                "--backend" => backend = args.next().ok_or(usage())?,
                "--output-dir" => output_dir = PathBuf::from(args.next().ok_or(usage())?),
                "--jobs" => {
                    let value = args.next().ok_or(usage())?;
                    jobs = value
                        .parse()
                        .map_err(|_| format!("--jobs must be a positive integer, got `{value}`"))?;
                    if jobs == 0 {
                        return Err("--jobs must be a positive integer, got `0`".into());
                    }
                }
                "--expect-all-fail" => expect_all_fail = true,
                "--probe-gated" => probe_gated.push(args.next().ok_or(usage())?),
                "--reference-replay" => {
                    reference_replay = Some(PathBuf::from(args.next().ok_or(usage())?))
                }
                "--reference-backend" => reference_backend = Some(args.next().ok_or(usage())?),
                "--replay-timeout-seconds" => {
                    let value = args.next().ok_or(usage())?;
                    let seconds = value.parse::<u64>().map_err(|_| {
                        format!(
                            "--replay-timeout-seconds must be a positive integer, got `{value}`"
                        )
                    })?;
                    if seconds == 0 {
                        return Err(format!(
                            "--replay-timeout-seconds must be a positive integer, got `{value}`"
                        )
                        .into());
                    }
                    replay_timeout = Duration::from_secs(seconds);
                }
                _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
            }
        }
        if reference_replay.is_some() != reference_backend.is_some() {
            return Err(
                "--reference-replay and --reference-backend must be provided together".into(),
            );
        }
        if reference_backend
            .as_deref()
            .is_some_and(|backend| backend != "ffi-dawn")
        {
            return Err("--reference-backend must be `ffi-dawn`".into());
        }
        Ok(Self {
            manifest,
            replay,
            backend,
            output_dir,
            jobs,
            expect_all_fail,
            probe_gated,
            reference_replay,
            reference_backend,
            replay_timeout,
        })
    }

    fn dynamic_reference(&self) -> Option<ReplayInvocation<'_>> {
        Some(ReplayInvocation {
            replay: self.reference_replay.as_deref()?,
            backend: self.reference_backend.as_deref()?,
        })
    }
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error + Send + Sync>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn usage() -> &'static str {
    "usage: corpus-r [--manifest FILE] [--replay FILE] [--backend stub|rust-wgpu|ffi-metal|ffi-dawn] [--reference-replay FILE --reference-backend BACKEND] [--output-dir DIR] [--jobs N] [--replay-timeout-seconds N] [--expect-all-fail] [--probe-gated ID ...]"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, Barrier, Condvar, Mutex};
    use std::time::Duration;

    fn entry(id: &str, mode: &str, reference: &str) -> Entry {
        Entry {
            id: id.to_owned(),
            stream: PathBuf::from("fixture.rive-stream"),
            reference: PathBuf::from(reference),
            status: "gated".to_owned(),
            frame: 0,
            max_channel_delta: 2,
            max_different_pixels: 0,
            gated: Some("algorithm-core".to_owned()),
            mode: mode.to_owned(),
        }
    }

    #[test]
    fn rejects_reference_reuse_across_modes() {
        let entries = [
            entry("atomic", "clockwise-atomic", "shared.png"),
            entry("msaa", "msaa", "shared.png"),
        ];
        let error = validate_reference_identity(Path::new("/repo"), &entries).unwrap_err();
        assert!(error.contains("keyed by stream, frame, and mode"));
    }

    #[test]
    fn accepts_mode_specific_reference_paths() {
        let entries = [
            entry("atomic", "clockwise-atomic", "atomic.png"),
            entry("msaa", "msaa", "msaa.png"),
        ];
        validate_reference_identity(Path::new("/repo"), &entries).unwrap();
    }

    #[test]
    fn rejects_an_unapproved_missing_clockwise_atomic_reference() {
        static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "corpus-r-reference-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        let source = "fixtures/renderer/reference/metal/gm/unapproved-clockwise-atomic.png";
        let broad_fallback = root.join("fixtures/renderer/reference/metal/gm/unapproved.png");
        fs::create_dir_all(broad_fallback.parent().unwrap()).unwrap();
        fs::write(&broad_fallback, b"not an approved oracle").unwrap();

        let entry = entry("unapproved", "clockwise-atomic", source);
        let error = resolve_reference(&root, &entry).unwrap_err();
        assert!(error.contains("missing and has no existing approved target"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_lexical_aliases_of_the_same_reference() {
        let entries = [
            entry("atomic", "clockwise-atomic", "alias/shared.png"),
            entry("msaa", "msaa", "alias/sub/../shared.png"),
        ];
        validate_reference_identity(Path::new("/repo"), &entries).unwrap_err();
    }

    #[test]
    fn rejects_absolute_and_relative_aliases() {
        let entries = [
            entry("one", "clockwise-atomic", "fixtures/shared.png"),
            entry("two", "msaa", "/repo/fixtures/shared.png"),
        ];
        validate_reference_identity(Path::new("/repo"), &entries).unwrap_err();
    }

    #[test]
    fn gated_probe_selection_is_explicit_and_fail_closed() {
        let entries = [
            entry("first", "clockwise-atomic", "first.png"),
            entry("second", "clockwise-atomic", "second.png"),
        ];
        let selected = selected_entries(&entries, &["second".to_owned()]).unwrap();
        assert_eq!(
            selected
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            ["second"]
        );
        assert!(selected_entries(&entries, &["missing".to_owned()]).is_err());
        assert!(selected_entries(&entries, &["first".to_owned(), "first".to_owned()]).is_err());

        let mut exact = entry("exact", "clockwise-atomic", "exact.png");
        exact.status = "exact".to_owned();
        assert!(selected_entries(&[exact], &["exact".to_owned()]).is_err());
    }

    #[test]
    fn bounded_runner_limits_concurrency_and_preserves_input_order() {
        let entries = [0, 1, 2, 3, 4, 5];
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let first_workers = Arc::new(Barrier::new(3));
        let mut results = Vec::new();
        run_bounded(
            &entries,
            3,
            {
                let active = Arc::clone(&active);
                let peak = Arc::clone(&peak);
                let first_workers = Arc::clone(&first_workers);
                move |entry| {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(current, Ordering::SeqCst);
                    if *entry < 3 {
                        first_workers.wait();
                    }
                    thread::sleep(Duration::from_millis((5 - *entry) as u64));
                    active.fetch_sub(1, Ordering::SeqCst);
                    entry * 10
                }
            },
            |_| false,
            |_, result| {
                results.push(result);
                Ok::<_, ()>(())
            },
        )
        .unwrap();

        assert_eq!(results, [0, 10, 20, 30, 40, 50]);
        assert_eq!(peak.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn bounded_runner_does_not_advance_past_its_retention_window() {
        let entries = [0, 1, 2, 3, 4, 5];
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let (started_tx, started_rx) = mpsc::channel();
        let runner = thread::spawn({
            let gate = Arc::clone(&gate);
            move || {
                run_bounded(
                    &entries,
                    3,
                    move |entry| {
                        started_tx.send(*entry).unwrap();
                        if *entry == 0 {
                            let (lock, ready) = &*gate;
                            let mut released = lock.lock().unwrap();
                            while !*released {
                                released = ready.wait(released).unwrap();
                            }
                        }
                        *entry
                    },
                    |_| false,
                    |_, _| Ok::<_, ()>(()),
                )
                .unwrap();
            }
        });

        let mut started = (0..3)
            .map(|_| started_rx.recv_timeout(Duration::from_secs(1)).unwrap())
            .collect::<Vec<_>>();
        started.sort_unstable();
        assert_eq!(started, [0, 1, 2]);
        assert!(started_rx.recv_timeout(Duration::from_millis(20)).is_err());

        let (lock, ready) = &*gate;
        *lock.lock().unwrap() = true;
        ready.notify_one();
        runner.join().unwrap();
    }

    #[test]
    fn one_job_fails_fast_with_per_entry_interleaving() {
        let entries = [0, 1, 2];
        let events = Arc::new(Mutex::new(Vec::new()));
        let error = run_bounded(
            &entries,
            1,
            {
                let events = Arc::clone(&events);
                move |entry| {
                    events.lock().unwrap().push(format!("run-{entry}"));
                    if *entry == 1 {
                        Err("replay failed")
                    } else {
                        Ok(*entry)
                    }
                }
            },
            Result::is_err,
            {
                let events = Arc::clone(&events);
                move |index, result| {
                    events.lock().unwrap().push(format!("emit-{index}"));
                    result.map(|_| ())
                }
            },
        )
        .unwrap_err();

        assert_eq!(error, "replay failed");
        assert_eq!(
            *events.lock().unwrap(),
            ["run-0", "emit-0", "run-1", "emit-1"]
        );
    }

    #[test]
    fn bounded_runner_keeps_failures_at_their_input_positions() {
        let entries = ["first", "second", "third"];
        let mut emitted = Vec::new();
        let error = run_bounded(
            &entries,
            3,
            |entry| match *entry {
                "second" => Err("replay failed"),
                entry => Ok(entry.len()),
            },
            Result::is_err,
            |index, result| {
                emitted.push(index);
                result.map(|_| ())
            },
        )
        .unwrap_err();

        assert_eq!(error, "replay failed");
        assert_eq!(emitted, [0, 1]);
    }

    #[test]
    fn parallel_runner_stops_claiming_work_after_an_error() {
        let entries = [0, 1, 2, 3];
        let first_workers = Arc::new(Barrier::new(2));
        let started = Arc::new(Mutex::new(Vec::new()));
        let error = run_bounded(
            &entries,
            2,
            {
                let first_workers = Arc::clone(&first_workers);
                let started = Arc::clone(&started);
                move |entry| {
                    started.lock().unwrap().push(*entry);
                    if *entry < 2 {
                        first_workers.wait();
                    }
                    if *entry == 0 {
                        Err("replay failed")
                    } else {
                        thread::sleep(Duration::from_millis(20));
                        Ok(*entry)
                    }
                }
            },
            Result::is_err,
            |_, result| result.map(|_| ()),
        )
        .unwrap_err();

        assert_eq!(error, "replay failed");
        let mut started = started.lock().unwrap().clone();
        started.sort_unstable();
        assert_eq!(started, [0, 1]);
    }

    #[test]
    fn child_diagnostics_are_emitted_in_input_order() {
        let entries = [0, 1, 2];
        let first_workers = Arc::new(Barrier::new(3));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        run_bounded(
            &entries,
            3,
            {
                let first_workers = Arc::clone(&first_workers);
                move |entry| {
                    first_workers.wait();
                    thread::sleep(Duration::from_millis((2 - *entry) as u64));
                    ChildDiagnostics {
                        stdout: format!("out-{entry}\n").into_bytes(),
                        stderr: format!("err-{entry}\n").into_bytes(),
                    }
                }
            },
            |_| false,
            |_, diagnostics| {
                emit_child_diagnostics(&diagnostics, &mut stdout, &mut stderr)
                    .map_err(|error| error.to_string())
            },
        )
        .unwrap();

        assert_eq!(stdout, b"out-0\nout-1\nout-2\n");
        assert_eq!(stderr, b"err-0\nerr-1\nerr-2\n");
    }

    #[test]
    fn byte_exact_is_reported_without_redefining_the_tolerance_contract() {
        let mut entry = entry("within-contract", "clockwise-atomic", "reference.png");
        entry.status = "exact".to_owned();
        let execution = EntryExecution {
            diagnostics: ChildDiagnostics::default(),
            outcome: Ok(EntryOutcome::Compared {
                report: DiffReport {
                    width: 1,
                    height: 1,
                    different_pixels: 0,
                    max_channel_delta: 1,
                    within_tolerance: true,
                },
                byte_exact: false,
                reference_is_transparent_blank: false,
                adapter_check: None,
            }),
        };
        let mut counts = Counts::default();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        emit_entry(
            &entry,
            execution,
            false,
            Path::new("output"),
            &mut counts,
            &mut stdout,
            &mut stderr,
        )
        .unwrap();

        assert_eq!(counts.exact, 1);
        assert_eq!(counts.byte_exact, 0);
        assert_eq!(counts.diverges, 0);
        assert!(String::from_utf8(stdout)
            .unwrap()
            .contains("exact within-contract: byte-exact=false"));
    }

    #[test]
    fn transparent_blank_byte_matches_are_counted_for_the_stub_guard() {
        let entry = entry("transparent-blank", "clockwise-atomic", "reference.png");
        let execution = EntryExecution {
            diagnostics: ChildDiagnostics::default(),
            outcome: Ok(EntryOutcome::Compared {
                report: DiffReport {
                    width: 1,
                    height: 1,
                    different_pixels: 0,
                    max_channel_delta: 0,
                    within_tolerance: true,
                },
                byte_exact: true,
                reference_is_transparent_blank: true,
                adapter_check: None,
            }),
        };
        let mut counts = Counts::default();

        emit_entry(
            &entry,
            execution,
            false,
            Path::new("output"),
            &mut counts,
            &mut Vec::new(),
            &mut Vec::new(),
        )
        .unwrap();

        assert_eq!(counts.byte_exact, 1);
        assert_eq!(counts.transparent_blank_byte_exact, 1);
    }

    #[test]
    fn stub_baseline_allows_transparent_blank_byte_matches() {
        validate_stub_baseline(&Counts {
            exact: 36,
            byte_exact: 26,
            transparent_blank_byte_exact: 26,
            diverges: 1_432,
            gated: 0,
        })
        .unwrap();
    }

    #[test]
    fn stub_baseline_rejects_nonblank_byte_matches() {
        let error = validate_stub_baseline(&Counts {
            exact: 1,
            byte_exact: 1,
            transparent_blank_byte_exact: 0,
            diverges: 1,
            gated: 0,
        })
        .unwrap_err();

        assert!(error.contains("nonblank"));
    }

    #[test]
    fn stub_baseline_requires_a_tolerance_divergence() {
        let error = validate_stub_baseline(&Counts {
            exact: 1,
            byte_exact: 0,
            transparent_blank_byte_exact: 0,
            diverges: 0,
            gated: 0,
        })
        .unwrap_err();

        assert!(error.contains("no tolerance-divergent references"));
    }

    #[test]
    fn entry_outcomes_do_not_own_compared_images() {
        assert!(size_of::<EntryOutcome>() < size_of::<RgbaImage>() * 2);
    }

    #[test]
    fn rejects_duplicate_manifest_entry_ids() {
        let entries = [
            entry("duplicate", "clockwise-atomic", "first.png"),
            entry("duplicate", "clockwise-atomic", "second.png"),
        ];
        let error = validate_entry_ids(&entries).unwrap_err();

        assert!(error.contains("duplicate manifest entry id `duplicate`"));
    }

    #[test]
    fn rejects_case_insensitive_manifest_entry_id_collisions() {
        let entries = [
            entry("Case", "clockwise-atomic", "first.png"),
            entry("case", "clockwise-atomic", "second.png"),
        ];
        let error = validate_entry_ids(&entries).unwrap_err();

        assert!(error.contains("`Case` and `case` collide case-insensitively"));
    }

    #[test]
    fn rejects_actual_and_diff_output_namespace_collisions() {
        let entries = [
            entry("shape", "clockwise-atomic", "first.png"),
            entry("shape-diff", "clockwise-atomic", "second.png"),
        ];
        let error = validate_entry_ids(&entries).unwrap_err();

        assert!(error.contains("collides in the renderer output namespace"));
    }

    #[test]
    fn rejects_manifest_entry_ids_with_path_components() {
        let entries = [entry("./shape", "clockwise-atomic", "first.png")];
        let error = validate_entry_ids(&entries).unwrap_err();

        assert!(error.contains("unsafe manifest entry id"));
    }

    #[test]
    fn bounded_runner_propagates_worker_panics_without_deadlocking() {
        let (done_tx, done_rx) = mpsc::channel();
        thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                run_bounded(
                    &[0, 1],
                    2,
                    |entry| {
                        if *entry == 0 {
                            panic!("worker failure");
                        }
                        *entry
                    },
                    |_| false,
                    |_, _| Ok::<_, ()>(()),
                )
                .unwrap();
            });
            done_tx.send(result.is_err()).unwrap();
        });

        assert!(done_rx.recv_timeout(Duration::from_secs(1)).unwrap());
    }

    #[test]
    fn jobs_default_to_one_and_rejects_zero() {
        assert_eq!(Options::parse_args([]).unwrap().jobs, 1);
        let error = Options::parse_args(["--jobs".to_owned(), "0".to_owned()])
            .err()
            .unwrap();
        assert!(error.to_string().contains("positive integer"));
    }

    #[test]
    fn replay_timeout_defaults_to_sixty_seconds_and_accepts_an_override() {
        assert_eq!(
            Options::parse_args([]).unwrap().replay_timeout,
            Duration::from_secs(60)
        );
        assert_eq!(
            Options::parse_args(["--replay-timeout-seconds".to_owned(), "17".to_owned()])
                .unwrap()
                .replay_timeout,
            Duration::from_secs(17)
        );
    }

    #[test]
    fn dynamic_reference_replay_and_backend_are_an_atomic_option_pair() {
        for lone_option in ["--reference-replay", "--reference-backend"] {
            let error = Options::parse_args([lone_option.to_owned(), "value".to_owned()])
                .err()
                .unwrap();
            assert!(error.to_string().contains("must be provided together"));
        }

        let options = Options::parse_args([
            "--reference-replay".to_owned(),
            "renderer-replay".to_owned(),
            "--reference-backend".to_owned(),
            "ffi-dawn".to_owned(),
        ])
        .unwrap();
        let reference = options.dynamic_reference().unwrap();
        assert_eq!(reference.replay, Path::new("renderer-replay"));
        assert_eq!(reference.backend, "ffi-dawn");
    }

    #[test]
    fn dynamic_reference_rejects_a_non_dawn_oracle_backend() {
        let error = Options::parse_args([
            "--reference-replay".to_owned(),
            "renderer-replay".to_owned(),
            "--reference-backend".to_owned(),
            "rust-wgpu".to_owned(),
        ])
        .err()
        .unwrap();

        assert!(error
            .to_string()
            .contains("--reference-backend must be `ffi-dawn`"));
    }
}
