use anyhow::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

mod action;
mod pointer_log_replay;
mod scripted_artboard_replay;
mod scripted_misc_replay;
mod scripted_properties_replay;
mod scripted_source;
mod scripted_text_runs_replay;
mod scripting;
mod stateful_component_replay;
mod view_model_dependents_replay;
pub use action::{
    Action, ActionTarget, Execution, GamepadInputKind, GamepadMapping, GamepadRecord,
    PointerCoordinate,
};

pub const EXPECTED_ENTRIES: usize = 282;
pub const EXPECTED_RUNTIME: usize = 231;
pub const EXPECTED_SCRIPTED: usize = 48;
pub const MAX_PROVENANCE_UNKNOWN: usize = 2;
pub const UPSTREAM_REF: &str = "f01989c611635ed8343db65041d4f48ec3c2fc46";

pub use nuxie_sriv::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub corpus: CorpusConfig,
    #[serde(rename = "case")]
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusConfig {
    pub version: u32,
    pub upstream_ref: String,
    pub expected_entries: usize,
    pub expected_runtime: usize,
    pub expected_scripted: usize,
    pub max_provenance_unknown: usize,
    pub min_cpp_rust_exact: usize,
    #[serde(default)]
    pub cpp_rust_exact_ids: Vec<String>,
    /// Explicit CSS replacements for historical exact obligations. The
    /// original IDs and numeric floor stay intact; equality is reported apart.
    #[serde(default)]
    pub reviewed_layout_divergences: Vec<ReviewedLayoutDivergence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedLayoutDivergence {
    pub id: String,
    pub first_difference: String,
    #[serde(default)]
    pub rust_stream_sha256: String,
    /// Reviewed platform-specific streams. An empty single hash means absent.
    #[serde(default)]
    pub rust_stream_sha256_by_target: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Lane {
    Runtime,
    Scripted,
    Unknown,
    Retired,
}

impl Display for Lane {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Runtime => "runtime",
            Self::Scripted => "scripted",
            Self::Unknown => "unknown",
            Self::Retired => "retired",
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Exact,
    Pending,
    PendingScripted,
    Diverges,
    UnsupportedFeature,
    ProvenanceUnknown,
    RetiredUpstream,
}

impl Display for Status {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Exact => "exact",
            Self::Pending => "pending",
            Self::PendingScripted => "pending-scripted",
            Self::Diverges => "diverges",
            Self::UnsupportedFeature => "unsupported-feature",
            Self::ProvenanceUnknown => "provenance-unknown",
            Self::RetiredUpstream => "retired-upstream",
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub expected: String,
    pub source: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub artboard: String,
    #[serde(default)]
    pub clone_artboard_instance: bool,
    pub animation: String,
    pub state_machine: String,
    pub lane: Lane,
    pub deterministic: String,
    pub random: String,
    pub view_model: String,
    #[serde(default)]
    pub sample_times: Vec<f32>,
    pub actions: Actions,
    pub verification: String,
    pub status: Status,
    pub producer_class: String,
    pub provenance_file: String,
    pub provenance_test: String,
    pub producer_line: usize,
    pub note: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Actions {
    Executable(Vec<Action>),
    Legacy(String),
}

impl Actions {
    pub fn executable(&self) -> Option<&[Action]> {
        match self {
            Self::Executable(actions) => Some(actions),
            Self::Legacy(_) => None,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Executable(actions) => actions.is_empty(),
            Self::Legacy(marker) => marker.is_empty(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CorpusSummary {
    pub entries: usize,
    pub provenanced: usize,
    pub runtime: usize,
    pub scripted: usize,
    pub retired: usize,
    pub cpp_baseline_exact: usize,
    pub cpp_rust_exact: usize,
    pub reviewed_layout_divergences: usize,
    pub statuses: BTreeMap<Status, usize>,
    pub operations: usize,
    pub bytes: u64,
}

impl CorpusSummary {
    pub fn status(&self, status: Status) -> usize {
        self.statuses.get(&status).copied().unwrap_or(0)
    }
}

pub fn read_manifest(path: &Path) -> anyhow::Result<Manifest> {
    let contents = fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
    toml::from_str(&contents)
        .map_err(|error| anyhow::anyhow!("failed to parse {}: {error}", path.display()))
}

pub fn validate_manifest(manifest: &Manifest, runtime_dir: &Path) -> anyhow::Result<CorpusSummary> {
    let config = &manifest.corpus;
    anyhow::ensure!(config.version == 1, "manifest version must be 1");
    anyhow::ensure!(
        config.upstream_ref == UPSTREAM_REF,
        "manifest upstream_ref must remain {UPSTREAM_REF}"
    );
    anyhow::ensure!(
        config.expected_entries == EXPECTED_ENTRIES,
        "expected_entries ratchet must remain {EXPECTED_ENTRIES}"
    );
    anyhow::ensure!(
        config.expected_runtime == EXPECTED_RUNTIME,
        "expected_runtime ratchet must remain {EXPECTED_RUNTIME}"
    );
    anyhow::ensure!(
        config.expected_scripted == EXPECTED_SCRIPTED,
        "expected_scripted ratchet must remain {EXPECTED_SCRIPTED}"
    );
    anyhow::ensure!(
        config.max_provenance_unknown <= MAX_PROVENANCE_UNKNOWN,
        "max_provenance_unknown may not exceed {MAX_PROVENANCE_UNKNOWN}"
    );
    anyhow::ensure!(
        manifest.cases.len() == config.expected_entries,
        "manifest has {} entries; expected {}",
        manifest.cases.len(),
        config.expected_entries
    );

    let mut ids = BTreeSet::new();
    let mut expected_paths = BTreeSet::new();
    let mut exact_ids = BTreeSet::new();
    let mut summary = CorpusSummary {
        entries: manifest.cases.len(),
        ..CorpusSummary::default()
    };

    for case in &manifest.cases {
        anyhow::ensure!(!case.id.is_empty(), "manifest contains an empty id");
        anyhow::ensure!(
            ids.insert(case.id.as_str()),
            "duplicate case id {}",
            case.id
        );
        anyhow::ensure!(
            expected_paths.insert(case.expected.as_str()),
            "duplicate expected path {}",
            case.expected
        );
        let canonical_expected = format!("tests/unit_tests/silvers/{}.sriv", case.id);
        anyhow::ensure!(
            case.expected == canonical_expected,
            "{} expected path must be {}",
            case.id,
            canonical_expected
        );
        anyhow::ensure!(
            case.verification == "sriv-v1-epsilon",
            "{} has unsupported verification {}",
            case.id,
            case.verification
        );
        anyhow::ensure!(
            !case.note.trim().is_empty(),
            "{} must include a note",
            case.id
        );
        anyhow::ensure!(
            (case.lane == Lane::Retired) == (case.status == Status::RetiredUpstream),
            "{} retired lane and retired-upstream status must be paired",
            case.id
        );
        anyhow::ensure!(
            !case.deterministic.is_empty()
                && !case.random.is_empty()
                && !case.view_model.is_empty()
                && (!case.actions.is_empty()
                    || matches!(
                        case.status,
                        Status::UnsupportedFeature | Status::RetiredUpstream
                    )),
            "{} is missing producer settings",
            case.id
        );

        match case.lane {
            Lane::Runtime => {
                summary.runtime += 1;
                anyhow::ensure!(
                    case.status != Status::Pending,
                    "{} runtime entry must be classified after execution",
                    case.id
                );
                anyhow::ensure!(
                    case.actions.executable().is_some(),
                    "{} runtime entry must use executable actions",
                    case.id
                );
                if case.status == Status::UnsupportedFeature {
                    anyhow::ensure!(
                        case.note.contains("Unsupported feature:"),
                        "{} unsupported entry must name its blocking subsystem",
                        case.id
                    );
                } else {
                    anyhow::ensure!(
                        !case.actions.is_empty(),
                        "{} executable runtime entry must include at least one action",
                        case.id
                    );
                }
                if case.status == Status::Diverges {
                    anyhow::ensure!(
                        case.note.contains("first difference:"),
                        "{} divergent entry must record its first difference",
                        case.id
                    );
                }
            }
            Lane::Scripted => {
                summary.scripted += 1;
                anyhow::ensure!(
                    matches!(
                        case.status,
                        Status::PendingScripted
                            | Status::Exact
                            | Status::Diverges
                            | Status::UnsupportedFeature
                    ),
                    "{} scripted entry must have an explicit supported classification",
                    case.id
                );
                if case.status == Status::Exact {
                    anyhow::ensure!(
                        case.actions
                            .executable()
                            .is_some_and(|actions| !actions.is_empty()),
                        "{} exact scripted entry must have executable replay actions",
                        case.id
                    );
                }
            }
            Lane::Unknown => {}
            Lane::Retired => {
                summary.retired += 1;
                anyhow::ensure!(
                    case.actions
                        .executable()
                        .is_some_and(|actions| actions.is_empty()),
                    "{} retired entry must have an empty action list",
                    case.id
                );
                anyhow::ensure!(
                    !matches!(
                        case.source.as_str(),
                        "inline-script" | "provenance-unknown" | ""
                    ) && !case.provenance_test.trim().is_empty()
                        && case.producer_line > 0,
                    "{} retired entry must retain its upstream source and producer",
                    case.id
                );
            }
        }
        *summary.statuses.entry(case.status).or_default() += 1;

        if case.status == Status::ProvenanceUnknown {
            anyhow::ensure!(
                case.lane == Lane::Unknown,
                "{} provenance-unknown entry must use unknown lane",
                case.id
            );
        } else {
            summary.provenanced += 1;
            anyhow::ensure!(
                case.lane != Lane::Unknown,
                "{} provenanced entry may not use unknown lane",
                case.id
            );
            let provenance = runtime_dir.join(&case.provenance_file);
            anyhow::ensure!(
                provenance.is_file(),
                "{} provenance file is missing: {}",
                case.id,
                provenance.display()
            );
        }
        if case.status == Status::Exact {
            exact_ids.insert(case.id.as_str());
        }

        let expected = runtime_dir.join(&case.expected);
        let bytes = fs::read(&expected).map_err(|error| {
            anyhow::anyhow!(
                "{}: failed to read {}: {error}",
                case.id,
                expected.display()
            )
        })?;
        let parsed = parse_sriv(&bytes).map_err(|error| {
            anyhow::anyhow!("{}: invalid {}: {error}", case.id, expected.display())
        })?;
        summary.cpp_baseline_exact += 1;
        summary.operations += parsed.operations.len();
        summary.bytes += bytes.len() as u64;

        for source in std::iter::once(&case.source).chain(&case.dependencies) {
            if source == "inline-script" || source == "provenance-unknown" {
                continue;
            }
            let path = runtime_dir.join("tests/unit_tests/assets").join(source);
            anyhow::ensure!(
                path.is_file(),
                "{} source is missing: {}",
                case.id,
                path.display()
            );
        }
    }

    anyhow::ensure!(
        summary.runtime == config.expected_runtime,
        "runtime lane has {} entries; expected {}",
        summary.runtime,
        config.expected_runtime
    );
    anyhow::ensure!(
        summary.scripted == config.expected_scripted,
        "scripted lane has {} entries; expected {}",
        summary.scripted,
        config.expected_scripted
    );
    anyhow::ensure!(
        summary.status(Status::ProvenanceUnknown) <= config.max_provenance_unknown,
        "provenance-unknown={} exceeds ratchet {}",
        summary.status(Status::ProvenanceUnknown),
        config.max_provenance_unknown
    );

    let silver_dir = runtime_dir.join("tests/unit_tests/silvers");
    let actual_paths = fs::read_dir(&silver_dir)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", silver_dir.display()))?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|extension| extension.to_str()) == Some("sriv")).then(|| {
                format!(
                    "tests/unit_tests/silvers/{}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let manifest_paths = expected_paths
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    let missing = actual_paths.difference(&manifest_paths).collect::<Vec<_>>();
    let extra = manifest_paths.difference(&actual_paths).collect::<Vec<_>>();
    anyhow::ensure!(
        missing.is_empty() && extra.is_empty(),
        "silver coverage mismatch; unrepresented={missing:?}, missing-files={extra:?}"
    );

    let ratchet_ids = config
        .cpp_rust_exact_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut reviewed_ids = BTreeSet::new();
    for review in &config.reviewed_layout_divergences {
        anyhow::ensure!(
            reviewed_ids.insert(review.id.as_str()),
            "duplicate reviewed layout id: {}",
            review.id
        );
        anyhow::ensure!(
            ratchet_ids.contains(review.id.as_str()),
            "reviewed layout id is not a historical exact obligation: {}",
            review.id
        );
        review.validate_stream_hashes()?;
        let case = manifest
            .cases
            .iter()
            .find(|case| case.id == review.id)
            .with_context(|| format!("reviewed layout case is missing: {}", review.id))?;
        anyhow::ensure!(
            matches!(case.lane, Lane::Runtime | Lane::Scripted)
                && case
                    .actions
                    .executable()
                    .is_some_and(|actions| !actions.is_empty())
                && case.status == Status::Diverges,
            "reviewed layout case must remain an executable divergence: {}",
            review.id
        );
        anyhow::ensure!(
            !review.first_difference.is_empty()
                && case.note.starts_with(
                    "Layout divergence (stock Taffy with CSS semantics replaces Rive's Yoga): "
                )
                && case
                    .note
                    .ends_with(&format!("first difference: {}.", review.first_difference)),
            "reviewed layout first difference or reason changed: {}",
            review.id
        );
    }
    let accounted_ids = exact_ids
        .union(&reviewed_ids)
        .copied()
        .collect::<BTreeSet<_>>();
    let downgraded = ratchet_ids.difference(&accounted_ids).collect::<Vec<_>>();
    anyhow::ensure!(
        downgraded.is_empty(),
        "exact entries were downgraded without a ledger change: {downgraded:?}"
    );
    summary.cpp_rust_exact = exact_ids.len();
    summary.reviewed_layout_divergences = reviewed_ids.len();
    anyhow::ensure!(
        accounted_ids.len() >= config.min_cpp_rust_exact,
        "exact plus reviewed CSS coverage={} is below historical ratchet {}",
        accounted_ids.len(),
        config.min_cpp_rust_exact
    );
    Ok(summary)
}

impl ReviewedLayoutDivergence {
    fn validate_stream_hashes(&self) -> anyhow::Result<()> {
        fn valid_hash(hash: &str) -> bool {
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }
        match (
            &self.rust_stream_sha256_by_target,
            self.rust_stream_sha256.is_empty(),
        ) {
            (None, false) => anyhow::ensure!(
                valid_hash(&self.rust_stream_sha256),
                "reviewed layout requires a full lowercase SHA256 stream hash: {}",
                self.id
            ),
            (Some(hashes), true) => {
                anyhow::ensure!(
                    !hashes.is_empty(),
                    "reviewed target hashes must not be empty: {}",
                    self.id
                );
                for (target, hash) in hashes {
                    anyhow::ensure!(
                        matches!(target.as_str(), "linux-x86_64" | "macos-aarch64"),
                        "unsupported reviewed stream target {target}: {}",
                        self.id
                    );
                    anyhow::ensure!(
                        valid_hash(hash),
                        "reviewed target {target} requires a full lowercase SHA256 stream hash: {}",
                        self.id
                    );
                }
            }
            _ => anyhow::bail!(
                "reviewed layout requires exactly one single or target stream hash representation: {}",
                self.id
            ),
        }
        Ok(())
    }

    fn stream_hash_for_target(&self, target: &str) -> anyhow::Result<&str> {
        self.validate_stream_hashes()?;
        match &self.rust_stream_sha256_by_target {
            Some(hashes) => hashes.get(target).map(String::as_str).with_context(|| {
                format!("no reviewed stream hash for target {target}: {}", self.id)
            }),
            None => Ok(&self.rust_stream_sha256),
        }
    }
}

/// Verify the reviewed replay. The full stream is part of the approval.
pub fn verify_reviewed_layout_stream(
    review: &ReviewedLayoutDivergence,
    difference: &Difference,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    verify_reviewed_layout_stream_for_target(review, difference, bytes, &target)
}

fn verify_reviewed_layout_stream_for_target(
    review: &ReviewedLayoutDivergence,
    difference: &Difference,
    bytes: &[u8],
    target: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        difference.to_string() == review.first_difference,
        "{} reviewed first difference changed",
        review.id
    );
    let expected_hash = review.stream_hash_for_target(target)?;
    let actual_hash = format!("{:x}", Sha256::digest(bytes));
    anyhow::ensure!(
        actual_hash == expected_hash,
        "{} reviewed full stream changed: expected {}, got {}",
        review.id,
        expected_hash,
        actual_hash
    );
    Ok(())
}

pub fn compare_files(expected: &Path, actual: &Path) -> anyhow::Result<()> {
    let expected_bytes = fs::read(expected)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", expected.display()))?;
    let actual_bytes = fs::read(actual)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", actual.display()))?;
    let expected_sriv = parse_sriv(&expected_bytes)
        .map_err(|error| anyhow::anyhow!("invalid expected {}: {error}", expected.display()))?;
    let actual_sriv = parse_sriv(&actual_bytes)
        .map_err(|error| anyhow::anyhow!("invalid actual {}: {error}", actual.display()))?;
    compare_sriv(&expected_sriv, &actual_sriv).map_err(|difference| anyhow::anyhow!("{difference}"))
}

pub fn resolve_expected(runtime_dir: &Path, case: &Case) -> PathBuf {
    runtime_dir.join(&case.expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEST_CORPUS: AtomicUsize = AtomicUsize::new(0);

    struct TestCorpus {
        root: PathBuf,
    }

    impl TestCorpus {
        fn new() -> (Self, Manifest) {
            let root = std::env::temp_dir().join(format!(
                "nuxie-silver-corpus-test-{}-{}",
                std::process::id(),
                NEXT_TEST_CORPUS.fetch_add(1, Ordering::Relaxed),
            ));
            if root.exists() {
                fs::remove_dir_all(&root).unwrap();
            }
            let silvers = root.join("tests/unit_tests/silvers");
            let runtime = root.join("tests/unit_tests/runtime");
            fs::create_dir_all(&silvers).unwrap();
            fs::create_dir_all(&runtime).unwrap();
            fs::write(runtime.join("producer.cpp"), "// fixture").unwrap();

            fs::create_dir_all(root.join("tests/unit_tests/assets")).unwrap();
            fs::write(root.join("tests/unit_tests/assets/retired.riv"), b"fixture").unwrap();
            let mut cases = Vec::new();
            for index in 0..EXPECTED_ENTRIES {
                let id = format!("case-{index:03}");
                fs::write(silvers.join(format!("{id}.sriv")), b"SRIV\x01").unwrap();
                let (lane, status) = if index < EXPECTED_RUNTIME {
                    (Lane::Runtime, Status::UnsupportedFeature)
                } else if index < EXPECTED_RUNTIME + EXPECTED_SCRIPTED {
                    (Lane::Scripted, Status::PendingScripted)
                } else if index == EXPECTED_RUNTIME + EXPECTED_SCRIPTED {
                    (Lane::Retired, Status::RetiredUpstream)
                } else {
                    (Lane::Unknown, Status::ProvenanceUnknown)
                };
                cases.push(Case {
                    id: id.clone(),
                    expected: format!("tests/unit_tests/silvers/{id}.sriv"),
                    source: if lane == Lane::Unknown {
                        "provenance-unknown".to_owned()
                    } else if lane == Lane::Retired {
                        "retired.riv".to_owned()
                    } else {
                        "inline-script".to_owned()
                    },
                    dependencies: Vec::new(),
                    artboard: "default".to_owned(),
                    clone_artboard_instance: false,
                    animation: "none".to_owned(),
                    state_machine: "default".to_owned(),
                    lane,
                    deterministic: "cpp-test-defined".to_owned(),
                    random: "cpp-test-defined".to_owned(),
                    view_model: "none".to_owned(),
                    sample_times: Vec::new(),
                    actions: if matches!(lane, Lane::Runtime | Lane::Retired) {
                        Actions::Executable(Vec::new())
                    } else {
                        Actions::Legacy("cpp-test-body".to_owned())
                    },
                    verification: "sriv-v1-epsilon".to_owned(),
                    status,
                    producer_class: status.to_string(),
                    provenance_file: if lane == Lane::Unknown {
                        String::new()
                    } else {
                        "tests/unit_tests/runtime/producer.cpp".to_owned()
                    },
                    provenance_test: "fixture".to_owned(),
                    producer_line: 1,
                    note: if lane == Lane::Runtime {
                        "Unsupported feature: fixture".to_owned()
                    } else {
                        "fixture".to_owned()
                    },
                });
            }
            let manifest = Manifest {
                corpus: CorpusConfig {
                    version: 1,
                    upstream_ref: UPSTREAM_REF.to_owned(),
                    expected_entries: EXPECTED_ENTRIES,
                    expected_runtime: EXPECTED_RUNTIME,
                    expected_scripted: EXPECTED_SCRIPTED,
                    max_provenance_unknown: MAX_PROVENANCE_UNKNOWN,
                    min_cpp_rust_exact: 0,
                    cpp_rust_exact_ids: Vec::new(),
                    reviewed_layout_divergences: Vec::new(),
                },
                cases,
            };
            (Self { root }, manifest)
        }
    }

    impl Drop for TestCorpus {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn manifest_validation_enforces_full_corpus_and_lane_ratchets() {
        let (fixture, manifest) = TestCorpus::new();
        let summary = validate_manifest(&manifest, &fixture.root).unwrap();
        assert_eq!(summary.entries, EXPECTED_ENTRIES);
        assert_eq!(summary.runtime, EXPECTED_RUNTIME);
        assert_eq!(summary.scripted, EXPECTED_SCRIPTED);
        assert_eq!(summary.cpp_baseline_exact, EXPECTED_ENTRIES);
        assert_eq!(summary.status(Status::ProvenanceUnknown), 2);
        assert_eq!(summary.retired, 1);
        assert_eq!(summary.status(Status::RetiredUpstream), 1);
        assert_eq!(summary.cpp_rust_exact, 0);
    }

    #[test]
    fn retired_baselines_cannot_claim_execution_or_exact_credit() {
        let (fixture, mut manifest) = TestCorpus::new();
        let index = EXPECTED_RUNTIME + EXPECTED_SCRIPTED;
        manifest.cases[index].status = Status::Exact;
        assert!(
            validate_manifest(&manifest, &fixture.root)
                .unwrap_err()
                .to_string()
                .contains("must be paired")
        );
        manifest.cases[index].status = Status::RetiredUpstream;
        manifest.cases[index].actions = Actions::Executable(vec![Action::Draw]);
        assert!(
            validate_manifest(&manifest, &fixture.root)
                .unwrap_err()
                .to_string()
                .contains("empty action list")
        );
        manifest.cases[index].actions = Actions::Executable(Vec::new());
        manifest.corpus.min_cpp_rust_exact = 1;
        manifest.corpus.cpp_rust_exact_ids = vec![manifest.cases[index].id.clone()];
        assert!(validate_manifest(&manifest, &fixture.root).is_err());
    }

    #[test]
    fn retired_baselines_require_real_source_and_provenance() {
        let (fixture, mut manifest) = TestCorpus::new();
        let index = EXPECTED_RUNTIME + EXPECTED_SCRIPTED;
        manifest.cases[index].source = "inline-script".to_owned();
        assert!(
            validate_manifest(&manifest, &fixture.root)
                .unwrap_err()
                .to_string()
                .contains("upstream source")
        );
        manifest.cases[index].source = "missing.riv".to_owned();
        assert!(
            validate_manifest(&manifest, &fixture.root)
                .unwrap_err()
                .to_string()
                .contains("source is missing")
        );
        manifest.cases[index].source = "retired.riv".to_owned();
        manifest.cases[index].provenance_file = "missing.cpp".to_owned();
        assert!(
            validate_manifest(&manifest, &fixture.root)
                .unwrap_err()
                .to_string()
                .contains("provenance file is missing")
        );
    }

    #[test]
    fn manifest_validation_rejects_duplicate_ids() {
        let (fixture, mut manifest) = TestCorpus::new();
        manifest.cases[1].id = manifest.cases[0].id.clone();
        let error = validate_manifest(&manifest, &fixture.root).unwrap_err();
        assert!(error.to_string().contains("duplicate case id"));
    }

    #[test]
    fn manifest_validation_rejects_unclassified_runtime_entries() {
        let (fixture, mut manifest) = TestCorpus::new();
        manifest.cases[0].status = Status::Pending;
        manifest.cases[0].actions = Actions::Executable(vec![Action::Draw]);
        let error = validate_manifest(&manifest, &fixture.root).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("runtime entry must be classified after execution")
        );
    }
    fn layout_review_fixture(records: &str) -> (TestCorpus, Manifest) {
        let (fixture, mut manifest) = TestCorpus::new();
        let config = format!(
            r#"version = 1
upstream_ref = "{UPSTREAM_REF}"
expected_entries = {EXPECTED_ENTRIES}
expected_runtime = {EXPECTED_RUNTIME}
expected_scripted = {EXPECTED_SCRIPTED}
max_provenance_unknown = {MAX_PROVENANCE_UNKNOWN}
min_cpp_rust_exact = 1
cpp_rust_exact_ids = ["case-000"]
reviewed_layout_divergences = [{records}]
"#
        );
        manifest.corpus = toml::from_str(&config).unwrap();
        manifest.cases[0].status = Status::Diverges;
        manifest.cases[0].actions = Actions::Executable(vec![Action::Draw]);
        manifest.cases[0].note = "Layout divergence (stock Taffy with CSS semantics replaces Rive's Yoga): CSS content minimum; first difference: frame 0, op 0, width 200 != 260.".to_owned();
        (fixture, manifest)
    }

    const LAYOUT_REVIEW: &str = r#"{ id = "case-000", first_difference = "frame 0, op 0, width 200 != 260", rust_stream_sha256 = "0000000000000000000000000000000000000000000000000000000000000000" }"#;

    #[test]
    fn reviewed_layout_keeps_historical_exact_coverage() {
        let (fixture, manifest) = layout_review_fixture(LAYOUT_REVIEW);
        assert_eq!(manifest.corpus.min_cpp_rust_exact, 1);
        assert_eq!(manifest.corpus.cpp_rust_exact_ids, ["case-000"]);
        let summary = validate_manifest(&manifest, &fixture.root).unwrap();
        // Approved CSS output is reported separately from C++ equality.
        assert_eq!(summary.cpp_rust_exact, 0);
    }

    #[test]
    fn layout_review_cannot_hide_an_unlisted_exact_downgrade() {
        let (fixture, mut manifest) = layout_review_fixture("");
        // An unrelated exact entry meets the numeric floor, but cannot
        // replace the historical case-000 obligation.
        manifest.cases[1].status = Status::Exact;
        manifest.cases[1].actions = Actions::Executable(vec![Action::Draw]);
        let error = validate_manifest(&manifest, &fixture.root).unwrap_err();
        assert!(error.to_string().contains("exact entries were downgraded"));
    }

    #[test]
    fn layout_review_requires_its_exact_first_difference() {
        let (fixture, mut manifest) = layout_review_fixture(LAYOUT_REVIEW);
        manifest.cases[0].note = manifest.cases[0].note.replace("260", "261");
        assert!(validate_manifest(&manifest, &fixture.root).is_err());
    }

    #[test]
    fn layout_review_cannot_count_a_case_twice() {
        let (fixture, manifest) =
            layout_review_fixture(&format!("{LAYOUT_REVIEW}, {LAYOUT_REVIEW}"));
        assert!(validate_manifest(&manifest, &fixture.root).is_err());
    }

    #[test]
    fn layout_review_cannot_credit_an_unrelated_case() {
        let (fixture, mut manifest) = layout_review_fixture(LAYOUT_REVIEW);
        manifest.corpus.cpp_rust_exact_ids = vec!["case-001".to_owned()];
        manifest.cases[1].status = Status::Exact;
        manifest.cases[1].actions = Actions::Executable(vec![Action::Draw]);
        assert!(validate_manifest(&manifest, &fixture.root).is_err());
    }

    #[test]
    fn layout_review_cannot_credit_a_missing_case() {
        let (fixture, mut manifest) = layout_review_fixture(LAYOUT_REVIEW);
        // Preserve the entry count, lane count and complete baseline coverage.
        // The reviewed ID alone disappears, rather than an unrelated validator
        // rejecting a shortened corpus before the missing-review guard runs.
        let case = &mut manifest.cases[0];
        let old_path = fixture.root.join(&case.expected);
        case.id = "renamed-000".to_owned();
        case.expected = "tests/unit_tests/silvers/renamed-000.sriv".to_owned();
        fs::rename(old_path, fixture.root.join(&case.expected)).unwrap();
        let error = validate_manifest(&manifest, &fixture.root).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("reviewed layout case is missing: case-000")
        );
    }

    #[test]
    fn layout_review_cannot_credit_an_unsupported_case() {
        let (fixture, mut manifest) = layout_review_fixture(LAYOUT_REVIEW);
        manifest.cases[0].status = Status::UnsupportedFeature;
        // Keep a valid CSS review note and a valid unsupported-feature note.
        // Only the executable runtime divergence guard must reject this case.
        manifest.cases[0].note = manifest.cases[0]
            .note
            .replace("CSS content minimum", "Unsupported feature: fixture");
        let error = validate_manifest(&manifest, &fixture.root).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("must remain an executable divergence")
        );
    }
    fn two_frame_transform_stream(first_x: f32, second_x: f32) -> Vec<u8> {
        let mut bytes = b"SRIV\x01".to_vec();
        for (frame, x) in [first_x, second_x].into_iter().enumerate() {
            if frame != 0 {
                bytes.push(28);
            }
            bytes.push(9);
            for value in [1.0f32, 0.0, 0.0, 1.0, x, 0.0] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes
    }

    #[test]
    fn layout_review_rejects_a_change_after_the_first_difference() {
        let expected = parse_sriv(&two_frame_transform_stream(0.0, 0.0)).unwrap();
        let approved = two_frame_transform_stream(10.0, 0.0);
        let changed = two_frame_transform_stream(10.0, 20.0);
        let approved_difference =
            compare_sriv(&expected, &parse_sriv(&approved).unwrap()).unwrap_err();
        let changed_difference =
            compare_sriv(&expected, &parse_sriv(&changed).unwrap()).unwrap_err();
        assert_eq!(approved_difference, changed_difference);
        let review = ReviewedLayoutDivergence {
            id: "two-frame-layout".to_owned(),
            first_difference: "frame 0, op 0 (transform), field tx: expected 0, got 10".to_owned(),
            rust_stream_sha256: "193cfb855baa26c49cff15ba0dc597c3772430f3768ee35306aafc8781450197"
                .to_owned(),
            rust_stream_sha256_by_target: None,
        };
        assert!(verify_reviewed_layout_stream(&review, &approved_difference, &approved).is_ok());
        assert!(verify_reviewed_layout_stream(&review, &changed_difference, &changed).is_err());
    }

    #[test]
    fn layout_review_requires_a_full_stream_hash() {
        let (fixture, mut manifest) = layout_review_fixture(LAYOUT_REVIEW);
        manifest.corpus.reviewed_layout_divergences[0]
            .rust_stream_sha256
            .clear();
        assert!(validate_manifest(&manifest, &fixture.root).is_err());
    }
    #[test]
    fn target_hashes_reject_wrong_platform_and_later_frame_changes() {
        let expected = parse_sriv(&two_frame_transform_stream(0.0, 0.0)).unwrap();
        let linux = two_frame_transform_stream(10.0, 0.0);
        let macos = two_frame_transform_stream(10.0, 1.0);
        let changed = two_frame_transform_stream(10.0, 2.0);
        let difference = compare_sriv(&expected, &parse_sriv(&linux).unwrap()).unwrap_err();
        let review = ReviewedLayoutDivergence {
            id: "target-review".to_owned(),
            first_difference: difference.to_string(),
            rust_stream_sha256: String::new(),
            rust_stream_sha256_by_target: Some(BTreeMap::from([
                (
                    "linux-x86_64".to_owned(),
                    format!("{:x}", Sha256::digest(&linux)),
                ),
                (
                    "macos-aarch64".to_owned(),
                    format!("{:x}", Sha256::digest(&macos)),
                ),
            ])),
        };
        for (target, approved, wrong) in [
            ("linux-x86_64", &linux, &macos),
            ("macos-aarch64", &macos, &linux),
        ] {
            assert!(
                verify_reviewed_layout_stream_for_target(&review, &difference, approved, target)
                    .is_ok()
            );
            assert!(
                verify_reviewed_layout_stream_for_target(&review, &difference, wrong, target)
                    .is_err()
            );
            let later_difference =
                compare_sriv(&expected, &parse_sriv(&changed).unwrap()).unwrap_err();
            assert_eq!(later_difference, difference);
            assert!(
                verify_reviewed_layout_stream_for_target(
                    &review,
                    &later_difference,
                    &changed,
                    target
                )
                .is_err()
            );
        }
        assert!(
            verify_reviewed_layout_stream_for_target(&review, &difference, &linux, "linux-aarch64")
                .unwrap_err()
                .to_string()
                .contains("no reviewed stream hash")
        );
        let different_first = compare_sriv(
            &expected,
            &parse_sriv(&two_frame_transform_stream(11.0, 0.0)).unwrap(),
        )
        .unwrap_err();
        assert!(
            verify_reviewed_layout_stream_for_target(
                &review,
                &different_first,
                &linux,
                "linux-x86_64"
            )
            .unwrap_err()
            .to_string()
            .contains("first difference changed")
        );
    }

    #[test]
    fn target_hash_metadata_requires_one_valid_representation() {
        let hash = "0".repeat(64);
        let prefix = r#"id = "case-000"
first_difference = "frame 0, op 0, width 200 != 260"
"#;
        for fields in [
            String::new(),
            format!(
                "rust_stream_sha256 = \"{hash}\"\nrust_stream_sha256_by_target = {{ linux-x86_64 = \"{hash}\" }}"
            ),
            "rust_stream_sha256_by_target = {}".to_owned(),
            format!("rust_stream_sha256_by_target = {{ linux-arm64 = \"{hash}\" }}"),
            "rust_stream_sha256_by_target = { linux-x86_64 = \"abc\" }".to_owned(),
            format!(
                "rust_stream_sha256_by_target = {{ macos-aarch64 = \"{}\" }}",
                "A".repeat(64)
            ),
        ] {
            let review: ReviewedLayoutDivergence =
                toml::from_str(&format!("{prefix}{fields}")).unwrap();
            assert!(
                review.validate_stream_hashes().is_err(),
                "accepted {fields}"
            );
        }
        let review: ReviewedLayoutDivergence = toml::from_str(&format!(
            "{prefix}rust_stream_sha256_by_target = {{ linux-x86_64 = \"{hash}\" }}"
        ))
        .unwrap();
        assert_eq!(review.stream_hash_for_target("linux-x86_64").unwrap(), hash);
        assert!(review.stream_hash_for_target("macos-aarch64").is_err());
    }
}
