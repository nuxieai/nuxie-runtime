#!/usr/bin/env python3
"""Bind LayoutComponentStyle's C++ *Changed handlers to Rust dirt routes.

The Rust port dispatches LayoutComponentStyle property changes through the
string tables and bespoke arms in
crates/nuxie-runtime/src/layout/layout_component_style.rs. Upstream, the same
dispatch is a set of generated ``*Changed()`` virtuals overridden in
src/layout/layout_component_style.cpp. Nothing used to tie the two together:
a pin advance that adds, removes, or reroutes a handler left the Rust tables
stale silently.

This checker extracts every ``*Changed`` handler from the pinned C++ (the
``upstream_ref`` in file-correspondence-manifest.toml, verified against the
upstream checkout's HEAD like the sibling checkers) and asserts each one has
the corresponding Rust route:

- handlers whose body routes ``markLayoutNodeDirty()`` must appear in
  ``NODE_DIRTY_PROPERTIES``;
- handlers whose body routes ``markLayoutStyleDirty()`` must appear in
  ``STYLE_DIRTY_PROPERTIES``;
- bespoke handlers must be listed in ``BESPOKE_ROUTES`` below, and every
  required Rust marker for the route must be present.

It also checks the reverse direction (no stale Rust table entries), and that
generated-base virtuals which LayoutComponentStyle deliberately leaves as
inherited no-ops stay triaged in ``INHERITED_NOOP_HANDLERS``.
"""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import subprocess
import sys
import tomllib


class CheckFailure(Exception):
    """Raised when the C++ handler set and the Rust dirt routes diverge."""


UPSTREAM_CPP = "src/layout/layout_component_style.cpp"
UPSTREAM_CONCRETE_HPP = "include/rive/layout/layout_component_style.hpp"
UPSTREAM_GENERATED_HPPS = (
    "include/rive/generated/layout/layout_component_style_base.hpp",
    "include/rive/generated/layout/layout_sizing_style_base.hpp",
)
RUST_STYLE_MODULE = "crates/nuxie-runtime/src/layout/layout_component_style.rs"
RUST_ARTBOARD_MODULE = "crates/nuxie-runtime/src/artboard.rs"

# Generated-base virtuals that LayoutComponentStyle does not override at the
# pin: upstream treats the change as a no-op, so no Rust route is required.
# A pin advance that overrides one of these (or introduces a new virtual)
# must be triaged here or routed in Rust.
INHERITED_NOOP_HANDLERS = frozenset(
    {
        "animationStyleType",
        "flexBasisUnitsValue",
        "interpolationType",
        "interpolatorId",
        "linkCornerRadius",
    }
)

# C++ ``*Changed`` methods on LayoutComponentStyle that are internal routing
# helpers, not generated property handlers.
HELPER_METHODS = frozenset({"scaleType", "display"})

# Handlers whose C++ body does more than the bulk markLayoutNodeDirty /
# markLayoutStyleDirty routes. Each maps to the Rust markers that implement
# the same route; a marker is (rust module, required substring).
BESPOKE_ROUTES: dict[str, tuple[tuple[str, str], ...]] = {
    # scaleTypeChanged(): recomputes intrinsic sizing on the parent layout.
    "layoutWidthScaleType": (
        (RUST_STYLE_MODULE, "fn scale_type_changed"),
        (RUST_STYLE_MODULE, '"layoutWidthScaleType"'),
    ),
    "layoutHeightScaleType": (
        (RUST_STYLE_MODULE, "fn scale_type_changed"),
        (RUST_STYLE_MODULE, '"layoutHeightScaleType"'),
    ),
    # displayChanged(): LayoutComponent::displayChanged propagates collapse
    # before marking the layout node dirty. The collapse propagation lives in
    # artboard.rs; the node dirt comes from NODE_DIRTY_PROPERTIES membership,
    # which the route below also requires.
    "displayValue": (
        (RUST_ARTBOARD_MODULE, "fn propagate_layout_component_display_changed"),
        (RUST_ARTBOARD_MODULE, "layout_component_style_display_value_property_key"),
    ),
    "layoutTypeValue": (
        (RUST_STYLE_MODULE, "fn layout_type_changed"),
        (RUST_STYLE_MODULE, '"layoutTypeValue"'),
    ),
    "positionTypeValue": (
        (RUST_STYLE_MODULE, "fn position_type_changed"),
        (RUST_STYLE_MODULE, '"positionTypeValue"'),
    ),
    "flexDirectionValue": (
        (RUST_STYLE_MODULE, "fn flex_direction_changed"),
        (RUST_STYLE_MODULE, '"flexDirectionValue"'),
    ),
    "directionValue": (
        (RUST_STYLE_MODULE, "fn direction_changed"),
        (RUST_STYLE_MODULE, '"directionValue"'),
    ),
    "positionLeft": ((RUST_STYLE_MODULE, "mark_position_left_changed"),),
    "positionTop": ((RUST_STYLE_MODULE, "mark_position_top_changed"),),
}

# Bespoke handlers whose C++ route also marks the layout node dirty, so the
# property must additionally sit in NODE_DIRTY_PROPERTIES. displayValue's
# markLayoutNodeDirty happens inside LayoutComponent::displayChanged rather
# than in the handler body, so it is listed explicitly.
BESPOKE_NODE_DIRTY = frozenset({"displayValue"})

OVERRIDE_RE = re.compile(r"\bvoid\s+(\w+)Changed\(\)\s+override\s*;")
VIRTUAL_RE = re.compile(r"\bvirtual\s+void\s+(\w+)Changed\(\)")
DEFINITION_RE = re.compile(r"\bvoid\s+LayoutComponentStyle::(\w+)Changed\(\)")


def git_head(repository: pathlib.Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(repository), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise CheckFailure(
            f"unable to resolve HEAD in {repository}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def read_upstream_file(rive_runtime_dir: pathlib.Path, relative: str) -> str:
    path = rive_runtime_dir / relative
    if not path.is_file():
        raise CheckFailure(f"pinned upstream file is missing: {relative}")
    return path.read_text()


def parse_generated_virtuals(sources: list[str]) -> set[str]:
    virtuals: set[str] = set()
    for source in sources:
        virtuals.update(VIRTUAL_RE.findall(source))
    return virtuals


def parse_concrete_overrides(source: str) -> set[str]:
    return set(OVERRIDE_RE.findall(source))


def parse_cpp_handler_bodies(source: str) -> dict[str, str]:
    """Map each ``*Changed`` method to the union of its definition bodies.

    A method can be defined more than once (real body under
    ``#ifdef WITH_RIVE_LAYOUT`` plus a no-op ``#else`` stub); the bodies are
    concatenated so route markers survive either order.
    """

    bodies: dict[str, str] = {}
    for match in DEFINITION_RE.finditer(source):
        name = match.group(1)
        index = source.index("{", match.end())
        depth = 0
        for end in range(index, len(source)):
            if source[end] == "{":
                depth += 1
            elif source[end] == "}":
                depth -= 1
                if depth == 0:
                    break
        else:
            raise CheckFailure(f"unbalanced body for {name}Changed in {UPSTREAM_CPP}")
        bodies[name] = bodies.get(name, "") + source[index : end + 1]
    return bodies


def parse_rust_table(source: str, table: str) -> list[str]:
    match = re.search(
        rf"const {table}: &\[&str\] = &\[(.*?)\];",
        source,
        re.S,
    )
    if match is None:
        raise CheckFailure(f"{RUST_STYLE_MODULE} no longer defines {table}")
    return re.findall(r'"([^"]+)"', match.group(1))


def check(
    repo_root: pathlib.Path,
    rive_runtime_dir: pathlib.Path,
    file_manifest: pathlib.Path,
) -> str:
    errors: list[str] = []

    manifest = tomllib.loads(file_manifest.read_text())
    upstream_ref = str(manifest.get("upstream_ref", ""))
    if not re.fullmatch(r"[0-9a-f]{40}", upstream_ref):
        raise CheckFailure(
            f"{file_manifest.name} upstream_ref is not a 40-hex commit: {upstream_ref!r}"
        )
    actual = git_head(rive_runtime_dir)
    if actual != upstream_ref:
        raise CheckFailure(
            f"upstream checkout is {actual}; {file_manifest.name} pins {upstream_ref}"
        )

    virtuals = parse_generated_virtuals(
        [read_upstream_file(rive_runtime_dir, path) for path in UPSTREAM_GENERATED_HPPS]
    )
    overrides = parse_concrete_overrides(
        read_upstream_file(rive_runtime_dir, UPSTREAM_CONCRETE_HPP)
    )
    bodies = parse_cpp_handler_bodies(read_upstream_file(rive_runtime_dir, UPSTREAM_CPP))

    if not virtuals:
        raise CheckFailure("no *Changed virtuals found in the generated base headers")
    if not overrides:
        raise CheckFailure(f"no *Changed overrides found in {UPSTREAM_CONCRETE_HPP}")

    for name in sorted(overrides - virtuals):
        errors.append(
            f"{UPSTREAM_CONCRETE_HPP} overrides {name}Changed which no generated base declares"
        )
    for name in sorted(virtuals - overrides - INHERITED_NOOP_HANDLERS):
        errors.append(
            f"generated base declares {name}Changed with no LayoutComponentStyle override; "
            "triage it in INHERITED_NOOP_HANDLERS (upstream no-op) or port a route"
        )
    for name in sorted(INHERITED_NOOP_HANDLERS - virtuals):
        errors.append(
            f"INHERITED_NOOP_HANDLERS lists {name} but no generated base declares "
            f"{name}Changed; remove the stale entry"
        )
    for name in sorted(INHERITED_NOOP_HANDLERS & overrides):
        errors.append(
            f"INHERITED_NOOP_HANDLERS lists {name} but LayoutComponentStyle now "
            "overrides it; port a route and remove the entry"
        )

    for name in sorted(set(bodies) - overrides - HELPER_METHODS):
        errors.append(
            f"{UPSTREAM_CPP} defines {name}Changed which is neither an override nor a "
            "known helper method"
        )
    for name in sorted(overrides - set(bodies)):
        errors.append(f"{UPSTREAM_CPP} does not define the {name}Changed override")

    rust_style = (repo_root / RUST_STYLE_MODULE).read_text()
    rust_sources = {RUST_STYLE_MODULE: rust_style}
    node_table = parse_rust_table(rust_style, "NODE_DIRTY_PROPERTIES")
    style_table = parse_rust_table(rust_style, "STYLE_DIRTY_PROPERTIES")
    node_properties = set(node_table)
    style_properties = set(style_table)

    def rust_source(module: str) -> str:
        if module not in rust_sources:
            path = repo_root / module
            if not path.is_file():
                raise CheckFailure(f"Rust module is missing: {module}")
            rust_sources[module] = path.read_text()
        return rust_sources[module]

    node_dirty_handlers: set[str] = set()
    style_dirty_handlers: set[str] = set()
    for name in sorted(overrides & set(bodies)):
        body = bodies[name]
        routes_node = "markLayoutNodeDirty" in body
        routes_style = "markLayoutStyleDirty" in body
        if name in BESPOKE_ROUTES:
            for module, marker in BESPOKE_ROUTES[name]:
                if marker not in rust_source(module):
                    errors.append(
                        f"bespoke handler {name}Changed requires marker {marker!r} in "
                        f"{module}, which is absent"
                    )
            if routes_node or name in BESPOKE_NODE_DIRTY:
                node_dirty_handlers.add(name)
                if name not in node_properties:
                    errors.append(
                        f"{name}Changed also marks the layout node dirty but {name} is "
                        "not in NODE_DIRTY_PROPERTIES"
                    )
        elif routes_node:
            node_dirty_handlers.add(name)
            if name not in node_properties:
                errors.append(
                    f"{name}Changed routes markLayoutNodeDirty but {name} is not in "
                    "NODE_DIRTY_PROPERTIES"
                )
        elif routes_style:
            style_dirty_handlers.add(name)
            if name not in style_properties:
                errors.append(
                    f"{name}Changed routes markLayoutStyleDirty but {name} is not in "
                    "STYLE_DIRTY_PROPERTIES"
                )
        else:
            errors.append(
                f"{name}Changed has no recognized route; port it and add the property "
                "to BESPOKE_ROUTES"
            )

    for name in node_table:
        if name not in node_dirty_handlers and name not in BESPOKE_ROUTES:
            errors.append(
                f"NODE_DIRTY_PROPERTIES entry {name} has no markLayoutNodeDirty (or "
                "bespoke) C++ handler at the pin; remove or reroute it"
            )
    for name in style_table:
        if name not in style_dirty_handlers:
            errors.append(
                f"STYLE_DIRTY_PROPERTIES entry {name} has no markLayoutStyleDirty C++ "
                "handler at the pin; remove or reroute it"
            )

    for name, table in (("NODE_DIRTY_PROPERTIES", node_table), ("STYLE_DIRTY_PROPERTIES", style_table)):
        duplicates = sorted({entry for entry in table if table.count(entry) > 1})
        if duplicates:
            errors.append(f"{name} has duplicate entries: {', '.join(duplicates)}")

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))

    return (
        f"layout-style-handlers: overrides={len(overrides)} "
        f"(node={len(node_dirty_handlers - set(BESPOKE_ROUTES))}, "
        f"style={len(style_dirty_handlers)}, bespoke={len(BESPOKE_ROUTES)}); "
        f"inherited-noop={len(INHERITED_NOOP_HANDLERS)}"
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument(
        "--rive-runtime-dir",
        type=pathlib.Path,
        default=os.environ.get("RIVE_RUNTIME_DIR"),
        required="RIVE_RUNTIME_DIR" not in os.environ,
    )
    parser.add_argument("--file-manifest", type=pathlib.Path, required=True)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        summary = check(
            args.repo_root.resolve(),
            pathlib.Path(args.rive_runtime_dir).resolve(),
            args.file_manifest.resolve(),
        )
    except CheckFailure as failure:
        print(f"layout-style-handlers check failed:\n{failure}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
