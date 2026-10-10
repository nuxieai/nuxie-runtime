"""Resolve first-party Cargo features without putting Cargo in Bazel actions.

Cargo manifests remain the dependency authority. Build scripts have an isolated
host graph, and each product graph retains its own optional dependency closure.
The parent Nuxie workspace imports this module to generate its targets too.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
import tomllib
import json
import shutil
import re
import sys
import platform as host_platform


@dataclass(frozen=True)
class Dependency:
    alias: str
    package: str
    kind: str
    condition: str | None
    spec: dict
    local: str | None


@dataclass
class Package:
    name: str
    directory: Path
    manifest: dict
    dependencies: list[Dependency]
    edition: str
    version: str


@dataclass
class FeatureGraph:
    features: dict[str, set[str]]
    optional: dict[str, set[str]]
    packages: dict[str, Package]
    include_dev: bool = False
    platform: str | dict = "native"
    dev_packages: set[str] = field(default_factory=set)

    def dependencies(self, package: str, kinds=("dependencies",)):
        return [
            dep
            for dep in self.packages[package].dependencies
            if dep.kind in kinds
            and (dep.kind != "dev-dependencies" or package in self.dev_packages)
            and cfg_matches(dep.condition, self.platform)
            and (
                not dep.spec.get("optional", False)
                or dep.alias in self.optional[package]
            )
        ]


@dataclass
class NativeDependencies:
    """Dependencies selected independently for each supported native platform."""
    platforms: dict[str, list[Dependency]]


@dataclass
class NativeFeatureGraph:
    """Portable generated targets retain each platform's feature closure.

    The feature union is only used to decide whether a feature-gated target can
    exist at all. Its actual compiler features and edges are emitted as selects.
    """
    platforms: dict[str, FeatureGraph]

    @property
    def features(self):
        return {name: set().union(*(graph.features[name] for graph in self.platforms.values()))
                for name in next(iter(self.platforms.values())).features}

    def dependencies(self, package, kinds=("dependencies",)):
        return NativeDependencies({name: graph.dependencies(package, kinds)
                                   for name, graph in self.platforms.items()})


NATIVE_PLATFORMS = {
    "linux": {"target_arch": "x86_64", "target_os": "linux", "target_vendor": "unknown", "target_env": "gnu", "target_abi": ""},
    "macos": {"target_arch": "aarch64", "target_os": "macos", "target_vendor": "apple", "target_env": "", "target_abi": ""},
    "windows": {"target_arch": "x86_64", "target_os": "windows", "target_vendor": "pc", "target_env": "msvc", "target_abi": ""},
    "ios": {"target_arch": "aarch64", "target_os": "ios", "target_vendor": "apple", "target_env": "", "target_abi": ""},
    "android": {"target_arch": "aarch64", "target_os": "android", "target_vendor": "unknown", "target_env": "", "target_abi": ""},
}


def resolve_native_features(packages, roots=None, include_dev=False, *, host=False):
    """Resolve deterministic platform cuts, independent of the generator host."""
    return NativeFeatureGraph({name: resolve_features(packages, host_roots(packages, target) if host else roots,
                                                     include_dev=include_dev, platform=target)
                               for name, target in NATIVE_PLATFORMS.items()})


def collect_packages(manifests, patches=None):
    """Load local packages; patch names make vendored registry deps local.

    Callers supply every first-party/patch manifest and may combine repositories.
    Patch values can be package names or Paths to the patched manifest directory.
    """
    raw = {}
    by_directory = {}
    for manifest_path in manifests:
        manifest_path = Path(manifest_path).resolve()
        data = tomllib.loads(manifest_path.read_text())
        if "package" not in data:
            continue
        package = data["package"]
        workspace = {}
        workspace_dependencies = {}
        workspace_lints = {}
        for directory in manifest_path.parents:
            candidate = directory / "Cargo.toml"
            if candidate == manifest_path or not candidate.exists():
                continue
            parent = tomllib.loads(candidate.read_text())
            if "workspace" in parent:
                workspace = parent["workspace"].get("package", {})
                workspace_dependencies = parent["workspace"].get("dependencies", {})
                workspace_lints = parent["workspace"].get("lints", {})
                break
        if data.get("lints", {}).get("workspace"):
            data["lints"] = workspace_lints

        def inherited(key, default):
            value = package.get(key, default)
            return workspace.get(key, default) if isinstance(value, dict) else value

        name = package["name"]
        for table in [data, *data.get("target", {}).values()]:
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                for alias, spec in list(table.get(kind, {}).items()):
                    if isinstance(spec, dict) and spec.get("workspace"):
                        inherited_spec = workspace_dependencies[alias]
                        inherited_spec = {"version": inherited_spec} if isinstance(inherited_spec, str) else dict(inherited_spec)
                        inherited_features = inherited_spec.get("features", [])
                        inherited_spec.update({key: value for key, value in spec.items() if key != "workspace"})
                        if "features" in spec:
                            inherited_spec["features"] = sorted(set(inherited_features + spec["features"]))
                        table[kind][alias] = inherited_spec
        if name in raw:
            raise ValueError(f"duplicate local package {name}")
        raw[name] = Package(
            name, manifest_path.parent, data, [],
            str(inherited("edition", "2015")), str(inherited("version", "0.1.0")),
        )
        by_directory[manifest_path.parent] = name

    patch_names = {}
    for name, value in (patches or {}).items():
        if isinstance(value, Path):
            patch_names[name] = by_directory[value.resolve()]
        else:
            patch_names[name] = value
    for package in raw.values():
        tables = [(None, package.manifest)]
        tables.extend(package.manifest.get("target", {}).items())
        for condition, table in tables:
            for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
                for alias, spec in table.get(kind, {}).items():
                    spec = {"version": spec} if isinstance(spec, str) else dict(spec)
                    dependency_name = spec.get("package", alias)
                    local = None
                    if "path" in spec:
                        directory = (package.directory / spec["path"]).resolve()
                        if directory not in by_directory:
                            raise ValueError(f"unmapped local dependency {package.name}: {directory}")
                        local = by_directory[directory]
                    elif dependency_name in patch_names:
                        local = patch_names[dependency_name]
                    package.dependencies.append(
                        Dependency(alias, dependency_name, kind, condition, spec, local)
                    )
    return raw


def cfg_matches(condition, platform="native"):
    """Evaluate Cargo target predicates before optional-feature propagation."""
    if condition is None:
        return True
    if isinstance(platform, dict):
        target = dict(platform)
    elif platform == "wasm":
        target = {"target_arch": "wasm32", "target_os": "unknown", "target_vendor": "unknown", "target_env": "", "target_abi": ""}
    elif platform == "android":
        target = {"target_arch": "aarch64", "target_os": "android", "target_vendor": "unknown", "target_env": "", "target_abi": ""}
    elif platform == "apple":
        target = {"target_arch": "aarch64", "target_os": "macos", "target_vendor": "apple", "target_env": "", "target_abi": ""}
    else:
        system = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")
        arch = {"arm64": "aarch64", "AMD64": "x86_64"}.get(host_platform.machine(), host_platform.machine())
        target = {"target_arch": arch, "target_os": system, "target_vendor": "apple" if system == "macos" else "unknown", "target_env": "msvc" if system == "windows" else "gnu" if system == "linux" else "", "target_abi": ""}
    if not condition.startswith("cfg("):
        raise ValueError(f"unsupported Cargo target triple predicate {condition}")
    tokens = re.findall(r'"[^"]*"|[A-Za-z_][A-Za-z0-9_]*|[(),=]', condition)
    index = 2

    def parse():
        nonlocal index
        token = tokens[index]
        index += 1
        if token in ("all", "any", "not"):
            if tokens[index] != "(":
                raise ValueError(condition)
            index += 1
            children = []
            while tokens[index] != ")":
                children.append(parse())
                if tokens[index] == ",":
                    index += 1
            index += 1
            return all(children) if token == "all" else any(children) if token == "any" else not children[0]
        if tokens[index] == "=":
            index += 1
            expected = json.loads(tokens[index])
            index += 1
            if token not in target:
                raise ValueError(f"unsupported target key {token} in {condition}")
            return target[token] == expected
        if token == "windows":
            return target["target_os"] == "windows"
        if token == "unix":
            return target["target_os"] not in ("windows", "unknown")
        raise ValueError(f"unsupported Cargo target predicate {condition}")

    result = parse()
    if index != len(tokens) - 1:
        raise ValueError(condition)
    return result


def resolve_features(packages, roots, include_dev=False, platform="native"):
    """Resolve Cargo's additive local features for a single product graph.

    roots maps package names to feature lists. Include `default` explicitly when
    the root uses Cargo defaults. Target conditions are retained on each edge;
    they become Bazel selects when targets are emitted.
    """
    graph = FeatureGraph(
        {name: set() for name in packages},
        {name: set() for name in packages}, packages, include_dev, platform,
        set(roots) if include_dev else set(),
    )
    reachable = set(roots)
    for name, features in roots.items():
        graph.features[name].update(features)
    changed = True
    while changed:
        before = (
            tuple((name, tuple(sorted(values))) for name, values in graph.features.items()),
            tuple((name, tuple(sorted(values))) for name, values in graph.optional.items()),
            frozenset(reachable),
        )
        for name in list(reachable):
            package = packages[name]
            features = graph.features[name]
            defined = package.manifest.get("features", {})
            explicit = {
                value[4:]
                for values in defined.values()
                for value in values
                if value.startswith("dep:")
            }
            dependency_features = []
            for feature in list(features):
                if feature in defined:
                    values = defined[feature]
                elif feature not in explicit and any(
                    dep.alias == feature and dep.spec.get("optional")
                    for dep in package.dependencies
                ):
                    values = [f"dep:{feature}"]
                elif feature == "default":
                    values = []
                else:
                    raise ValueError(f"unknown feature {name}/{feature}")
                for value in values:
                    if value.startswith("dep:"):
                        graph.optional[name].add(value[4:])
                    elif "/" in value:
                        alias, dep_feature = value.split("/", 1)
                        weak = alias.endswith("?")
                        alias = alias.rstrip("?")
                        if not weak:
                            graph.optional[name].add(alias)
                            if alias not in explicit and any(dep.alias == alias and dep.spec.get("optional") for dep in package.dependencies):
                                features.add(alias)
                        dependency_features.append((alias, dep_feature, weak))
                    else:
                        features.add(value)

            kinds = ("dependencies", "dev-dependencies") if include_dev else ("dependencies",)
            for dep in graph.dependencies(name, kinds):
                if dep.local is None:
                    continue
                reachable.add(dep.local)
                selected = graph.features[dep.local]
                if dep.spec.get("default-features", True):
                    selected.add("default")
                selected.update(dep.spec.get("features", []))
                selected.update(
                    dep_feature
                    for alias, dep_feature, weak in dependency_features
                    if alias == dep.alias
                    and (not weak or dep.alias in graph.optional[name])
                )
        after = (
            tuple((name, tuple(sorted(values))) for name, values in graph.features.items()),
            tuple((name, tuple(sorted(values))) for name, values in graph.optional.items()),
            frozenset(reachable),
        )
        changed = before != after
    return graph


def host_roots(packages, platform=None):
    """Build dependencies use an execution-platform graph, never target features."""
    roots = {}
    for package in packages.values():
        for dep in package.dependencies:
            if dep.kind != "build-dependencies" or dep.local is None or (platform is not None and not cfg_matches(dep.condition, platform)):
                continue
            selected = roots.setdefault(dep.local, set())
            if dep.spec.get("default-features", True):
                selected.add("default")
            selected.update(dep.spec.get("features", []))
    return roots


def _toml_value(value):
    if isinstance(value, str):
        return json.dumps(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return "[" + ", ".join(_toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{json.dumps(key)} = {_toml_value(item)}" for key, item in value.items()) + " }"
    return str(value)


def graph_feature_union(graphs):
    """Features actually selected by the supported source build graphs."""
    result = {}
    for graph in graphs.values():
        for name, features in graph.features.items():
            result.setdefault(name, set()).update(features)
    return result


def write_registry_workspace(packages, destination, source_lock, patches=None, reset_lock=False,
                             workspace_members=None, enabled_features=None):
    """Flatten local manifests so crate_universe resolves one self-contained root.

    These stubs are dependency-resolution inputs only. Real Rust compilation uses
    the original source files in direct rust_library/rust_binary targets. Cargo
    is used while repinning dependencies, never inside a Rust build action.

    workspace_members identifies authored roots. Every stub must be a metadata
    workspace member so crate_universe exports its dependency alias mappings.
    Other stubs have inert root defaults and no dev dependencies. Their authored
    defaults move to an internal feature selected only by incoming dependency
    edges, preserving Cargo's default-features behavior without root pollution.
    """
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    # Resolution directories are generated outputs too. Remove obsolete stubs
    # when a connected product's package closure shrinks, including leftovers
    # from a previously regenerated workspace whose member list already changed.
    for directory in destination.iterdir():
        manifest = directory / "Cargo.toml"
        if (directory.is_dir() and directory.name not in packages and manifest.is_file()
                and manifest.read_text().startswith("# Generated dependency metadata.")):
            shutil.rmtree(directory)
    members = set(packages) if workspace_members is None else set(workspace_members)
    if not members.issubset(packages):
        raise ValueError("unknown registry workspace members: " + ", ".join(sorted(members - packages.keys())))
    root_lines = [
        "# Generated by tools/bazel; dependency-resolution inputs only.",
        "[workspace]", "resolver = \"3\"",
        "members = " + _toml_value(sorted(packages)), "",
    ]
    patch_names = patches or {}
    if patch_names:
        root_lines.extend(["[patch.crates-io]"])
        for name, package_name in sorted(patch_names.items()):
            root_lines.append(f"{json.dumps(name)} = {{ path = {json.dumps(package_name)} }}")
    (destination / "Cargo.toml").write_text("\n".join(root_lines) + "\n")
    if reset_lock or not (destination / "Cargo.lock").exists():
        shutil.copyfile(source_lock, destination / "Cargo.lock")
    for package in packages.values():
        directory = destination / package.name
        directory.mkdir(exist_ok=True)
        original = package.manifest
        lines = [
            "# Generated dependency metadata. Rust sources stay in their owning package.",
            "[package]", f"name = {json.dumps(package.name)}",
            f"version = {json.dumps(package.version)}", f"edition = {json.dumps(package.edition)}",
            "autobins = false", "autoexamples = false", "autotests = false", "autobenches = false",
        ]
        if "links" in original["package"]:
            lines.append("links = " + _toml_value(original["package"]["links"]))
        has_build = original["package"].get("build") is not False and (package.directory / original["package"].get("build", "build.rs")).is_file()
        if has_build:
            lines.append('build = "build.rs"')
            (directory / "build.rs").write_text("fn main() {}\n")
        else:
            lines.append("build = false")
        lines.extend(["", "[lib]", 'path = "lib.rs"'])
        lib = original.get("lib", {})
        if lib.get("proc-macro"):
            lines.append("proc-macro = true")
        if "name" in lib:
            lines.append("name = " + _toml_value(lib["name"]))
        (directory / "lib.rs").write_text("// Dependency metadata only; never compiled by Bazel.\n")
        dependency_default = "__bazel_dependency_default"
        if dependency_default in original.get("features", {}):
            raise ValueError(f"reserved metadata feature {package.name}/{dependency_default}")
        selected = set((enabled_features or {}).get(package.name, ())) - {"default"} if package.name in members else set()
        retained_dependencies = [dep for dep in package.dependencies
                                 if package.name in members or dep.kind != "dev-dependencies"]
        removed_aliases = {dep.alias for dep in package.dependencies} - {dep.alias for dep in retained_dependencies}

        def rewrite_feature(value):
            if "/" in value:
                alias, feature = value.split("/", 1)
                if alias.rstrip("?") in removed_aliases:
                    return None
                if feature == "default" and any(dep.alias == alias.rstrip("?") and dep.local not in members
                                                and dep.local is not None for dep in package.dependencies):
                    return alias + "/" + dependency_default
            if value.startswith("dep:") and value[4:] in removed_aliases:
                return None
            return dependency_default if value == "default" and package.name not in members else value

        if original.get("features") or selected:
            lines.extend(["", "[features]"])
            for name, values in original.get("features", {}).items():
                if name == "default":
                    if package.name not in members:
                        name = dependency_default
                    else:
                        values = sorted(set(values) | selected)
                rewritten = [rewrite_feature(value) for value in values]
                lines.append(f"{json.dumps(name)} = {_toml_value([value for value in rewritten if value is not None])}")
            if package.name not in members:
                lines.append('"default" = []')
            if "default" not in original.get("features", {}) and selected:
                lines.append('"default" = ' + _toml_value(sorted(selected)))
        tables = {}
        for dep in retained_dependencies:
            spec = dict(dep.spec)
            spec.pop("workspace", None)
            if dep.local is not None:
                spec["path"] = f"../{dep.local}"
                if dep.local not in members:
                    target_features = packages[dep.local].manifest.get("features", {})
                    defaults = spec.get("default-features", True)
                    spec["default-features"] = False
                    features = [dependency_default if feature == "default" else feature
                                for feature in spec.get("features", [])]
                    if defaults and "default" in target_features:
                        features.append(dependency_default)
                    if features:
                        spec["features"] = sorted(set(features))
            table = dep.kind if dep.condition is None else f"target.{json.dumps(dep.condition)}.{dep.kind}"
            tables.setdefault(table, []).append((dep.alias, spec))
        for table, dependencies in tables.items():
            lines.extend(["", f"[{table}]"])
            for alias, spec in dependencies:
                lines.append(f"{json.dumps(alias)} = {_toml_value(spec)}")
        (directory / "Cargo.toml").write_text("\n".join(lines) + "\n")
