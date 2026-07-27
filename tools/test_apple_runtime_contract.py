import re
import unittest
from pathlib import Path

from tools.apple_runtime_contract import ContractError
from tools.apple_runtime_contract import validate_metadata
from tools.apple_runtime_contract import validate_symbols


REPO_ROOT = Path(__file__).resolve().parents[1]


def valid_metadata() -> dict[str, object]:
    revision = "a" * 40
    return {
        "schemaVersion": 2,
        "runtimeVersion": "0.2.0",
        "sourceRevision": revision,
        "runtimeIdentity": f"0.2.0@{revision}",
        "contractFingerprint": "b" * 64,
        "luaurVersion": "0.1.8",
        "buildProfile": "release-apple",
        "rustToolchain": "1.94.1",
        "xcodeVersion": "26.2",
        "xcodeBuild": "17C52",
        "iphoneOSSDKVersion": "26.2",
        "iphoneOSSDKBuild": "23C53",
        "iphoneSimulatorSDKVersion": "26.2",
        "iphoneSimulatorSDKBuild": "23C53",
        "minimumIOSVersion": "15.0",
        "thirdPartyNoticesPath": (
            "NuxieRuntime.xcframework/THIRD_PARTY_NOTICES.md"
        ),
        "swiftPackageChecksum": "c" * 64,
    }


class MetadataTests(unittest.TestCase):
    def test_exact_schema_two_metadata_passes(self) -> None:
        validate_metadata(valid_metadata())

    def test_wrong_schema_or_hidden_abi_field_fails(self) -> None:
        wrong_schema = valid_metadata()
        wrong_schema["schemaVersion"] = 1
        with self.assertRaisesRegex(ContractError, "schemaVersion"):
            validate_metadata(wrong_schema)

        hidden_abi = valid_metadata()
        hidden_abi["abiMajor"] = 1
        with self.assertRaisesRegex(ContractError, "extra=.*abiMajor"):
            validate_metadata(hidden_abi)

    def test_identity_fingerprint_and_checksum_disagreement_fails(self) -> None:
        wrong_identity = valid_metadata()
        wrong_identity["runtimeIdentity"] = "0.2.0@" + "d" * 40
        with self.assertRaisesRegex(ContractError, "runtimeIdentity"):
            validate_metadata(wrong_identity)

        malformed_fingerprint = valid_metadata()
        malformed_fingerprint["contractFingerprint"] = "not-a-digest"
        with self.assertRaisesRegex(ContractError, "contractFingerprint"):
            validate_metadata(malformed_fingerprint)

        malformed_checksum = valid_metadata()
        malformed_checksum["swiftPackageChecksum"] = "not-a-digest"
        with self.assertRaisesRegex(ContractError, "swiftPackageChecksum"):
            validate_metadata(malformed_checksum)

    def test_unverifiable_source_revision_fails(self) -> None:
        metadata = valid_metadata()
        metadata["sourceRevision"] = "unknown"
        metadata["runtimeIdentity"] = "0.2.0@unknown"
        with self.assertRaisesRegex(ContractError, "sourceRevision"):
            validate_metadata(metadata)


class SymbolTests(unittest.TestCase):
    HEADER = """
NuxStatus nux_runtime_bind(const uint8_t *version,
                           uint64_t version_len);
void nux_operation_result_free(struct NuxOperationResult *result);
"""
    SYMBOLS = "_nux_operation_result_free\n_nux_runtime_bind\n_rust_eh_personality\n"

    def test_exact_public_symbol_set_passes(self) -> None:
        validate_symbols(self.HEADER, self.SYMBOLS)

    def test_missing_or_extra_public_symbol_fails(self) -> None:
        with self.assertRaisesRegex(ContractError, "missing=.*nux_runtime_bind"):
            validate_symbols(self.HEADER, "_nux_operation_result_free\n")
        with self.assertRaisesRegex(ContractError, "extra=.*nux_unpublished"):
            validate_symbols(
                self.HEADER,
                self.SYMBOLS + "_nux_unpublished\n",
            )

    def test_removed_client_abi_identifier_fails_even_when_exported(self) -> None:
        header = self.HEADER + (
            "\nuint16_t nux_runtime_abi_minor(void);\n"
            "#define NUX_RUNTIME_ABI_MINOR 6\n"
        )
        symbols = self.SYMBOLS + "_nux_runtime_abi_minor\n"
        with self.assertRaisesRegex(ContractError, "removed client ABI identifiers"):
            validate_symbols(header, symbols)


class ReleaseWorkflowSourcePolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.release_workflow = (
            REPO_ROOT / ".github/workflows/apple-runtime-release.yml"
        ).read_text()
        self.trusted_macos_workflow = (
            REPO_ROOT / ".github/workflows/_trusted-macos.yml"
        ).read_text()
        self.release_documentation = (
            REPO_ROOT / "docs/apple-runtime-release.md"
        ).read_text()
        self.identity_verifier = (
            REPO_ROOT / "tools/test-apple-runtime-build-identity.sh"
        ).read_text()
        self.makefile = (REPO_ROOT / "Makefile").read_text()

    def test_manual_release_dispatch_checks_out_the_supplied_tag(self) -> None:
        self.assertRegex(
            self.release_workflow,
            re.compile(
                r"workflow_dispatch:\s*\n"
                r"\s+inputs:\s*\n"
                r"\s+release_tag:\s*\n"
                r"(?:\s+.*\n){0,5}"
                r"\s+required:\s+true\s*\n"
                r"\s+type:\s+string"
            ),
        )
        self.assertIn(
            "ref: ${{ github.event_name == 'workflow_dispatch' "
            "&& format('refs/tags/{0}', inputs.release_tag) || github.ref }}",
            self.release_workflow,
        )
        self.assertNotIn("ref: ${{ github.ref }}", self.release_workflow)
        self.assertIn(
            'if [[ "${GITHUB_REF}" != "refs/heads/main" ]]',
            self.release_workflow,
        )

    def test_both_triggers_share_one_exact_release_tag(self) -> None:
        normalized_tag = (
            "${{ github.event_name == 'workflow_dispatch' "
            "&& inputs.release_tag || github.ref_name }}"
        )
        self.assertIn(
            'tags:\n      - "apple-runtime-v*"',
            self.release_workflow,
        )
        self.assertIn(
            f"group: apple-runtime-release-{normalized_tag}",
            self.release_workflow,
        )
        self.assertIn(
            f"NUX_RUNTIME_RELEASE_TAG: {normalized_tag}",
            self.release_workflow,
        )
        self.assertIn(
            'release_tag="${NUX_RUNTIME_RELEASE_TAG}"',
            self.release_workflow,
        )
        self.assertIn(
            'if [[ "${release_tag}" != "${expected_tag}" ]]',
            self.release_workflow,
        )
        self.assertIn(
            'tagged_revision="$(git rev-list -n 1 "refs/tags/${release_tag}")"',
            self.release_workflow,
        )
        self.assertIn(
            'test "${source_revision}" = "${tagged_revision}"',
            self.release_workflow,
        )
        self.assertIn(
            'git merge-base --is-ancestor "${source_revision}" '
            "refs/remotes/origin/main",
            self.release_workflow,
        )

    def test_self_hosted_xcode_pin_is_current_without_changing_fallback(self) -> None:
        self.assertIn('NUX_APPLE_XCODE_VERSION: "26.2"', self.release_workflow)
        self.assertIn('NUX_APPLE_XCODE_BUILD: "17C52"', self.release_workflow)
        self.assertNotIn('NUX_APPLE_XCODE_VERSION: "26.6"', self.release_workflow)
        self.assertNotIn('NUX_APPLE_XCODE_BUILD: "17F113"', self.release_workflow)

        hosted = (
            "inputs.force_hosted || (github.event_name == 'pull_request' "
            "&& github.event.pull_request.head.repo.full_name != github.repository)"
        )
        version_mapping = (
            "NUX_APPLE_XCODE_VERSION: "
            f"${{{{ ({hosted}) && '15.4' || '26.2' }}}}"
        )
        build_mapping = (
            "NUX_APPLE_XCODE_BUILD: "
            f"${{{{ ({hosted}) && '15F31d' || '17C52' }}}}"
        )
        self.assertEqual(self.trusted_macos_workflow.count(version_mapping), 2)
        self.assertEqual(self.trusted_macos_workflow.count(build_mapping), 2)
        self.assertNotIn("'26.6'", self.trusted_macos_workflow)
        self.assertNotIn("'17F113'", self.trusted_macos_workflow)

    def test_release_requires_the_single_qualified_runner_label(self) -> None:
        documentation_words = " ".join(self.release_documentation.split())
        runner_selectors = re.findall(
            r"^\s+runs-on:\s*(.+)$",
            self.release_workflow,
            re.MULTILINE,
        )
        self.assertEqual(
            runner_selectors,
            [
                "[self-hosted, macOS, ARM64, nuxie-signoff, "
                "nuxie-release]"
            ],
        )
        self.assertIn(
            "custom `nuxie-release` label to exactly one signed macOS "
            "release host",
            documentation_words,
        )
        self.assertIn(
            "runner-group workflow and runner-label policies before a job "
            "starts",
            documentation_words,
        )

    def test_identity_verifier_is_portable_and_ignores_cargo_ui_words(self) -> None:
        self.assertIn('if [[ "$#" -gt 1 ]]', self.identity_verifier)
        self.assertIn(
            'repo_root="$(cd -P "${1}" && pwd -P)"',
            self.identity_verifier,
        )
        self.assertIn(
            '(cd "${repo_root}" && env -u NUX_RUNTIME_SOURCE_REVISION',
            self.identity_verifier,
        )
        self.assertIn("expected_revision()", self.identity_verifier)
        self.assertNotIn("Fresh nux-apple-runtime", self.identity_verifier)
        self.assertNotIn("Dirty nux-apple-runtime", self.identity_verifier)

    def test_release_runs_the_reviewed_identity_verifier_and_all_other_gates(
        self,
    ) -> None:
        reviewed_verifier = (
            'reviewed_identity_verifier="$(mktemp "${RUNNER_TEMP}/'
            'test-apple-runtime-build-identity.XXXXXX")"'
        )
        cleanup_verifier = (
            """trap 'rm -f -- "${reviewed_identity_verifier}"' EXIT"""
        )
        peel_workflow_revision = (
            'workflow_revision="$(git --no-replace-objects rev-parse '
            '--verify "${GITHUB_WORKFLOW_SHA}^{commit}")"'
        )
        materialize_verifier = (
            'git --no-replace-objects show "${workflow_revision}:'
            'tools/test-apple-runtime-build-identity.sh" '
            '> "${reviewed_identity_verifier}"'
        )
        self.assertEqual(self.release_workflow.count(reviewed_verifier), 1)
        self.assertEqual(self.release_workflow.count(cleanup_verifier), 1)
        self.assertEqual(
            self.release_workflow.count(peel_workflow_revision),
            1,
        )
        self.assertEqual(
            self.release_workflow.count(
                'if [[ "${workflow_revision}" != "${GITHUB_WORKFLOW_SHA}" ]]'
            ),
            1,
        )
        self.assertEqual(self.release_workflow.count(materialize_verifier), 1)
        self.assertEqual(
            self.release_workflow.count(
                'chmod 0700 "${reviewed_identity_verifier}"'
            ),
            1,
        )
        self.assertNotIn(
            'git show "${GITHUB_WORKFLOW_SHA}:',
            self.release_workflow,
        )

        ordered_gates = [
            "make apple-runtime-contract-test",
            'bash -x "${reviewed_identity_verifier}" "${GITHUB_WORKSPACE}"',
            "make apple-runtime-header-smoke",
            "make apple-runtime-release-panic-smoke",
            (
                "cargo test --locked -p nux-apple-runtime "
                "--features apple-product"
            ),
            (
                "cargo clippy --locked -p nux-apple-runtime --lib "
                "--no-default-features --features apple-product "
                "--no-deps --quiet -- -D warnings"
            ),
        ]
        for gate in ordered_gates:
            self.assertEqual(self.release_workflow.count(gate), 1)
        self.assertEqual(
            sorted(self.release_workflow.index(gate) for gate in ordered_gates),
            [self.release_workflow.index(gate) for gate in ordered_gates],
        )
        cleanliness_assertion = "assert_clean_tagged_worktree"
        cleanliness_calls = [
            match.start()
            for match in re.finditer(
                rf"^          {cleanliness_assertion}$",
                self.release_workflow,
                re.MULTILINE,
            )
        ]
        self.assertEqual(len(cleanliness_calls), 2)
        self.assertEqual(
            self.release_workflow.count(
                'actual_head="$(git rev-parse --verify HEAD)"'
            ),
            1,
        )
        self.assertEqual(
            self.release_workflow.count(
                'if [[ "${actual_head}" != '
                '"${NUX_RUNTIME_SOURCE_REVISION}" ]]'
            ),
            1,
        )
        self.assertEqual(
            self.release_workflow.count(
                'worktree_status="$(git status --porcelain '
                '--untracked-files=all)"'
            ),
            1,
        )
        self.assertEqual(
            self.release_workflow.count(
                'if [[ -n "${worktree_status}" ]]'
            ),
            1,
        )
        self.assertLess(
            self.release_workflow.index(ordered_gates[0]),
            cleanliness_calls[0],
        )
        self.assertLess(
            cleanliness_calls[0],
            self.release_workflow.index(ordered_gates[1]),
        )
        self.assertLess(
            self.release_workflow.index(ordered_gates[1]),
            cleanliness_calls[1],
        )
        self.assertLess(
            cleanliness_calls[1],
            self.release_workflow.index(ordered_gates[2]),
        )
        self.assertNotIn("git diff --exit-code -- .", self.release_workflow)
        self.assertNotIn(
            "bash -x tools/test-apple-runtime-build-identity.sh",
            self.release_workflow,
        )
        self.assertNotIn(
            "make apple-runtime-build-identity-test",
            self.release_workflow,
        )
        self.assertNotIn("make apple-runtime-check", self.release_workflow)

    def test_ordinary_apple_runtime_check_target_is_unchanged(self) -> None:
        self.assertIn(
            "apple-runtime-check: apple-runtime-contract-test "
            "apple-runtime-build-identity-test apple-runtime-header-smoke "
            "apple-runtime-release-panic-smoke\n"
            "\tcargo test --locked -p nux-apple-runtime "
            "--features apple-product\n"
            "\tcargo clippy --locked -p nux-apple-runtime --lib "
            "--no-default-features --features apple-product --no-deps "
            "--quiet -- -D warnings",
            self.makefile,
        )

    def test_release_documents_the_portable_retry_boundary(self) -> None:
        documentation_words = " ".join(self.release_documentation.split())
        self.assertIn("GITHUB_WORKFLOW_SHA", self.release_documentation)
        self.assertIn(
            "reviewed verifier is materialized at an unpredictable `mktemp` "
            "path under `RUNNER_TEMP` and removed when the step exits",
            documentation_words,
        )
        self.assertIn(
            "runs with the tagged workspace passed as its explicit repository "
            "root",
            documentation_words,
        )
        self.assertIn(
            "remaining `apple-runtime-check` component gates run explicitly "
            "in their original order",
            documentation_words,
        )
        self.assertIn(
            "peels `GITHUB_WORKFLOW_SHA` to an exact commit",
            documentation_words,
        )
        self.assertIn(
            "`git --no-replace-objects show`",
            documentation_words,
        )
        self.assertIn(
            "local replace refs cannot redirect the immutable workflow-source "
            "lookup",
            documentation_words,
        )
        self.assertIn(
            "whole worktree, including untracked files, is clean before and "
            "after the verifier",
            documentation_words,
        )
    def test_apple_jobs_force_the_exact_rustup_toolchain_onto_path(self) -> None:
        toolchain_bin = (
            'toolchain_bin="$(dirname "$(rustup which '
            '--toolchain 1.94.1 cargo)")"'
        )
        github_path = 'echo "${toolchain_bin}" >> "${GITHUB_PATH}"'
        direct_path = 'export PATH="${toolchain_bin}:${PATH}"'
        cargo_version = (
            'test "$(cargo --version | awk \'{ print $2 }\')" = "1.94.1"'
        )
        rustc_version = (
            'test "$(rustc --version | awk \'{ print $2 }\')" = "1.94.1"'
        )

        for source, expected_count in (
            (self.release_workflow, 1),
            (self.trusted_macos_workflow, 2),
        ):
            self.assertEqual(source.count(toolchain_bin), expected_count)
            self.assertEqual(source.count(github_path), expected_count)
            self.assertEqual(source.count(direct_path), expected_count)
            self.assertEqual(source.count(cargo_version), expected_count)
            self.assertEqual(source.count(rustc_version), expected_count)

    def test_release_documents_both_required_runner_policy_refs(self) -> None:
        self.assertIn(
            "nuxieai/nuxie-runtime/.github/workflows/"
            "apple-runtime-release.yml@refs/tags/apple-runtime-v0.2.0",
            self.release_documentation,
        )
        self.assertIn(
            "nuxieai/nuxie-runtime/.github/workflows/"
            "apple-runtime-release.yml@refs/heads/main",
            self.release_documentation,
        )
        self.assertIn(
            "evaluates the runner-group workflow and runner-label policies "
            "before a",
            self.release_documentation,
        )
        self.assertIn(
            "future tag-push release requires adding that new exact tag ref",
            self.release_documentation,
        )

    def test_release_documents_environment_ref_and_token_prerequisites(self) -> None:
        self.assertIn(
            "deployment branch and tag policy must allow both",
            self.release_documentation,
        )
        self.assertIn(
            "tag pattern `apple-runtime-v*`",
            self.release_documentation,
        )
        self.assertIn(
            "branch `main`",
            self.release_documentation,
        )
        self.assertIn(
            "**Administration: read** permission",
            self.release_documentation,
        )
        self.assertIn(
            "environment deployment-ref policy before release steps run",
            self.release_documentation,
        )


if __name__ == "__main__":
    unittest.main()
