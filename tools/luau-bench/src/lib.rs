//! Upstream Luau bench suite runner and ceiling check.
//!
//! `vm` holds the only VM-facing code. Everything else here is VM-agnostic:
//! parsing the `bench_support.lua` report lines, aggregating passes, writing
//! and reading result files, and comparing results against a baseline.
//!
//! Bench ids: a script that reports once is named after its file
//! (`tests__sieve`); a script that reports several cases gets one bench per
//! case, `<file>/<description>` (`micro_tests__test_TableSort/table.sort: 12 strings`).

pub mod vm;

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub const SCHEMA: u64 = 1;

/// One `bench.runCode` report: `|><|description|><|t1|><|t2...||_||`, times in ms.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub description: String,
    pub times_ms: Vec<f64>,
}

pub fn parse_report(line: &str) -> Option<Report> {
    let start = line.find("|><|")?;
    let body = line[start..].strip_suffix("||_||")?;
    let mut fields = body.split("|><|").skip(1);
    let description = fields.next()?.to_string();
    let times_ms = fields
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if times_ms.is_empty() {
        return None;
    }
    Some(Report {
        description,
        times_ms,
    })
}

/// Bench ids and their reports for one script's printed output.
pub fn script_benches(script: &str, lines: &[String]) -> Vec<(String, Report)> {
    let reports: Vec<Report> = lines
        .iter()
        .filter_map(|line| parse_report(line.trim_end()))
        .collect();
    if reports.len() == 1 {
        return reports
            .into_iter()
            .map(|report| (script.to_string(), report))
            .collect();
    }
    reports
        .into_iter()
        .map(|report| (format!("{script}/{}", report.description), report))
        .collect()
}

pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    Some(if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    })
}

pub fn geomean(values: &[f64]) -> Option<f64> {
    if values.is_empty() || values.iter().any(|value| *value <= 0.0) {
        return None;
    }
    Some((values.iter().map(|value| value.ln()).sum::<f64>() / values.len() as f64).exp())
}

/// Per-bench outcome across all passes of a run.
#[derive(Debug, Clone, Default)]
pub struct BenchResult {
    /// Median of the script's own reported runs, one entry per pass that succeeded.
    pub pass_medians_ms: Vec<f64>,
    /// Error text from passes where the script failed.
    pub errors: Vec<String>,
}

impl BenchResult {
    pub fn median_ms(&self) -> Option<f64> {
        if self.errors.is_empty() {
            median(&self.pass_medians_ms)
        } else {
            None
        }
    }
}

/// A run or baseline file as read back from disk.
#[derive(Debug, Clone)]
pub struct ResultFile {
    pub vm: String,
    pub meta: Map<String, Value>,
    /// Bench id to median ms; `None` where the bench failed.
    pub benches: BTreeMap<String, Option<f64>>,
}

pub fn results_json(
    vm: &str,
    runs: usize,
    meta: &Map<String, Value>,
    passes: &[Value],
    benches: &BTreeMap<String, BenchResult>,
) -> Value {
    let mut bench_map = Map::new();
    for (id, result) in benches {
        let entry = match result.median_ms() {
            Some(median_ms) => json!({
                "status": "ok",
                "median_ms": round3(median_ms),
                "pass_medians_ms": result.pass_medians_ms.iter().map(|value| round3(*value)).collect::<Vec<_>>(),
            }),
            None => json!({ "status": "error", "errors": result.errors }),
        };
        bench_map.insert(id.clone(), entry);
    }
    let ok: Vec<f64> = benches
        .values()
        .filter_map(BenchResult::median_ms)
        .collect();
    json!({
        "schema": SCHEMA,
        "vm": vm,
        "runs": runs,
        "aggregate": "per bench: median of the script's reported runs (bench_support: 20 kept of 24) per pass, then median across passes",
        "meta": meta,
        "passes": passes,
        "summary": {
            "benches": benches.len(),
            "ok": ok.len(),
            "geomean_ms": geomean(&ok).map(round3),
        },
        "benches": bench_map,
    })
}

pub fn results_csv(benches: &BTreeMap<String, BenchResult>) -> String {
    let mut out = String::from("bench,status,median_ms,min_pass_ms,max_pass_ms,passes\n");
    for (id, result) in benches {
        let quoted = format!("\"{}\"", id.replace('"', "\"\""));
        match result.median_ms() {
            Some(median_ms) => {
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
                out.push_str(&format!(
                    "{quoted},ok,{median_ms:.3},{min:.3},{max:.3},{}\n",
                    result.pass_medians_ms.len()
                ));
            }
            None => out.push_str(&format!(
                "{quoted},error,,,,{}\n",
                result.pass_medians_ms.len()
            )),
        }
    }
    out
}

pub fn parse_result_file(text: &str) -> Result<ResultFile, String> {
    let root: Value =
        serde_json::from_str(text).map_err(|error| format!("invalid JSON: {error}"))?;
    if root.get("schema").and_then(Value::as_u64) != Some(SCHEMA) {
        return Err(format!("unsupported result schema (want {SCHEMA})"));
    }
    let vm = root
        .get("vm")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let meta = root
        .get("meta")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut benches = BTreeMap::new();
    for (id, entry) in root
        .get("benches")
        .and_then(Value::as_object)
        .ok_or("missing `benches` object")?
    {
        let ok = entry.get("status").and_then(Value::as_str) == Some("ok");
        benches.insert(
            id.clone(),
            if ok {
                entry.get("median_ms").and_then(Value::as_f64)
            } else {
                None
            },
        );
    }
    Ok(ResultFile { vm, meta, benches })
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Ok,
    /// Slower than the ceiling allows.
    OverCeiling,
    /// In the baseline but missing or failed in the results.
    Missing,
    /// In the results only; not judged.
    New,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub bench: String,
    pub baseline_ms: Option<f64>,
    pub result_ms: Option<f64>,
    pub verdict: Verdict,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    pub rows: Vec<Row>,
    pub geomean_ratio: Option<f64>,
    pub compared: usize,
    pub passed: bool,
}

/// Judges results against a baseline. A bench fails when it is slower than
/// `ceiling` times its baseline AND the absolute difference exceeds
/// `slack_ms` (sub-millisecond benches jitter by more than 10% between quiet
/// runs). The geomean of result/baseline over the shared benches must also
/// stay within `ceiling`.
pub fn compare(
    baseline: &ResultFile,
    results: &ResultFile,
    ceiling: f64,
    slack_ms: f64,
) -> Comparison {
    let mut rows = Vec::new();
    let mut ratios = Vec::new();
    for (bench, baseline_ms) in &baseline.benches {
        let Some(baseline_ms) = *baseline_ms else {
            continue;
        };
        let result_ms = results.benches.get(bench).copied().flatten();
        let verdict = match result_ms {
            None => Verdict::Missing,
            Some(result_ms) => {
                ratios.push(result_ms / baseline_ms);
                if result_ms > baseline_ms * ceiling && result_ms - baseline_ms > slack_ms {
                    Verdict::OverCeiling
                } else {
                    Verdict::Ok
                }
            }
        };
        rows.push(Row {
            bench: bench.clone(),
            baseline_ms: Some(baseline_ms),
            result_ms,
            verdict,
        });
    }
    for (bench, result_ms) in &results.benches {
        if !baseline.benches.get(bench).is_some_and(Option::is_some) {
            rows.push(Row {
                bench: bench.clone(),
                baseline_ms: None,
                result_ms: *result_ms,
                verdict: Verdict::New,
            });
        }
    }
    rows.sort_by(|a, b| a.bench.cmp(&b.bench));
    let geomean_ratio = geomean(&ratios);
    let passed = rows
        .iter()
        .all(|row| matches!(row.verdict, Verdict::Ok | Verdict::New))
        && geomean_ratio.is_none_or(|ratio| ratio <= ceiling);
    Comparison {
        rows,
        geomean_ratio,
        compared: ratios.len(),
        passed,
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bench_support_report_lines() {
        let report = parse_report("|><|sieve|><|99.5|><|101.25||_||").unwrap();
        assert_eq!(report.description, "sieve");
        assert_eq!(report.times_ms, vec![99.5, 101.25]);
        assert!(parse_report("RESULT sieve DONE").is_none());
        assert!(parse_report("|><|sieve||_||").is_none());
        assert!(parse_report("|><|sieve|><|abc||_||").is_none());
    }

    #[test]
    fn names_multi_report_scripts_per_case() {
        let lines = vec![
            "|><|a|><|1||_||".to_string(),
            "noise".to_string(),
            "|><|b|><|2||_||".to_string(),
        ];
        let ids: Vec<String> = script_benches("micro", &lines)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(ids, vec!["micro/a", "micro/b"]);
        let single: Vec<String> = script_benches("tests__sieve", &lines[..1])
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(single, vec!["tests__sieve"]);
    }

    #[test]
    fn median_and_geomean() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(&[4.0, 1.0, 2.0, 3.0]), Some(2.5));
        assert_eq!(median(&[]), None);
        assert!((geomean(&[1.0, 4.0]).unwrap() - 2.0).abs() < 1e-12);
    }

    fn file(entries: &[(&str, Option<f64>)]) -> ResultFile {
        ResultFile {
            vm: "luaur".into(),
            meta: Map::new(),
            benches: entries
                .iter()
                .map(|(id, ms)| (id.to_string(), *ms))
                .collect(),
        }
    }

    #[test]
    fn compare_flags_slow_missing_and_geomean() {
        let baseline = file(&[("a", Some(10.0)), ("b", Some(10.0)), ("c", Some(0.1))]);
        let same = compare(
            &baseline,
            &file(&[("a", Some(10.5)), ("b", Some(9.0)), ("c", Some(0.1))]),
            1.10,
            0.0,
        );
        assert!(same.passed);

        let slow = compare(
            &baseline,
            &file(&[("a", Some(11.5)), ("b", Some(10.0)), ("c", Some(0.1))]),
            1.10,
            0.0,
        );
        assert!(!slow.passed);
        assert_eq!(slow.rows[0].verdict, Verdict::OverCeiling);

        // A jittery sub-millisecond bench stays within the absolute slack.
        let jitter = compare(
            &baseline,
            &file(&[("a", Some(10.0)), ("b", Some(10.0)), ("c", Some(0.13))]),
            1.10,
            0.25,
        );
        assert!(jitter.passed);

        let missing = compare(
            &baseline,
            &file(&[("a", Some(10.0)), ("b", None)]),
            1.10,
            0.0,
        );
        assert!(!missing.passed);

        // Every bench within the ceiling, geomean over it.
        let drift = compare(
            &baseline,
            &file(&[("a", Some(10.9)), ("b", Some(10.9)), ("c", Some(0.13))]),
            1.10,
            0.25,
        );
        assert!(drift.rows.iter().all(|row| row.verdict == Verdict::Ok));
        assert!(!drift.passed);

        let extra = compare(
            &baseline,
            &file(&[
                ("a", Some(10.0)),
                ("b", Some(10.0)),
                ("c", Some(0.1)),
                ("d", Some(1.0)),
            ]),
            1.10,
            0.0,
        );
        assert!(extra.passed);
        assert_eq!(extra.rows.last().unwrap().verdict, Verdict::New);
    }
}
