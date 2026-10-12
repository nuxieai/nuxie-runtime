"""Exercise CI cache setup without installing or running a compiler."""

import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from tools.bazel_runtime_build import BazelRuntime
from tools.bazel.pipeline import pipeline


ROOT = Path(__file__).resolve().parents[1]
CACHE_VARIABLES = (
    "BAZELISK_HOME", "NUXIE_BAZEL_OUTPUT_USER_ROOT", "NUXIE_BAZEL_CACHE_DIR",
    "NUXIE_BAZEL_BATCH", "NUXIE_BAZEL_OUTPUT_BASE",
)
SNAPSHOT = "import json,os; print(json.dumps({name:os.environ.get(name) for name in " + repr(CACHE_VARIABLES) + "}))"


def pipeline_commands():
    return {step['label']: step['command'] for step in pipeline({})['steps']}


def workflow_jobs(content):
    content = content.split("\njobs:\n", 1)[1]
    matches = list(re.finditer(r"(?m)^  ([\w-]+):\n", content))
    return {
        match.group(1): content[match.end():matches[index + 1].start() if index + 1 < len(matches) else len(content)]
        for index, match in enumerate(matches)
    }


class BazelCiCacheTests(unittest.TestCase):
    def environment(self, directory, **values):
        return {"HOME": str(directory / "home"), "PATH": "/usr/bin:/bin",
                "PYTHONDONTWRITEBYTECODE": "1", **values}

    def evaluate(self, directory, values=None, repetitions=1):
        command = 'source "$1"\n' * repetitions + 'exec "$2" -B -c "$3"'
        result = subprocess.run(
            ["/bin/bash", "-eu", "-c", command, "cache-fixture",
             str(ROOT / "tools/bazel/ci-cache.sh"), sys.executable, SNAPSHOT],
            cwd=directory, env=self.environment(directory, **(values or {})),
            capture_output=True, text=True, check=True, timeout=10,
        )
        return json.loads(result.stdout)

    def expected(self, directory, agent):
        cache_root = directory / "home/.nuxie-ci/editor-cargo-target" / agent
        return {"BAZELISK_HOME": str(cache_root / "bazelisk"),
                "NUXIE_BAZEL_OUTPUT_USER_ROOT": str(cache_root / "bazel"),
                "NUXIE_BAZEL_CACHE_DIR": str(cache_root / "bazel-shared-cache"),
                "NUXIE_BAZEL_BATCH": "1", "NUXIE_BAZEL_OUTPUT_BASE": None}

    def test_defaults_match_parent_and_sdk_agent_namespace(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            self.assertEqual(self.evaluate(directory), self.expected(directory, "default"))

    def test_agent_name_precedence_sanitization_and_fallback(self):
        cases = (
            ({"RUNNER_NAME": "Mac pool/runner #2"}, "Mac-pool-runner--2"),
            ({"BUILDKITE_AGENT_NAME": "agent:arm64", "RUNNER_NAME": "ignored"}, "agent-arm64"),
            ({"BUILDKITE_AGENT_NAME": "", "RUNNER_NAME": "fallback"}, "fallback"),
            ({"BUILDKITE_AGENT_NAME": "."}, "default"),
            ({"BUILDKITE_AGENT_NAME": ".."}, "default"),
            ({"BUILDKITE_AGENT_NAME": ""}, "default"),
        )
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            for values, agent in cases:
                with self.subTest(values=values):
                    self.assertEqual(self.evaluate(directory, values), self.expected(directory, agent))

    def test_explicit_overrides_and_private_base_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            values = {name: str(directory / (name + " with spaces")) for name in CACHE_VARIABLES}
            values["NUXIE_BAZEL_BATCH"] = "0"
            self.assertEqual(self.evaluate(directory, values), values)

    def test_explicit_empty_overrides_remain_empty(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            values = {name: "" for name in CACHE_VARIABLES}
            self.assertEqual(self.evaluate(directory, values), values)

    def test_repeated_sourcing_is_idempotent(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            values = {"BUILDKITE_AGENT_NAME": "cache-agent"}
            self.assertEqual(self.evaluate(directory, values, 2), self.expected(directory, "cache-agent"))

    def test_frontend_keeps_checkout_configuration_private_and_does_not_pin_a_base(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            selected = self.evaluate(directory, {"BUILDKITE_AGENT_NAME": "shared-agent"})
            launcher = directory / "stub-bazel"
            launcher.write_text("#!" + sys.executable + "\n" + '''import json,os,sys
from pathlib import Path
with open(os.environ["CALLS"], "a") as log:
    log.write(json.dumps({"argv": sys.argv[1:], "cwd": os.getcwd()}) + "\\n")
query = sys.argv[-1]
print("release 9.3.0" if query == "release" else str(Path.cwd() / ("stub-only-" + query)))
''')
            launcher.chmod(0o755)
            calls = directory / "calls.jsonl"
            environments = {name: value for name, value in selected.items() if value is not None}
            environments.update({"NUXIE_BAZEL_BIN": str(launcher), "CALLS": str(calls)})
            configurations = []
            checkouts = [directory / "checkout-a", directory / "checkout-b"]
            for checkout in checkouts:
                checkout.mkdir()
                with patch.dict(os.environ, environments, clear=True):
                    frontend = BazelRuntime(checkout)
                self.assertIn("--output_user_root=" + selected["NUXIE_BAZEL_OUTPUT_USER_ROOT"], frontend.startup)
                self.assertIn("--batch", frontend.startup)
                self.assertFalse(any(argument.startswith("--output_base=") for argument in frontend.startup))
                configuration = checkout / ".bazel-cache.local.bazelrc"
                self.assertIn("--bazelrc=" + str(configuration), frontend.startup)
                options = [shlex.split(line) for line in configuration.read_text().splitlines()]
                self.assertEqual(options, [
                    ["common", "--repository_cache=" + selected["NUXIE_BAZEL_CACHE_DIR"] + "/repositories"],
                    ["common", "--repo_contents_cache=" + selected["NUXIE_BAZEL_CACHE_DIR"] + "/repository-contents"],
                    ["build", "--disk_cache=" + selected["NUXIE_BAZEL_CACHE_DIR"] + "/actions"],
                ])
                configurations.append(configuration)
            self.assertNotEqual(configurations[0], configurations[1])
            records = [json.loads(line) for line in calls.read_text().splitlines()]
            self.assertEqual([record["cwd"] for record in records],
                             [str(checkouts[0])] * 4 + [str(checkouts[1])] * 4)
            self.assertFalse(any(argument.startswith("--output_base=")
                                 for record in records for argument in record["argv"]))

    def test_each_buildkite_lane_exports_cache_environment_before_installing(self):
        commands = pipeline_commands()
        self.assertEqual(len(commands), 6, "all active runtime Buildkite lanes must be exercised")
        for label, command in commands.items():
            with self.subTest(lane=label), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary).resolve()
                tools = directory / "tools/bazel"
                tools.mkdir(parents=True)
                shutil.copy2(ROOT / "tools/bazel/ci-cache.sh", tools / "ci-cache.sh")
                installer = tools / "install.sh"
                installer.write_text("#!" + sys.executable + "\n" + SNAPSHOT + "\n")
                installer.chmod(0o755)
                lines = command.splitlines()
                install = next(index for index, line in enumerate(lines)
                               if shlex.split(line) == ["tools/bazel/install.sh"])
                result = subprocess.run(
                    ["/bin/bash", "-c", "\n".join(lines[:install + 1])], cwd=directory,
                    env=self.environment(directory, BUILDKITE_AGENT_NAME="lane-host"),
                    capture_output=True, text=True, check=True, timeout=10,
                )
                self.assertEqual(json.loads(result.stdout), self.expected(directory, "lane-host"))

    def test_scheduled_github_job_restores_only_shared_caches_before_install(self):
        caller = workflow_jobs((ROOT / ".github/workflows/runtime-differentials.yml").read_text())["runtime-differentials"]
        self.assertRegex(caller, r"(?m)^    uses: \./\.github/workflows/_trusted-macos.yml$")
        self.assertRegex(caller, r"(?m)^      runtime_differentials_only: true$")
        job = workflow_jobs((ROOT / ".github/workflows/_trusted-macos.yml").read_text())["runtime-differentials"]
        steps = re.split(r"(?m)^      - name: ", job)[1:]
        caches = [index for index, step in enumerate(steps) if re.search(r"(?m)^        uses: actions/cache@", step)]
        self.assertEqual(len(caches), 1, "scheduled runtime jobs must restore the shared caches")
        cache = caches[0]
        install = next(index for index, step in enumerate(steps) if re.search(r"(?m)^        run: tools/bazel/install.sh$", step))
        self.assertLess(cache, install)
        paths = re.search(r"(?m)^          path: \|\n((?:            .*\n)+)", steps[cache])
        self.assertIsNotNone(paths)
        self.assertEqual([line.strip() for line in paths.group(1).splitlines()],
                         ["~/.cache/nuxie/bazel", "~/.cache/nuxie-tools/bazelisk",
                          "~/.cache/bazelisk", "~/Library/Caches/bazelisk"])
        self.assertRegex(steps[install], r"(?m)^        working-directory: \$\{\{ github.workspace \}\}/rive-rust$")


if __name__ == "__main__":
    unittest.main()
