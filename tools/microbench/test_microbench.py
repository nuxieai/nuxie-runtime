import copy
import hashlib
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
TOOL_PATH = REPO_ROOT / "tools" / "microbench" / "microbench.py"


def load_tool():
    spec = importlib.util.spec_from_file_location("microbench", TOOL_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def make_sealed_run_fixture(tool, root: pathlib.Path, case_count: int = 1):
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
    subprocess.run(
        ["git", "-C", str(root), "config", "user.email", "test@example.com"],
        check=True,
    )
    (root / ".gitignore").write_text("target/\n")
    manifest = root / "microbenchmarks.toml"
    manifest.write_text('schema = "fixture"\n')
    (root / "benchmark.rs").write_text("measured content\n")
    subprocess.run(["git", "-C", str(root), "add", "."], check=True)
    subprocess.run(["git", "-C", str(root), "commit", "-qm", "measured"], check=True)

    inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
    inventory = inventory._replace(cases=inventory.cases[:case_count])
    run_dir = root / "target" / "run"
    run_id = "fixture-run"
    criterion_home = (
        run_dir
        / "criterion-root"
        / "nuxie-upstream-microbenchmarks"
        / run_id
    )
    samples = {}
    for index, case in enumerate(inventory.cases):
        sample = criterion_home / case.name / "new" / "sample.json"
        sample.parent.mkdir(parents=True)
        sample.write_text(json.dumps({"iters": [1.0], "times": [7.0 + index]}))
        samples[case.name] = sample
    files = {
        "cpp_source_archive": run_dir / "cpp-source.tar",
        "cpp_binary": run_dir / "cpp-build" / "bench",
        "cpp_build_log": run_dir / "cpp-build.log",
        "cpp_output": run_dir / "cpp.txt",
    }
    for path in files.values():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("artifact\n")
    files["cpp_output"].write_text(
        "".join(f"1ms {case.name}\n" for case in inventory.cases)
    )
    cpp_source = run_dir / "cpp-source"
    (cpp_source / "tests").mkdir(parents=True)
    environment = tool.cpp_build_environment(cpp_source, run_dir)
    files["cpp_build_inputs"] = tool.write_cpp_build_inputs(
        cpp_source,
        run_dir,
        environment,
        [str((cpp_source / "build" / "build_rive.sh").resolve()), "release", "--", "bench"],
    )
    run = {
        "schema": tool.RUN_SCHEMA,
        "status": "complete",
        "run_id": run_id,
        "repo_revision": subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip(),
        "benchmark_content_sha256": tool.benchmark_content_identity(root),
        "settings": {"criterion_home": str(criterion_home)},
        "artifacts": {
            "inventory": tool.record_artifact(manifest),
            **{name: tool.record_artifact(path) for name, path in files.items()},
            **{
                f"criterion:{name}": tool.record_artifact(sample)
                for name, sample in samples.items()
            },
        },
    }
    run_manifest = run_dir / "run.json"
    run_manifest.write_text(json.dumps(run))
    return inventory, manifest, run_manifest, run


class MicrobenchContractTests(unittest.TestCase):
    @unittest.skipUnless(sys.platform == "darwin", "requires the Apple developer toolchain")
    def test_cpp_build_inputs_seal_effective_xcode_tools_not_dispatcher_shims(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "cpp-source"
            run_dir = root / "run"
            source.mkdir()
            run_dir.mkdir()

            environment = tool.cpp_build_environment(source, run_dir)
            developer_dir = pathlib.Path(
                subprocess.run(
                    ["/usr/bin/xcode-select", "-p"],
                    check=True,
                    capture_output=True,
                    text=True,
                ).stdout.strip()
            ).resolve()
            xcrun_environment = {
                "DEVELOPER_DIR": str(developer_dir),
                "PATH": "/usr/bin:/bin:/usr/sbin:/sbin",
            }

            def selected_tool(name: str) -> pathlib.Path:
                command = ["/usr/bin/xcrun"]
                if name in {"metal", "metallib"}:
                    command.extend(["--sdk", "macosx"])
                command.extend(["--find", name])
                return pathlib.Path(
                    subprocess.run(
                        command,
                        check=True,
                        capture_output=True,
                        text=True,
                        env=xcrun_environment,
                    ).stdout.strip()
                )

            selected_sdk = pathlib.Path(
                subprocess.run(
                    ["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-path"],
                    check=True,
                    capture_output=True,
                    text=True,
                    env=xcrun_environment,
                ).stdout.strip()
            ).resolve()

            inputs = tool.write_cpp_build_inputs(
                source,
                run_dir,
                environment,
                [str(source / "build" / "build_rive.sh"), "release", "--", "bench"],
            )
            document = json.loads(inputs.read_text())

            self.assertEqual(environment["DEVELOPER_DIR"], str(developer_dir))
            self.assertEqual(environment["CC"], str(selected_tool("clang")))
            self.assertEqual(environment["CXX"], str(selected_tool("clang++")))
            self.assertEqual(pathlib.Path(environment["CXX"]).name, "clang++")
            self.assertEqual(environment["SDKROOT"], str(selected_sdk))
            self.assertEqual(document["macos_sdk"]["path"], str(selected_sdk))
            self.assertEqual(document["tools"]["make"]["path"], str(selected_tool("make")))
            self.assertEqual(document["tools"]["metal"]["path"], str(selected_tool("metal")))
            self.assertEqual(
                document["tools"]["metallib"]["path"], str(selected_tool("metallib"))
            )
            self.assertNotEqual(document["tools"]["cc"]["path"], "/usr/bin/cc")
            self.assertNotEqual(document["tools"]["make"]["path"], "/usr/bin/make")

    def test_benchmark_content_identity_ignores_only_evidence_docs(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
            subprocess.run(
                ["git", "-C", str(root), "config", "user.email", "test@example.com"],
                check=True,
            )
            (root / "benchmark.rs").write_text("measured content\n")
            subprocess.run(["git", "-C", str(root), "add", "benchmark.rs"], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "measured"], check=True)
            measured = tool.benchmark_content_identity(root)

            evidence = root / "docs" / "evidence" / "run.md"
            evidence.parent.mkdir(parents=True)
            evidence.write_text("results\n")
            subprocess.run(["git", "-C", str(root), "add", str(evidence)], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "evidence"], check=True)

            self.assertEqual(tool.benchmark_content_identity(root), measured)

    def test_benchmark_content_identity_changes_with_benchmark_content(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
            subprocess.run(
                ["git", "-C", str(root), "config", "user.email", "test@example.com"],
                check=True,
            )
            source = root / "benchmark.rs"
            source.write_text("measured content\n")
            subprocess.run(["git", "-C", str(root), "add", "benchmark.rs"], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "measured"], check=True)
            measured = tool.benchmark_content_identity(root)

            source.write_text("changed benchmark content\n")
            subprocess.run(["git", "-C", str(root), "add", "benchmark.rs"], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "changed"], check=True)

            self.assertNotEqual(tool.benchmark_content_identity(root), measured)

    def test_load_run_accepts_an_evidence_only_descendant(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)
            evidence = root / "docs" / "evidence" / "run.md"
            evidence.parent.mkdir(parents=True)
            evidence.write_text("results\n")
            subprocess.run(["git", "-C", str(root), "add", str(evidence)], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "evidence"], check=True)

            self.assertEqual(
                tool.load_run(root, manifest, run_manifest, inventory)["repo_revision"],
                run["repo_revision"],
            )

    def test_load_run_rejects_uncommitted_benchmark_content(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, _ = make_sealed_run_fixture(tool, root)
            (root / "benchmark.rs").write_text("dirty benchmark content\n")

            with self.assertRaisesRegex(tool.ContractError, "uncommitted benchmark content"):
                tool.load_run(root, manifest, run_manifest, inventory)

    def test_load_run_rejects_wrong_schema_and_non_exact_artifact_sets(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, original = make_sealed_run_fixture(tool, root)
            fixtures = []
            wrong_schema = copy.deepcopy(original)
            wrong_schema["schema"] = "nuxie-upstream-microbench-run-v3"
            fixtures.append((wrong_schema, "unsupported benchmark run schema"))
            missing = copy.deepcopy(original)
            del missing["artifacts"][f"criterion:{inventory.cases[0].name}"]
            fixtures.append((missing, "artifact set mismatch"))
            extra = copy.deepcopy(original)
            extra["artifacts"]["criterion:Extra"] = copy.deepcopy(
                extra["artifacts"][f"criterion:{inventory.cases[0].name}"]
            )
            fixtures.append((extra, "artifact set mismatch"))

            for run, message in fixtures:
                with self.subTest(message=message):
                    run_manifest.write_text(json.dumps(run))
                    with self.assertRaisesRegex(tool.ContractError, message):
                        tool.load_run(root, manifest, run_manifest, inventory)

    def test_criterion_home_redirection_does_not_redirect_comparison(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)
            redirected = root / "target" / "redirected"
            sample = redirected / inventory.cases[0].name / "new" / "sample.json"
            sample.parent.mkdir(parents=True)
            sample.write_text(json.dumps({"iters": [1.0], "times": [999.0]}))
            run["settings"]["criterion_home"] = str(redirected)
            run_manifest.write_text(json.dumps(run))

            loaded = tool.load_run(root, manifest, run_manifest, inventory)
            self.assertEqual(
                tool.load_sealed_criterion_timings(loaded, inventory),
                {inventory.cases[0].name: 7.0},
            )

    def test_cpp_output_replaced_after_validation_does_not_change_comparison(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)

            loaded = tool.load_run(root, manifest, run_manifest, inventory)
            pathlib.Path(run["artifacts"]["cpp_output"]["path"]).write_text(
                f"999ms {inventory.cases[0].name}\n"
            )

            self.assertEqual(
                tool.load_sealed_cpp_timings(loaded),
                {inventory.cases[0].name: 1_000_000.0},
            )

    def test_criterion_sample_replaced_after_validation_does_not_change_comparison(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)

            loaded = tool.load_run(root, manifest, run_manifest, inventory)
            sample = pathlib.Path(
                run["artifacts"][f"criterion:{inventory.cases[0].name}"]["path"]
            )
            sample.write_text(json.dumps({"iters": [1.0], "times": [999.0]}))

            self.assertEqual(
                tool.load_sealed_criterion_timings(loaded, inventory),
                {inventory.cases[0].name: 7.0},
            )

    def test_load_run_rejects_mixed_run_criterion_sample_paths(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(
                tool, root, case_count=2
            )
            mixed_case = inventory.cases[1]
            key = f"criterion:{mixed_case.name}"
            mixed = (
                root
                / "target"
                / "other"
                / run["run_id"]
                / mixed_case.name
                / "new"
                / "sample.json"
            )
            mixed.parent.mkdir(parents=True)
            mixed.write_text(
                pathlib.Path(run["artifacts"][key]["path"]).read_text()
            )
            run["artifacts"][key] = tool.record_artifact(mixed)
            run_manifest.write_text(json.dumps(run))

            with self.assertRaisesRegex(tool.ContractError, "mixes run namespaces"):
                tool.load_run(root, manifest, run_manifest, inventory)

    def test_inventory_names_match_pinned_upstream_registry_one_for_one(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        with tempfile.TemporaryDirectory() as directory:
            upstream = pathlib.Path(directory)
            for source in {case.source for case in inventory.cases}:
                target = upstream / source
                target.parent.mkdir(parents=True, exist_ok=True)
                names = [case.name for case in inventory.cases if case.source == source]
                target.write_text("\n".join(f"REGISTER_BENCH({name});" for name in names))
            self.assertEqual(
                tool.discover_upstream_cases(upstream, inventory),
                {case.name for case in inventory.cases},
            )

            first_source = upstream / inventory.cases[0].source
            first_source.write_text(first_source.read_text().replace("REGISTER_BENCH", "RENAMED"))
            with self.assertRaisesRegex(tool.ContractError, "upstream registry mismatch"):
                tool.check_upstream_case_contract(upstream, inventory)

    def test_checked_in_datasets_match_declared_hashes_and_shapes(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")

        tool.check_datasets(REPO_ROOT, inventory)
        bbox_datasets = [dataset for dataset in inventory.datasets if dataset.kind == "i32-ltrb"]
        self.assertEqual(
            [(dataset.name, dataset.count) for dataset in bbox_datasets],
            [("paper_bboxes_6_copies", 19_305), ("marty_bboxes_187_copies", 12_377)],
        )

    def test_pinned_draw_capability_requires_raster_ordering(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        with tempfile.TemporaryDirectory() as directory:
            upstream = pathlib.Path(directory)
            for source in {case.source for case in inventory.cases}:
                target = upstream / source
                target.parent.mkdir(parents=True, exist_ok=True)
                names = [case.name for case in inventory.cases if case.source == source]
                target.write_text("\n".join(f"REGISTER_BENCH({name});" for name in names))
            capability = upstream / inventory.draw_capability_source
            capability.parent.mkdir(parents=True, exist_ok=True)
            capability.write_text(
                "m_platformFeatures.supportsRasterOrderingMode = false;\n"
            )
            cases = [
                case._replace(source_sha256=tool.sha256(upstream / case.source))
                for case in inventory.cases
            ]
            fixture = inventory._replace(
                cases=cases,
                draw_capability_source_sha256=tool.sha256(capability),
            )

            with self.assertRaisesRegex(tool.ContractError, "enable RasterOrdering"):
                tool.check_upstream_case_contract(upstream, fixture)

    def test_dataset_check_rejects_modified_bytes(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        dataset = inventory.datasets[0]

        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            target = root / dataset.path
            target.parent.mkdir(parents=True)
            target.write_bytes(b"not the upstream dataset")
            with self.assertRaisesRegex(tool.ContractError, "sha256"):
                tool.check_dataset(root, dataset)

    def test_converted_bbox_bytes_are_the_declared_little_endian_rows(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        dataset = inventory.datasets[0]

        with tempfile.TemporaryDirectory() as directory:
            source = pathlib.Path(directory) / "boxes.hpp"
            source.write_text("{ -1, 2, 3, 4 },\n{ 5, 6, 7, 8 },\n")
            local = dataset._replace(
                source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(), count=2
            )

            content = tool.converted_dataset_content(source, local)

        self.assertEqual(len(content), 32)
        self.assertEqual(content[:16].hex(), "ffffffff020000000300000004000000")

    def test_report_emits_direct_ratios_for_all_equivalent_boundaries(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        cpp = {case.name: index + 1.0 for index, case in enumerate(inventory.cases)}
        rust = {case.name: (index + 1.0) * 2.0 for index, case in enumerate(inventory.cases)}

        table = tool.render_report(inventory, cpp, rust)

        rows = [line for line in table.splitlines() if line.startswith("| `")]
        ratio_rows = [line for line in rows if "2.000x" in line]
        self.assertEqual(len(ratio_rows), 20)
        self.assertTrue(any("| `BuildRawPath` |" in row for row in rows))
        self.assertTrue(any("| `RawPathBounds` |" in row for row in rows))
        self.assertTrue(all("2.000x" in row for row in ratio_rows))
        self.assertNotIn("Directional timings (not ratio-comparable)", table)
        self.assertNotIn("Blocked equivalence", table)

    def test_inventory_rejects_a_directional_label(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        flipped = [
            case._replace(comparison="directional")
            if case.name == "DrawCustomFeathers"
            else case
            for case in inventory.cases
        ]

        with self.assertRaisesRegex(
            tool.ContractError,
            "ratio case names differ|directional case names differ",
        ):
            tool.check_bench_sources(REPO_ROOT, inventory._replace(cases=flipped))

    def test_report_marks_architecture_blockers_without_timing_or_ratio(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        blocked = inventory.cases[0]._replace(
            comparison="blocked",
            equivalence="requires a production backend-neutral logical frame",
        )
        fixture = inventory._replace(cases=[blocked])

        table = tool.render_report(fixture, {blocked.name: 1.0}, {blocked.name: 2.0})

        self.assertIn("## Blocked equivalence", table)
        self.assertIn("requires a production backend-neutral logical frame", table)
        self.assertNotIn("1.000000 ms", table)
        self.assertNotIn("2.000x", table)

    def test_evidence_run_accepts_directional_but_refuses_blocked_cases(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")
        directional = inventory.cases[0]._replace(comparison="directional")
        tool.check_runnable_inventory(inventory._replace(cases=[directional]))

        blocked = inventory.cases[0]._replace(comparison="blocked")
        fixture = inventory._replace(cases=[blocked, *inventory.cases[1:]])

        with self.assertRaisesRegex(tool.ContractError, blocked.name):
            tool.check_runnable_inventory(fixture)

    def test_criterion_uses_per_iteration_minimum_like_upstream_harness(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            criterion = pathlib.Path(directory)
            sample = criterion / "BuildRawPath" / "new" / "sample.json"
            sample.parent.mkdir(parents=True)
            sample.write_text(json.dumps({"iters": [1.0, 2.0, 4.0], "times": [9.0, 12.0, 40.0]}))

            self.assertEqual(tool.load_criterion_minimum(sample), 6.0)

    def test_all_criterion_cases_use_individually_timed_iterations(self):
        tool = load_tool()
        inventory = tool.load_inventory(REPO_ROOT / "microbenchmarks.toml")

        tool.check_bench_sources(REPO_ROOT, inventory)

        for crate in {case.crate for case in inventory.cases}:
            source = (
                REPO_ROOT / "crates" / crate / "benches" / "upstream_microbenchmarks.rs"
            ).read_text()
            self.assertNotIn("bench.iter(", source)
            self.assertIn("iter_individual_minimum", source)

    def test_cpp_benchmark_is_built_in_the_sealed_run_directory(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            upstream = root / "upstream"
            tests = upstream / "tests"
            build = upstream / "build"
            tests.mkdir(parents=True)
            build.mkdir()
            script = build / "build_rive.sh"
            script.write_text(
                "#!/bin/sh\n"
                "set -eu\n"
                'test "$RIVE_CONFIG" = "release"\n'
                'test "$RIVE_PREMAKE_ARGS" = "--with_rive_text --with_rive_layout --with_rive_canvas"\n'
                'test "$RIVE_BUILD_SYSTEM" = "gmake2"\n'
                'test "$RIVE_PREMAKE_TAG" = "v5.0.0-beta7"\n'
                'test -z "${RIVE_OS+x}"\n'
                'test -z "${RIVE_ARCH+x}"\n'
                'test -z "${RIVE_VARIANT+x}"\n'
                'test -z "${DEPENDENCIES+x}"\n'
                'test -z "${PREMAKE_PATH+x}"\n'
                'test -d "$SDKROOT"\n'
                "case \"$RIVE_OUT\" in /*) exit 2;; esac\n"
                "test ! -e \"$RIVE_OUT\"\n"
                "mkdir -p \"$RIVE_OUT\"\n"
                "printf 'pinned binary' > \"$RIVE_OUT/bench\"\n"
                "chmod +x \"$RIVE_OUT/bench\"\n"
            )
            script.chmod(0o755)
            run_dir = root / "run"
            run_dir.mkdir()

            ambient = {
                "RIVE_CONFIG": "debug",
                "RIVE_PREMAKE_ARGS": "--with_rive_scripting",
                "RIVE_BUILD_SYSTEM": "ninja",
                "RIVE_PREMAKE_TAG": "ambient-tag",
                "RIVE_OS": "android",
                "RIVE_ARCH": "wasm",
                "RIVE_VARIANT": "emulator",
                "DEPENDENCIES": "/tmp/ambient-dependencies",
                "DEVELOPER_DIR": "/tmp/ambient-developer-dir",
                "PREMAKE_PATH": "/tmp/ambient-premake",
                "SDKROOT": "/tmp/ambient-sdk",
            }
            with mock.patch.dict(os.environ, ambient):
                binary, log, command, inputs = tool.build_cpp_benchmark(
                    upstream, run_dir
                )

            self.assertEqual(binary, run_dir / "cpp-build" / "bench")
            self.assertEqual(binary.read_text(), "pinned binary")
            self.assertTrue(log.is_file())
            self.assertEqual(command[-3:], ["release", "--", "bench"])
            recorded = json.loads(inputs.read_text())
            self.assertEqual(recorded["environment"]["RIVE_CONFIG"], "release")
            self.assertNotEqual(
                recorded["environment"]["DEVELOPER_DIR"], ambient["DEVELOPER_DIR"]
            )
            self.assertNotEqual(recorded["environment"]["SDKROOT"], ambient["SDKROOT"])
            self.assertNotIn("RIVE_OS", recorded["environment"])
            self.assertNotIn("DEPENDENCIES", recorded["environment"])
            self.assertIn("dependency_trees", recorded)
            self.assertIn("tools", recorded)

    def test_cpp_source_stage_contains_only_the_pinned_commit(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            upstream = root / "upstream"
            subprocess.run(["git", "init", "-q", str(upstream)], check=True)
            subprocess.run(["git", "-C", str(upstream), "config", "user.name", "Test"], check=True)
            subprocess.run(
                ["git", "-C", str(upstream), "config", "user.email", "test@example.com"],
                check=True,
            )
            (upstream / "tracked.txt").write_text("pinned\n")
            subprocess.run(["git", "-C", str(upstream), "add", "."], check=True)
            subprocess.run(["git", "-C", str(upstream), "commit", "-qm", "pinned"], check=True)
            revision = subprocess.run(
                ["git", "-C", str(upstream), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            (upstream / "untracked.txt").write_text("contamination\n")
            run_dir = root / "run"
            run_dir.mkdir()

            source, archive = tool.stage_upstream_source(upstream, revision, run_dir)

            self.assertEqual((source / "tracked.txt").read_text(), "pinned\n")
            self.assertFalse((source / "untracked.txt").exists())
            self.assertTrue(archive.is_file())

    def test_run_manifest_rejects_mixed_or_stale_artifacts(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)
            run["artifacts"]["cpp_output"]["sha256"] = "0" * 64
            run_manifest.write_text(json.dumps(run))
            with self.assertRaisesRegex(tool.ContractError, "artifact hash mismatch"):
                tool.load_run(root, manifest, run_manifest, inventory)

    def test_load_run_rejects_changed_cpp_dependency_tree(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, _ = make_sealed_run_fixture(tool, root)
            dependency = (
                run_manifest.parent
                / "cpp-source"
                / "tests"
                / "dependencies"
                / "ambient.txt"
            )
            dependency.write_text("changed after build\n")

            with self.assertRaisesRegex(tool.ContractError, "dependency tree changed"):
                tool.load_run(root, manifest, run_manifest, inventory)

    def test_load_run_rejects_forged_cpp_tool_identity(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)
            inputs = pathlib.Path(run["artifacts"]["cpp_build_inputs"]["path"])
            document = json.loads(inputs.read_text())
            document["tools"]["cxx"]["sha256"] = "0" * 64
            inputs.write_text(json.dumps(document))
            run["artifacts"]["cpp_build_inputs"] = tool.record_artifact(inputs)
            run_manifest.write_text(json.dumps(run))

            with self.assertRaisesRegex(tool.ContractError, "build tool changed: cxx"):
                tool.load_run(root, manifest, run_manifest, inventory)

    def test_load_run_rejects_forged_developer_directory_identity(self):
        tool = load_tool()
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            inventory, manifest, run_manifest, run = make_sealed_run_fixture(tool, root)
            inputs = pathlib.Path(run["artifacts"]["cpp_build_inputs"]["path"])
            document = json.loads(inputs.read_text())
            document["developer_directory"]["version_plist"]["sha256"] = "0" * 64
            inputs.write_text(json.dumps(document))
            run["artifacts"]["cpp_build_inputs"] = tool.record_artifact(inputs)
            run_manifest.write_text(json.dumps(run))

            with self.assertRaisesRegex(tool.ContractError, "developer directory changed"):
                tool.load_run(root, manifest, run_manifest, inventory)


if __name__ == "__main__":
    unittest.main()
