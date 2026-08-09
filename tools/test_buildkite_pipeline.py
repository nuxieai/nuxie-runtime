import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class BuildkitePipelineContractTests(unittest.TestCase):
    @staticmethod
    def fast_checks_command() -> str:
        pipeline = (REPO_ROOT / ".buildkite" / "pipeline.yml").read_text()
        fast_checks = pipeline.split('label: ":linux: Runtime fast checks"', 1)[1]
        return fast_checks.split("\n  - label:", 1)[0]

    def test_fast_checks_prepare_fixtures_before_portable_feature_compile(self) -> None:
        fast_checks = self.fast_checks_command()

        fixture_bootstrap = fast_checks.index("make fixtures")
        portable_compile = fast_checks.index("feature-compile-gate-portable")

        self.assertLess(fixture_bootstrap, portable_compile)

    def test_fast_checks_checkout_pinned_runtime_before_fixtures(self) -> None:
        fast_checks = self.fast_checks_command()

        runtime_dir = fast_checks.index('export RIVE_RUNTIME_DIR="$(pwd)/rive-runtime"')
        clone = fast_checks.index("git clone --filter=blob:none")
        pin = fast_checks.index('git -C "$RIVE_RUNTIME_DIR" checkout "$RIVE_RUNTIME_REF"')
        fixtures = fast_checks.index("make fixtures")

        self.assertLess(runtime_dir, clone)
        self.assertLess(clone, pin)
        self.assertLess(pin, fixtures)


if __name__ == "__main__":
    unittest.main()
