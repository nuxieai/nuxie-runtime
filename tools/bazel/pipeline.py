#!/usr/bin/env python3
"""Assign the runtime's existing platform checks to guarded physical hosts."""
import json
import os
from pathlib import Path
try:
    from .cache_policy import cache_environment
except ImportError:
    from cache_policy import cache_environment

ROOT = Path(__file__).resolve().parents[2]


def pipeline(environment=None):
    environment = os.environ if environment is None else environment
    version = environment.get('NUXIE_CI_RUNNER_POLICY_VERSION', '1')
    split = environment.get('NUXIE_SDK_SPLIT_LANES', '0')
    topology = environment.get('NUXIE_CI_RUNNER_TOPOLOGY', 'final')
    if version not in ('', '1', '2') or split not in ('0', '1') or topology not in ('final', 'fallback'):
        raise ValueError('Unsupported runner policy or SDK split setting')
    if split == '1' and version != '2':
        raise ValueError('SDK split requires installed runner policy v2')
    if version == '2' and topology == 'fallback':
        raise ValueError('SDK native confidence requires the final 14 GiB Linux topology')
    document = json.loads((ROOT / '.buildkite/lanes.json').read_text())
    if split == '1':
        document['steps'].append({
            'label': ':linux: Android C ABI confidence',
            'if': 'build.source == "schedule"',
            'command': '\n'.join([
                'set -euo pipefail',
                'source tools/bazel/ci-cache.sh',
                'tools/bazel/install.sh',
                'unset CARGO_INCREMENTAL',
                'make nux-capi-android-contract-test nux-capi-android',
            ]) + '\n',
            'agents': {'queue': 'nuxie-pr', 'nuxie-platform': 'linux'},
            'timeout_in_minutes': 90,
        })
    if version == '2':
        for step in document['steps']:
            platform = step['agents']['nuxie-platform']
            step['agents']['nuxie-resources'] = 'mac-64g-v2' if platform == 'macos' else 'linux-14g-v2'
            step['concurrency'] = 1
            step['concurrency_group'] = 'nuxie/physical/mac-m2' if platform == 'macos' else 'nuxie/physical/proxmox-heavy'
            step['concurrency_method'] = 'eager'
            profile = 'macos-apple' if platform == 'macos' else 'linux-native'
            step['env'] = {
                'NUXIE_CI_RUNNER_POLICY_VERSION': '2',
                'NUXIE_CI_RUNNER_PROFILE': profile,
                **cache_environment(environment, profile),
            }
            step['artifact_paths'] = [
                'output/pipeline/**/*', 'bazel-testlogs/**/*',
                'target/*.json', 'target/*.md', 'target/runtime-differentials/**/*',
                'target/renderer-*/**/*.json', 'target/renderer-*/**/*.png',
                'target/renderer-*/**/*.txt', 'target/renderer-*/**/*.log',
            ]
            if step['label'] == ':linux: Android C ABI confidence':
                step['artifact_paths'].append('target/nux-capi-android/NuxieRuntimeAndroid*')
    return document


if __name__ == '__main__':
    print(json.dumps(pipeline()))
