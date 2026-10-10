"""Resolve source-owned package paths for shared direct Rust target graphs."""

def _repository_prefix():
    repository = native.repository_name()
    return "@" + repository if repository != "@" else ""

def package_label(relative_label):
    package = native.package_name()
    return _repository_prefix() + "//" + (package + "/" if package else "") + relative_label

def source_path(source_label = None):
    if source_label == None:
        source_label = _repository_prefix() + "//" + native.package_name() + ":Cargo.toml"
    label = Label(source_label)
    return ("external/" + label.workspace_name + "/" if label.workspace_name else "") + label.package

def provenance_env_file():
    return Label(":provenance_env")
