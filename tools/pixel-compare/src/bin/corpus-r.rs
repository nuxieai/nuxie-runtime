use pixel_compare::{
    artifact, compare, validate_reference_identities, DiffReport, ReferenceIdentity, RgbaImage,
    Tolerance,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{atomic::AtomicBool, atomic::Ordering, mpsc};
use std::thread;

#[derive(Debug, Deserialize)]
struct Manifest {
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    kind: String,
    stream: PathBuf,
    reference: PathBuf,
    backend: String,
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
    let manifest: Manifest = toml::from_str(&fs::read_to_string(&options.manifest)?)?;
    let reference_base = std::env::current_dir()?;
    validate_entry_ids(&manifest.entry)?;
    validate_reference_identity(&reference_base, &manifest.entry)?;
    let entries = selected_entries(&manifest.entry, &options.probe_gated)?;
    fs::create_dir_all(&options.output_dir)?;
    let mut counts = Counts::default();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    run_bounded(
        &entries,
        options.jobs,
        |entry| run_entry(*entry, &options, &reference_base),
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
            "renderer-corpus-probe passes={} diverges={} total={}",
            counts.exact,
            counts.diverges,
            entries.len()
        )?;
        return Ok(());
    }
    writeln!(
        stdout,
        "renderer-corpus exact={} diverges={} gated={} total={}",
        counts.exact,
        counts.diverges,
        counts.gated,
        entries.len()
    )?;
    if options.expect_all_fail {
        if counts.exact != 0 {
            return Err(
                format!("stub baseline unexpectedly passed {} entries", counts.exact).into(),
            );
        }
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
    Gated { diagnostic: String },
    Compared { report: DiffReport },
}

#[derive(Debug, Default)]
struct ChildDiagnostics {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Debug)]
struct EntryExecution {
    diagnostics: ChildDiagnostics,
    outcome: Result<EntryOutcome, String>,
}

#[derive(Default)]
struct Counts {
    exact: usize,
    diverges: usize,
    gated: usize,
}

fn run_entry(entry: &Entry, options: &Options, reference_base: &Path) -> EntryExecution {
    if entry.status == "gated" && options.probe_gated.is_empty() {
        return EntryExecution {
            diagnostics: ChildDiagnostics::default(),
            outcome: Ok(EntryOutcome::Gated {
                diagnostic: entry.gated.as_deref().unwrap_or("no diagnostic").to_owned(),
            }),
        };
    }
    let reference = match resolve_reference(reference_base, entry) {
        Ok(reference) => reference,
        Err(error) => return EntryExecution::failed(error),
    };
    let actual = options.output_dir.join(format!("{}.png", entry.id));
    let stream = match path_str(&entry.stream) {
        Ok(stream) => stream,
        Err(error) => return EntryExecution::failed(error.to_string()),
    };
    let actual_path = match path_str(&actual) {
        Ok(actual) => actual,
        Err(error) => return EntryExecution::failed(error.to_string()),
    };
    let output = match Command::new(&options.replay)
        .args(["--stream", stream])
        .args(["--output", actual_path])
        .args(["--backend", &options.backend])
        .args(["--frame", &entry.frame.to_string()])
        .args(["--mode", &entry.mode])
        .output()
    {
        Ok(output) => output,
        Err(error) => return EntryExecution::failed(error.to_string()),
    };
    let diagnostics = ChildDiagnostics {
        stdout: output.stdout,
        stderr: output.stderr,
    };
    let outcome = compare_entry(
        entry,
        &reference,
        &actual,
        &options.output_dir,
        output.status.success(),
    );
    EntryExecution {
        diagnostics,
        outcome,
    }
}

impl EntryExecution {
    fn failed(error: String) -> Self {
        Self {
            diagnostics: ChildDiagnostics::default(),
            outcome: Err(error),
        }
    }
}

fn compare_entry(
    entry: &Entry,
    reference: &Path,
    actual: &Path,
    output_dir: &Path,
    replay_succeeded: bool,
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
    Ok(EntryOutcome::Compared { report })
}

struct ReferenceRewrite {
    id: &'static str,
    kind: &'static str,
    stream: &'static str,
    source: &'static str,
    target: &'static str,
    mode: &'static str,
    backend: &'static str,
}

const REFERENCE_REWRITES: [ReferenceRewrite; 2] = [
    ReferenceRewrite {
        id: "gm-dstreadshuffle-clockwise-atomic",
        kind: "upstream-gm-stream",
        stream: "fixtures/renderer/streams/gm/dstreadshuffle.rive-stream",
        source: "fixtures/renderer/reference/metal/gm/dstreadshuffle-clockwise-atomic.png",
        target: "fixtures/renderer/reference/metal/gm/dstreadshuffle.png",
        mode: "clockwise-atomic",
        backend: "metal",
    },
    ReferenceRewrite {
        id: "gm-interleavedfeather-clockwise-atomic",
        kind: "upstream-gm-stream",
        stream: "fixtures/renderer/streams/gm/interleavedfeather.rive-stream",
        source: "fixtures/renderer/reference/metal/gm/interleavedfeather-clockwise-atomic.png",
        target: "fixtures/renderer/reference/metal/gm/interleavedfeather.png",
        mode: "clockwise-atomic",
        backend: "metal",
    },
];

fn resolved_reference_target(entry: &Entry) -> Result<PathBuf, String> {
    let Some(rewrite) = REFERENCE_REWRITES
        .iter()
        .find(|rewrite| entry.id == rewrite.id)
        .or_else(|| {
            REFERENCE_REWRITES
                .iter()
                .find(|rewrite| entry.reference.as_os_str() == std::ffi::OsStr::new(rewrite.source))
        })
    else {
        return Ok(entry.reference.clone());
    };

    let mismatch = if entry.id != rewrite.id {
        Some(("entry id", entry.id.as_str(), rewrite.id))
    } else if entry.kind != rewrite.kind {
        Some(("source kind", entry.kind.as_str(), rewrite.kind))
    } else if entry.stream.as_os_str() != std::ffi::OsStr::new(rewrite.stream) {
        Some((
            "stream",
            entry.stream.to_str().unwrap_or("<non-UTF-8>"),
            rewrite.stream,
        ))
    } else if entry.reference.as_os_str() != std::ffi::OsStr::new(rewrite.source) {
        Some((
            "reference",
            entry.reference.to_str().unwrap_or("<non-UTF-8>"),
            rewrite.source,
        ))
    } else if entry.mode != rewrite.mode {
        Some(("mode", entry.mode.as_str(), rewrite.mode))
    } else if entry.backend != rewrite.backend {
        Some(("backend", entry.backend.as_str(), rewrite.backend))
    } else {
        None
    };
    if let Some((field, actual, expected)) = mismatch {
        return Err(format!(
            "reference rewrite identity mismatch for {}: {field} is `{actual}`, expected `{expected}`",
            entry.reference.display()
        ));
    }

    Ok(PathBuf::from(rewrite.target))
}

fn resolve_reference(base: &Path, entry: &Entry) -> Result<PathBuf, String> {
    let target = resolved_reference_target(entry)?;
    let physical = if target.is_absolute() {
        target
    } else {
        base.join(target)
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
        EntryOutcome::Compared { report } if report.within_tolerance => {
            counts.exact += 1;
            writeln!(
                stdout,
                "{} {}: different-pixels={} max-channel-delta={}",
                if probing { "probe-pass" } else { "exact" },
                entry.id,
                report.different_pixels,
                report.max_channel_delta
            )
            .map_err(|error| error.to_string())?;
        }
        EntryOutcome::Compared { report } => {
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
        }
    }
    stdout.flush().map_err(|error| error.to_string())
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
    let resolved = entries
        .iter()
        .map(resolved_reference_target)
        .collect::<Result<Vec<_>, _>>()?;
    let mut owners = HashMap::<PathBuf, (&str, &Path)>::new();
    for (entry, reference) in entries.iter().zip(&resolved) {
        let physical = if reference.is_absolute() {
            reference.clone()
        } else {
            base.join(reference)
        };
        let physical = physical.canonicalize().unwrap_or(physical);
        if let Some((previous_id, previous_reference)) =
            owners.insert(physical.clone(), (&entry.id, &entry.reference))
        {
            if previous_reference != entry.reference {
                return Err(format!(
                    "manifest entries {previous_id} ({}) and {} ({}) resolve to the same physical reference {}; path rewrites must remain unique",
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
        entries
            .iter()
            .zip(&resolved)
            .map(|(entry, reference)| ReferenceIdentity {
                id: &entry.id,
                stream: &entry.stream,
                frame: entry.frame,
                mode: &entry.mode,
                reference,
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
                _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
            }
        }
        Ok(Self {
            manifest,
            replay,
            backend,
            output_dir,
            jobs,
            expect_all_fail,
            probe_gated,
        })
    }
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error + Send + Sync>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn usage() -> &'static str {
    "usage: corpus-r [--manifest FILE] [--replay FILE] [--backend stub|rust-wgpu|ffi-metal] [--output-dir DIR] [--jobs N] [--expect-all-fail] [--probe-gated ID ...]"
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
            kind: "upstream-gm-stream".to_owned(),
            stream: PathBuf::from("fixture.rive-stream"),
            reference: PathBuf::from(reference),
            backend: "metal".to_owned(),
            status: "gated".to_owned(),
            frame: 0,
            max_channel_delta: 2,
            max_different_pixels: 0,
            gated: Some("algorithm-core".to_owned()),
            mode: mode.to_owned(),
        }
    }

    fn dstreadshuffle_alias_entry() -> Entry {
        let mut entry = entry(
            "gm-dstreadshuffle-clockwise-atomic",
            "clockwise-atomic",
            "fixtures/renderer/reference/metal/gm/dstreadshuffle-clockwise-atomic.png",
        );
        entry.stream = PathBuf::from("fixtures/renderer/streams/gm/dstreadshuffle.rive-stream");
        entry
    }

    fn interleavedfeather_alias_entry() -> Entry {
        let mut entry = entry(
            "gm-interleavedfeather-clockwise-atomic",
            "clockwise-atomic",
            "fixtures/renderer/reference/metal/gm/interleavedfeather-clockwise-atomic.png",
        );
        entry.stream = PathBuf::from("fixtures/renderer/streams/gm/interleavedfeather.rive-stream");
        entry
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
    fn resolves_exactly_the_two_approved_manifest_reference_paths() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        for (entry, target) in [
            (
                dstreadshuffle_alias_entry(),
                "fixtures/renderer/reference/metal/gm/dstreadshuffle.png",
            ),
            (
                interleavedfeather_alias_entry(),
                "fixtures/renderer/reference/metal/gm/interleavedfeather.png",
            ),
        ] {
            assert_eq!(
                resolve_reference(&repo_root, &entry).unwrap(),
                repo_root.join(target)
            );
        }
    }

    #[test]
    fn rejects_rewrite_for_wrong_entry_id() {
        let mut entry = dstreadshuffle_alias_entry();
        entry.id = "gm-not-dstreadshuffle-clockwise-atomic".to_owned();
        let error = resolved_reference_target(&entry).unwrap_err();
        assert!(error.contains("entry id"));
    }

    #[test]
    fn rejects_rewrite_for_wrong_corpus_source_kind() {
        let mut entry = dstreadshuffle_alias_entry();
        entry.kind = "riv-stream".to_owned();
        let error = resolved_reference_target(&entry).unwrap_err();
        assert!(error.contains("source kind"));
    }

    #[test]
    fn rejects_rewrite_for_wrong_stream() {
        let mut entry = dstreadshuffle_alias_entry();
        entry.stream = PathBuf::from("fixtures/renderer/streams/gm/interleavedfeather.rive-stream");
        let error = resolved_reference_target(&entry).unwrap_err();
        assert!(error.contains("stream"));
    }

    #[test]
    fn rejects_rewrite_for_wrong_mode() {
        let mut entry = dstreadshuffle_alias_entry();
        entry.mode = "msaa".to_owned();
        let error = resolved_reference_target(&entry).unwrap_err();
        assert!(error.contains("mode"));
    }

    #[test]
    fn rejects_rewrite_for_wrong_backend() {
        let mut entry = dstreadshuffle_alias_entry();
        entry.backend = "dawn-webgpu-metal".to_owned();
        let error = resolved_reference_target(&entry).unwrap_err();
        assert!(error.contains("backend"));
    }

    #[test]
    fn approved_alias_id_with_direct_reference_fails_before_replay() {
        let mut entry = dstreadshuffle_alias_entry();
        entry.reference = PathBuf::from("fixtures/renderer/reference/metal/gm/dstreadshuffle.png");
        let options = Options {
            manifest: PathBuf::from("unused.toml"),
            replay: PathBuf::from("replay-must-not-run"),
            backend: "rust-wgpu".to_owned(),
            output_dir: PathBuf::from("unused-output"),
            jobs: 1,
            expect_all_fail: false,
            probe_gated: vec![entry.id.clone()],
        };

        let execution = run_entry(&entry, &options, Path::new("/repo"));
        let error = execution.outcome.unwrap_err();
        assert!(error.contains("reference rewrite identity mismatch"));
        assert!(error.contains("reference"));
        assert!(execution.diagnostics.stdout.is_empty());
        assert!(execution.diagnostics.stderr.is_empty());
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
    fn rejects_collisions_after_reference_path_resolution() {
        let entries = [
            dstreadshuffle_alias_entry(),
            entry(
                "direct",
                "clockwise-atomic",
                "fixtures/renderer/reference/metal/gm/dstreadshuffle.png",
            ),
        ];
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let error = validate_reference_identity(&repo_root, &entries).unwrap_err();
        assert!(error.contains("resolve to the same physical reference"));
        assert!(error.contains("gm-dstreadshuffle-clockwise-atomic"));
        assert!(error.contains("direct"));
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
}
