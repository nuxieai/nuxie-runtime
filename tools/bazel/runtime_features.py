"""Optional native tool cuts resolved separately from shipping dependencies."""

NATIVE_TOOLS_ROOTS = {
    "nuxie-audio": ["audio-device"],
    "renderer-replay": ["native-metal", "native-ore-metal"],
    "nuxie-runtime": ["scriptnet"],
}


def uses_native_tools(graph):
    return any(set(features) & graph.features.get(package, set())
               for package, features in NATIVE_TOOLS_ROOTS.items())
