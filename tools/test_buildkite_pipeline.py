import unittest
import re
import shlex
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class BuildkitePipelineContractTests(unittest.TestCase):
    def test_native_metal_validation_recipe_uses_existing_integration_tests(self) -> None:
        makefile = (REPO_ROOT / "Makefile").read_text()
        recipe = makefile.split("\nrenderer-native-metal-v3:\n", 1)[1].split("\n\n", 1)[0]
        test_targets = []
        for line in recipe.splitlines():
            words = shlex.split(line)
            if "--test" not in words:
                continue
            package = words[words.index("-p") + 1]
            target = words[words.index("--test") + 1]
            test_targets.append(target)
            self.assertTrue(
                (REPO_ROOT / "crates" / package / "tests" / f"{target}.rs").is_file(),
                f"native Metal recipe invokes missing integration target: {package}/{target}",
            )
        self.assertEqual(test_targets, ["native_metal_resource_shaders"])

    def test_all_pipeline_make_targets_exist(self) -> None:
        makefile = (REPO_ROOT / "Makefile").read_text()
        targets = set()
        for match in re.finditer(r"^([A-Za-z0-9_. -]+):(?!=)", makefile, re.MULTILINE):
            targets.update(match.group(1).split())
        pipeline = (REPO_ROOT / ".buildkite/pipeline.yml").read_text()
        called = set()
        for line in pipeline.splitlines():
            if not line.strip().startswith("make "):
                continue
            words = shlex.split(line.strip(), comments=True)
            if words and words[0] == "make":
                called.update(word for word in words[1:]
                              if not word.startswith("-") and "=" not in word)
        self.assertTrue(called, "pipeline must contain checked make commands")
        self.assertEqual(called - targets, set(), "pipeline invokes nonexistent make targets")

    @staticmethod
    def fast_checks_command() -> str:
        pipeline = (REPO_ROOT / ".buildkite" / "pipeline.yml").read_text()
        fast_checks = pipeline.split('label: ":linux: Runtime fast checks"', 1)[1]
        return fast_checks.split("\n  - label:", 1)[0]

    @staticmethod
    def apple_distribution_compile_command() -> str:
        pipeline = (REPO_ROOT / ".buildkite" / "pipeline.yml").read_text()
        apple_compile = pipeline.split('label: ":mac: Apple distribution compile"', 1)[1]
        return apple_compile.split("\n  - label:", 1)[0]

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

    def test_apple_compile_checks_out_pinned_runtime_before_feature_compile(self) -> None:
        apple_compile = self.apple_distribution_compile_command()

        runtime_dir = apple_compile.index('export RIVE_RUNTIME_DIR="$(pwd)/rive-runtime"')
        clone = apple_compile.index("git clone --filter=blob:none")
        pin = apple_compile.index(
            'git -C "$RIVE_RUNTIME_DIR" checkout "$RIVE_RUNTIME_REF"'
        )
        fixtures = apple_compile.index("make fixtures")
        feature_compile = apple_compile.index("make feature-compile-gate-apple")

        self.assertLess(runtime_dir, clone)
        self.assertLess(clone, pin)
        self.assertLess(pin, fixtures)
        self.assertLess(fixtures, feature_compile)


if __name__ == "__main__":
    unittest.main()
