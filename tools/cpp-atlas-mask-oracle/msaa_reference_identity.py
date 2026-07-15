"""Canonical, case-local provenance identities for Dawn MSAA references."""

import hashlib
import json


PROVENANCE_VERSION = "2"
CASE_IDENTITY_SCHEMA_VERSION = 1
ADAPTER_KEYS = (
    "adapter_vendor",
    "adapter_architecture",
    "adapter_device",
    "adapter_description",
    "adapter_vendor_id",
    "adapter_device_id",
)


def case_identity_payload(case: dict, provenance: dict[str, str]) -> dict:
    """Return the complete case-local contract hashed into an identity."""
    return {
        "schema_version": CASE_IDENTITY_SCHEMA_VERSION,
        "case_id": case["id"],
        "source_kind": case.get("profile", "gm"),
        "source_path": case["stream"],
        "source": case.get("source", ""),
        "scene": case.get("scene", ""),
        "artboard": case.get("artboard", ""),
        "stream_sha256": case["sha256"],
        "frame_width": case["width"],
        "frame_height": case["height"],
        "frame_index": case.get("frame", 0),
        "sample_seconds": case.get("sample_seconds", ""),
        "command_selector": dict(sorted(case["counts"].items())),
        "clear_color": case["clear_color"],
        "mode": "msaa",
        "artifact_format": "RIVEABL/v1 RGBA8",
        "sample_count": provenance["sample_count"],
        "backend": provenance["backend"],
        "renderer_implementation": provenance["renderer_implementation"],
        "adapter": {key: provenance[key] for key in ADAPTER_KEYS},
        "runtime_revision": provenance["runtime_revision"],
        "dawn_revision": provenance["dawn_revision"],
        "artifact_sha256": provenance["artifact_sha256"],
    }


def case_identity_sha256(case: dict, provenance: dict[str, str]) -> str:
    payload = case_identity_payload(case, provenance)
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode(
        "ascii"
    )
    return hashlib.sha256(encoded).hexdigest()


def require_unique_case_identities(records: list[tuple[str, str]]) -> None:
    seen: dict[str, str] = {}
    for case_id, identity in records:
        previous = seen.get(identity)
        if previous is not None:
            raise ValueError(
                f"duplicate MSAA per-case identity: {previous} and {case_id}"
            )
        seen[identity] = case_id
