"""Run workflow installer steps against an isolated GitHub checkout layout."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
JOBS = {
    "ci.yml": ("parity-runtime-evidence", "perf-gate", "backend-port-browser-artifacts"),
    "_trusted-macos.yml": ("golden-compare", "runtime-differentials"),
}


def workflow_jobs(path):
    # These run steps use scalar commands and working-directory fields. Read
    # only the job/step blocks needed to reproduce their runner filesystem.
    content = path.read_text().split("\njobs:\n", 1)[1]
    matches = list(re.finditer(r"(?m)^  ([\w-]+):\n", content))
    result = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(content)
        result[match.group(1)] = content[match.end():end]
    return result


class BazelCiInstallTest(unittest.TestCase):
    def test_installer_runs_in_each_workflows_runtime_checkout(self):
        for filename, required_jobs in JOBS.items():
            jobs = workflow_jobs(ROOT / ".github/workflows" / filename)
            for name in required_jobs:
                with self.subTest(workflow=filename, job=name), tempfile.TemporaryDirectory() as directory:
                    workspace = Path(directory) / "workspace"
                    workspace.mkdir()
                    steps = re.split(r"(?m)^      - name: ", jobs[name])[1:]
                    checkout = next(step for step in steps if "uses: actions/checkout@" in step)
                    checkout_path = re.search(r"(?m)^          path: (.+)$", checkout)
                    runtime = workspace / (checkout_path.group(1) if checkout_path else ".")
                    (runtime / "tools/bazel").mkdir(parents=True)
                    shutil.copy2(ROOT / "tools/bazel/install.sh", runtime / "tools/bazel/install.sh")
                    install = next((step for step in steps if step.startswith("Install Bazelisk\n")), None)
                    self.assertIsNotNone(install, "Bazel-backed lane must install Bazelisk")
                    command = re.search(r"(?m)^        run: (.+)$", install)
                    working_directory = re.search(r"(?m)^        working-directory: (.+)$", install)
                    cwd = workspace
                    if working_directory:
                        cwd = Path(working_directory.group(1).replace("${{ github.workspace }}", str(workspace)))
                    bin_directory = Path(directory) / "bin"
                    bin_directory.mkdir()
                    npm = bin_directory / "npm"
                    npm.write_text("""#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
  case "$1" in
    --prefix) destination=$2; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$destination/node_modules/.bin"
printf '#!/bin/sh\\nexit 0\\n' > "$destination/node_modules/.bin/bazelisk"
chmod +x "$destination/node_modules/.bin/bazelisk"
""")
                    npm.chmod(0o755)
                    result = subprocess.run(
                        ["/bin/bash", "-c", command.group(1)], cwd=cwd,
                        env={"PATH": str(bin_directory) + os.pathsep + "/usr/bin:/bin"},
                        capture_output=True, text=True, timeout=10,
                    )
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertTrue((runtime / "target/bazel-tools/node_modules/.bin/bazelisk").is_file())


if __name__ == "__main__":
    unittest.main()
