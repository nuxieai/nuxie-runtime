//! Paired GPU inputs for the nine local paths audited in README.md.
//! Keeps production CSS and the complete authored replay unchanged.
use anyhow::{Context, ensure};
use nuxie_runtime::{
    RuntimeArtboardInstanceHandle,
    source::shapes::{paint::shape_paint::ShapePaintPathKind, shape::Shape},
};
use silver_corpus::{
    Execution, OpKind, Operation, Value, parse_sriv, read_manifest, resolve_expected,
};
use std::path::PathBuf;

fn child(
    board: &RuntimeArtboardInstanceHandle,
    object: usize,
    index: i32,
) -> RuntimeArtboardInstanceHandle {
    board
        .with_artboard(|b| b.objects()[object].clone())
        .unwrap()
        .with_mut(|o| o.as_artboard_host_mut().unwrap().artboard_instance(index))
        .flatten()
        .unwrap()
}
fn rebuild(board: &RuntimeArtboardInstanceHandle, object: usize) {
    let shape = board
        .with_artboard(|b| b.objects()[object].clone())
        .unwrap();
    // PathComposer::update's LOCAL branch, using current transforms/geometry.
    // Do not advance scripts, mutate layout, or touch the cache's input snapshot.
    shape
        .with_downcast::<Shape, _>(|shape| {
            assert!(
                shape.is_flagged(nuxie_runtime::source::shapes::path_flags::PathFlags::LOCAL),
                "audited local path"
            );
            let inverse = shape.world_transform().invert_or_identity();
            let paths = shape
                .paths()
                .iter()
                .filter_map(|path| {
                    path.with(|o| {
                        let path = o.as_path().unwrap();
                        if path.is_hidden() || path.is_collapsed() {
                            return None;
                        }
                        let world = o
                            .as_points_path()
                            .map(|p| p.path_transform())
                            .unwrap_or_else(|| path.path_transform());
                        Some((path.raw_path().clone(), inverse * world))
                    })
                })
                .flatten()
                .collect::<Vec<_>>();
            shape.with_path_mut(ShapePaintPathKind::Local, |local| {
                local.rewind();
                for (raw, matrix) in paths {
                    local.add_path(&raw, Some(&matrix));
                }
            });
        })
        .expect("audited Shape owner");
}
fn main() -> anyhow::Result<()> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .context("output directory argument")?,
    );
    let runtime =
        PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").context("pinned RIVE_RUNTIME_DIR")?);
    let root = PathBuf::from(
        option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
    );
    let manifest = read_manifest(&root.join("../../silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|c| c.id == "layoutstest_8-planets-grid")
        .unwrap();
    std::fs::create_dir_all(&output)?;
    ensure!(
        std::fs::read_dir(&output)?.next().is_none(),
        "capture requires an empty output directory"
    );
    let mut cached_bytes = Vec::new();
    for fresh in [false, true] {
        let mut draws = 0;
        let execution = Execution::run_with_before_draw(case, &runtime, |board| {
            if fresh {
                for host in [131, 80, 31] {
                    let nested = child(&child(&child(board, host, 0), 8, 0), 18, 0);
                    for id in [17, 21] {
                        rebuild(&nested, id);
                    }
                }
                for host in [125, 75, 24] {
                    let nested = child(&child(&child(board, host, 0), 8, 0), 21, 0);
                    rebuild(&nested, 407);
                }
            }
            draws += 1;
            Ok(())
        })?;
        ensure!(
            draws == 164,
            "expected complete 164-frame replay, got {draws}"
        );
        let parsed = parse_sriv(execution.bytes())?;
        ensure!(parsed.operations.last().unwrap().frame == 163, "last frame");
        let rewinds = parsed
            .operations
            .iter()
            .filter(|o| o.kind == OpKind::Rewind)
            .count();
        if !fresh {
            cached_bytes = execution.bytes().to_vec();
        }
        if fresh {
            ensure!(
                execution.bytes() != cached_bytes.as_slice(),
                "fresh control must change the stream"
            );
            let cached_stream = parse_sriv(&cached_bytes)?;
            let use_sequence = |stream: &silver_corpus::Sriv| {
                stream
                    .operations
                    .iter()
                    .filter(|op| {
                        matches!(op.kind, OpKind::DrawPath | OpKind::ClipPath)
                            && PATH_IDS.contains(&uint(op, "path_id"))
                    })
                    .map(|op| (op.frame, op.kind as u64, op.fields.clone()))
                    .collect::<Vec<_>>()
            };
            ensure!(
                use_sequence(&cached_stream) == use_sequence(&parsed),
                "fresh control must preserve every path use/paint and its order"
            );
            for id in PATH_IDS {
                let mut adds = std::collections::BTreeMap::new();
                let mut rewinds = std::collections::BTreeMap::new();
                let mut uses = std::collections::BTreeMap::new();
                for op in &parsed.operations {
                    if !matches!(
                        op.kind,
                        OpKind::AddRawPath | OpKind::Rewind | OpKind::DrawPath | OpKind::ClipPath
                    ) || uint(op, "path_id") != id
                    {
                        continue;
                    }
                    match op.kind {
                        OpKind::AddRawPath => {
                            *adds.entry(op.frame).or_insert(0) += 1;
                        }
                        OpKind::Rewind => {
                            *rewinds.entry(op.frame).or_insert(0) += 1;
                        }
                        OpKind::DrawPath | OpKind::ClipPath => {
                            ensure!(
                                adds.get(&op.frame) == Some(&1),
                                "path {id} must rebuild before draw at frame {}",
                                op.frame
                            );
                            if op.frame != 0 {
                                ensure!(
                                    rewinds.get(&op.frame) == Some(&1),
                                    "path {id} must rewind before draw"
                                );
                            }
                            *uses.entry(op.frame).or_insert(0) += 1;
                        }
                        _ => unreachable!(),
                    }
                }
                ensure!(
                    adds.len() == 164
                        && uses.len() == 164
                        && uses.values().all(|n| *n >= 1)
                        && (163..=164).contains(&rewinds.len()),
                    "path {id} complete rebuild/use history: adds={adds:?} rewinds={rewinds:?} uses={uses:?}"
                );
            }
        }
        let lane = if fresh { "fresh" } else { "cached" };
        std::fs::write(output.join(format!("{lane}.sriv")), execution.bytes())?;
        println!(
            "lane={lane} draws={draws} operations={} rewinds={rewinds} bytes={}",
            parsed.operations.len(),
            execution.bytes().len()
        );
    }
    let expected = std::fs::read(resolve_expected(&runtime, case))?;
    let swapped = upstream_local_control(&cached_bytes, &expected)?;
    std::fs::write(output.join("upstream-local.sriv"), swapped)?;
    Ok(())
}

const PATH_IDS: [u64; 9] = [142, 143, 236, 466, 467, 560, 790, 791, 884];
fn uint(op: &Operation, key: &str) -> u64 {
    match op.fields.iter().find(|f| f.name == key).unwrap().value {
        Value::Uint(v) => v,
        _ => panic!("integer field {key}"),
    }
}
fn varuint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
/// Substitute only the audited local path payloads, at matching frame/draw use.
/// Rust's CSS transforms, paints and every other command remain byte-identical.
/// This is a geometry intervention, not a complete upstream renderer oracle.
fn upstream_local_control(cached: &[u8], expected: &[u8]) -> anyhow::Result<Vec<u8>> {
    use std::collections::BTreeMap;
    let source = parse_sriv(expected)?;
    let actual = parse_sriv(cached)?;
    let mut paths = BTreeMap::<u64, Vec<u8>>::new();
    let mut rules = BTreeMap::<u64, Vec<u8>>::new();
    let mut uses = std::collections::VecDeque::new();
    let mut coverage = std::collections::BTreeSet::new();
    for (i, op) in source.operations.iter().enumerate() {
        let end = source
            .operations
            .get(i + 1)
            .map_or(expected.len(), |o| o.offset);
        if !matches!(
            op.kind,
            OpKind::Rewind
                | OpKind::AddRawPath
                | OpKind::FillRule
                | OpKind::DrawPath
                | OpKind::ClipPath
        ) {
            continue;
        }
        let id = uint(op, "path_id");
        if !PATH_IDS.contains(&id) {
            continue;
        }
        match op.kind {
            OpKind::Rewind => {
                paths.entry(id).or_default().clear();
            }
            OpKind::AddRawPath => paths
                .entry(id)
                .or_default()
                .extend_from_slice(&expected[op.offset..end]),
            OpKind::FillRule => {
                rules.insert(id, expected[op.offset..end].to_vec());
            }
            OpKind::DrawPath | OpKind::ClipPath => {
                let mut payload = Vec::new();
                varuint(&mut payload, OpKind::Rewind as u64);
                varuint(&mut payload, id);
                payload.extend_from_slice(paths.get(&id).context("source path payload")?);
                if let Some(rule) = rules.get(&id) {
                    payload.extend_from_slice(rule);
                }
                coverage.insert((op.frame, id));
                uses.push_back((op.frame, op.kind, op.fields.clone(), payload));
            }
            _ => unreachable!(),
        }
    }
    ensure!(coverage.len() == 9 * 164, "all nine paths in all164 frames");
    let substituted_uses = uses.len();
    let mut output = cached[..actual.operations[0].offset].to_vec();
    let mut injected = BTreeMap::<(usize, u64), Vec<u8>>::new();
    for (i, op) in actual.operations.iter().enumerate() {
        let end = actual
            .operations
            .get(i + 1)
            .map_or(cached.len(), |o| o.offset);
        if matches!(op.kind, OpKind::DrawPath | OpKind::ClipPath)
            && PATH_IDS.contains(&uint(op, "path_id"))
        {
            let (frame, kind, fields, payload) = uses.pop_front().context("matching source use")?;
            ensure!(
                frame == op.frame && kind == op.kind && fields == op.fields,
                "source draw/clip identity and order"
            );
            let key = (op.frame, uint(op, "path_id"));
            if let Some(previous) = injected.get(&key) {
                ensure!(
                    previous == &payload,
                    "source path changes between uses in one frame"
                );
            } else {
                // RiveRenderPath cannot mutate after a draw queues it. Every
                // later consumer of this resource in the frame shares the
                // same payload; validate it, but never rewind it again.
                output.extend_from_slice(&payload);
                injected.insert(key, payload);
            }
        }
        output.extend_from_slice(&cached[op.offset..end]);
    }
    ensure!(uses.is_empty(), "all source path uses consumed");
    parse_sriv(&output)?;
    println!(
        "lane=upstream-local substituted_uses={} all other commands preserved",
        substituted_uses
    );
    Ok(output)
}
