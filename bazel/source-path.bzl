"""Resolve source-owned package paths for shared direct Rust target graphs."""

def package_label(relative_label):
    package = native.package_name()
    return native.repository_name() + "//" + (package + "/" if package else "") + relative_label

def source_path(source_label = None):
    if source_label == None:
        source_label = native.repository_name() + "//" + native.package_name() + ":Cargo.toml"
    label = Label(source_label)
    return ("external/" + label.workspace_name + "/" if label.workspace_name else "") + label.package
