#!/usr/bin/env python3
"""Generate strict C++ replays and a registry for pinned MSAA corpus cases."""

import argparse
import hashlib
import pathlib
import re
import tomllib

from generate_path_stream_replay import generate_include


CASE_KEYS = {
    "id",
    "stream",
    "sha256",
    "source",
    "scene",
    "width",
    "height",
    "clear_color",
    "counts",
}
ID_RE = re.compile(r"[A-Za-z0-9_-]+")
REVISION_RE = re.compile(r"[0-9a-f]{40}")
PATH_DECLARATION_RE = re.compile(r"    auto path(\d+) = context->makeEmptyRenderPath\(\);")
PATH_REFERENCE_RE = re.compile(r"\bpath(\d+)\b")


def chunk_large_path_replay(
    replay: str,
    function_name: str,
    first_local_path_id: int = 3,
    paths_per_chunk: int = 128,
) -> str:
    """Split the strict hit-test tail into bounded compiler functions."""
    lines = replay.rstrip().splitlines()
    first_declaration = f"    auto path{first_local_path_id} = context->makeEmptyRenderPath();"
    try:
        body_start = lines.index(first_declaration)
        body_end = len(lines) - 1 - lines[::-1].index("    renderer->restore();")
    except ValueError as error:
        raise ValueError("large path replay does not match the chunking contract") from error
    if body_start >= body_end or lines[-1] != "}":
        raise ValueError("large path replay has an invalid chunk boundary")

    groups: list[list[str]] = []
    for line in lines[body_start:body_end]:
        if PATH_DECLARATION_RE.fullmatch(line):
            groups.append([])
        if not groups:
            raise ValueError("large path replay tail must begin with a path declaration")
        groups[-1].append(line)
    if not groups:
        raise ValueError("large path replay has no local path groups")

    helpers = []
    calls = []
    for chunk_index, offset in enumerate(range(0, len(groups), paths_per_chunk)):
        chunk_groups = groups[offset : offset + paths_per_chunk]
        helper_name = f"{function_name}Chunk{chunk_index}"
        helper = [
            "__attribute__((optnone))",
            f"void {helper_name}(rive::RiveRenderer* renderer,",
            "                rive::gpu::RenderContext* context,",
            "                rive::RenderPaint* paint1,",
            "                std::vector<rive::rcp<rive::RenderPath>>& retainedPaths)",
            "{",
        ]
        for group in chunk_groups:
            declaration = PATH_DECLARATION_RE.fullmatch(group[0])
            if declaration is None:
                raise ValueError("large path replay group has no path declaration")
            path_id = declaration.group(1)
            referenced_paths = {
                match.group(1)
                for line in group
                for match in PATH_REFERENCE_RE.finditer(line)
            }
            if referenced_paths != {path_id}:
                raise ValueError(
                    f"large path replay group {path_id} references paths {sorted(referenced_paths)}"
                )
            if any(
                "paint" in line and "paint1" not in line
                or "renderer->save()" in line
                or "renderer->restore()" in line
                or "renderer->transform(" in line
                for line in group
            ):
                raise ValueError(f"large path replay group {path_id} is not self-contained")
            helper.extend(line.replace("paint1.get()", "paint1") for line in group)
            helper.append(f"    retainedPaths.emplace_back(std::move(path{path_id}));")
        helper.extend(["}", ""])
        helpers.extend(helper)
        calls.append(
            f"    {helper_name}(renderer, context, paint1.get(), retainedPaths);"
        )

    retained_paths = [
        "    std::vector<rive::rcp<rive::RenderPath>> retainedPaths;",
        f"    retainedPaths.reserve({len(groups)});",
    ]
    return (
        "\n".join(
            [*helpers, *lines[:body_start], *retained_paths, *calls, *lines[body_end:]]
        )
        + "\n"
    )


def load_cases(manifest: pathlib.Path, repo_root: pathlib.Path) -> list[dict]:
    document = tomllib.loads(manifest.read_text())
    if (
        set(document) != {"version", "case"}
        or type(document["version"]) is not int
        or document["version"] != 1
    ):
        raise ValueError("MSAA reference manifest must be version 1 with only case rows")
    cases = document["case"]
    if not isinstance(cases, list) or not cases:
        raise ValueError("MSAA reference manifest must contain at least one case")
    seen = set()
    for case in cases:
        if set(case) != CASE_KEYS:
            raise ValueError(f"MSAA reference case has unexpected fields: {case.get('id')!r}")
        case_id = case["id"]
        if not isinstance(case_id, str) or ID_RE.fullmatch(case_id) is None:
            raise ValueError(f"unsafe MSAA reference case id: {case_id!r}")
        folded_id = case_id.casefold()
        if folded_id in seen:
            raise ValueError(f"duplicate MSAA reference case id: {case_id}")
        seen.add(folded_id)
        stream = pathlib.Path(case["stream"])
        if stream.is_absolute() or ".." in stream.parts:
            raise ValueError(f"unsafe MSAA reference stream path: {stream}")
        case["stream_path"] = repo_root / stream
        if not isinstance(case["counts"], dict) or not case["counts"]:
            raise ValueError(f"MSAA reference case {case_id} has no command counts")
    return cases


def generate_registry(
    manifest: pathlib.Path,
    repo_root: pathlib.Path,
    runtime_revision: str,
    dawn_revision: str,
) -> tuple[str, str]:
    cases = load_cases(manifest, repo_root)
    body = [
        "// Generated by generate_msaa_reference_registry.py; do not edit.",
        "struct MsaaReferenceCase",
        "{",
        "    const char* id;",
        "    const char* streamSha256;",
        "    uint32_t width;",
        "    uint32_t height;",
        "    uint32_t clearColor;",
        "    bool expectsDrawBatches;",
        "    void (*replay)(rive::RiveRenderer*, rive::gpu::RenderContext*);",
        "};",
        "",
    ]
    for index, case in enumerate(cases):
        replay = generate_include(
            stream=case["stream_path"],
            profile="gm",
            expected_sha256=case["sha256"],
            expected_source=case["source"],
            expected_source_suffix=None,
            expected_artboard="",
            expected_scene=case["scene"],
            expected_width=case["width"],
            expected_height=case["height"],
            expected_clear_color=case["clear_color"],
            expected_sample_seconds=None,
            expected_counts=case["counts"],
            function_name=f"replayMsaaReference{index}",
            blend_mode_override=None,
            function_attribute="__attribute__((optnone))",
        )
        if case["counts"].get("drawPath", 0) > 10_000:
            replay = chunk_large_path_replay(replay, f"replayMsaaReference{index}")
        body.append(replay.rstrip())
        body.append("")
    body.append(
        f"constexpr std::array<MsaaReferenceCase, {len(cases)}> kMsaaReferenceCases = {{{{"
    )
    for index, case in enumerate(cases):
        expects_draw_batches = "true" if any(
            case["counts"].get(command, 0)
            for command in ("drawPath", "drawImage", "drawImageMesh")
        ) else "false"
        body.append(
            f'    {{"{case["id"]}", "{case["sha256"]}", {case["width"]}, '
            f'{case["height"]}, {case["clear_color"]}, {expects_draw_batches}, '
            f'replayMsaaReference{index}}},'
        )
    body.extend(
        [
            "}};",
            "",
            "const MsaaReferenceCase* findMsaaReferenceCase(const char* id)",
            "{",
            "    for (const auto& candidate : kMsaaReferenceCases)",
            "    {",
            "        if (std::strcmp(candidate.id, id) == 0)",
            "        {",
            "            return &candidate;",
            "        }",
            "    }",
            "    return nullptr;",
            "}",
            "",
        ]
    )
    body_text = "\n".join(body)
    registry_sha256 = hashlib.sha256(body_text.encode()).hexdigest()
    metadata = [
        f'constexpr char kMsaaReferenceRegistrySha256[] = "{registry_sha256}";',
        f'constexpr char kMsaaReferenceRuntimeRevision[] = "{runtime_revision}";',
        f'constexpr char kMsaaReferenceDawnRevision[] = "{dawn_revision}";',
        "",
    ]
    return "\n".join([body[0], *metadata, *body[1:]]), registry_sha256


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--runtime-revision", required=True)
    parser.add_argument("--dawn-revision", required=True)
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--print-registry-sha256", action="store_true")
    args = parser.parse_args()
    if sum((bool(args.output), args.check, args.print_registry_sha256)) != 1:
        parser.error(
            "specify exactly one of --output, --check, or --print-registry-sha256"
        )
    for name, revision in (
        ("--runtime-revision", args.runtime_revision),
        ("--dawn-revision", args.dawn_revision),
    ):
        if REVISION_RE.fullmatch(revision) is None:
            parser.error(f"{name} must be a lowercase 40-character revision")
    generated, registry_sha256 = generate_registry(
        args.manifest,
        args.repo_root,
        args.runtime_revision,
        args.dawn_revision,
    )
    if args.output:
        args.output.write_text(generated)
    elif args.print_registry_sha256:
        print(registry_sha256)


if __name__ == "__main__":
    main()
