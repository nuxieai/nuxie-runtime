"""Optional cache location override; outputs remain checkout-specific."""

import os
from pathlib import Path
import shlex
import subprocess
import tempfile


def guarded_environment():
    if os.environ.get("NUXIE_CI_RUNNER_POLICY_VERSION") != "2":
        return
    helper = Path(__file__).resolve().parent / "bazel/runner-policy.sh"
    output = subprocess.check_output([
        "/bin/bash", "-eu", "-c",
        'source "$1"; nuxie_runner_policy_verify; exec /usr/bin/python3 -I "$NUXIE_RUNNER_POLICY_HELPER" environment',
        "runtime-policy", str(helper),
    ], text=True, timeout=5)
    selected = {}
    for line in output.splitlines():
        words = shlex.split(line)
        if len(words) != 2 or words[0] != "export" or "=" not in words[1]:
            raise ValueError("Invalid installed runner environment")
        key, value = words[1].split("=", 1)
        if not key.startswith(("NUXIE_", "CARGO_", "BAZELISK_")):
            raise ValueError("Unexpected installed runner environment")
        selected[key] = value
    os.environ.update(selected)


def startup_options(root):
    guarded_environment()
    value = os.environ.get("NUXIE_BAZEL_CACHE_DIR")
    if value is None:
        return []
    directory = Path(value)
    if not value or not directory.is_absolute() or any(c in value for c in "\0\r\n"):
        raise ValueError("NUXIE_BAZEL_CACHE_DIR must be an absolute path on one line")
    configuration = Path(root) / ".bazel-cache.local.bazelrc"
    contents = "".join(
        kind + " --" + flag + "=" + shlex.quote(str(directory / child)) + "\n"
        for kind, flag, child in (
            ("common", "repository_cache", "repositories"),
            ("common", "repo_contents_cache", "repository-contents"),
            ("build", "disk_cache", "actions"),
        )
    )
    if not configuration.is_file() or configuration.read_text() != contents:
        descriptor, temporary = tempfile.mkstemp(prefix=".bazel-cache.", suffix=".tmp", dir=root)
        try:
            with os.fdopen(descriptor, "w") as file:
                file.write(contents)
            os.replace(temporary, configuration)
        finally:
            Path(temporary).unlink(missing_ok=True)
    return ["--bazelrc=" + str(configuration)]


if __name__ == "__main__":
    import sys

    try:
        print("\n".join(startup_options(Path(__file__).resolve().parents[1])))
    except ValueError as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
