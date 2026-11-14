"""Optional native tool cuts resolved separately from shipping dependencies."""

NATIVE_TOOLS_ROOTS = {
    "nuxie-audio": ["audio-device"],
    "renderer-replay": ["native-metal", "native-ore-metal"],
    "nuxie-runtime": ["scriptnet"],
}


def uses_native_tools(graph):
    return any(set(features) & graph.features.get(package, set())
               for package, features in NATIVE_TOOLS_ROOTS.items())


# Features whose code compiles only for Apple platforms. With one on, the
# renderer's Metal modules exist only for macOS and iOS, so a test whose
# build reaches one compiles and runs only on a Mac. Every Metal root reaches
# this one: nuxie-renderer's renderer-metal and native-ore-metal-experimental,
# nuxie's renderer-metal, nux-capi's apple-metal and renderer-replay's
# native-metal.
APPLE_ONLY_FEATURES = {
    "nuxie-renderer": ["native-metal-experimental"],
}


def requires_macos(graph, package):
    """Whether `package`'s tests, built from `graph`, need a Mac: an Apple-only
    feature is on in the package or in any local package its tests compile."""
    seen = {package}
    pending = [(package, ("dependencies", "dev-dependencies"))]
    while pending:
        name, kinds = pending.pop()
        if set(APPLE_ONLY_FEATURES.get(name, ())) & graph.features.get(name, set()):
            return True
        for dep in graph.dependencies(name, kinds):
            if dep.local and dep.local not in seen:
                seen.add(dep.local)
                pending.append((dep.local, ("dependencies",)))
    return False
