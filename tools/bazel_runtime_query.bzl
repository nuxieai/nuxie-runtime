"""cquery projection of each Rust crate's actual configured input closure."""

def format(target):
    crate = None
    for name, value in (providers(target) or {}).items():
        if name.endswith("%CrateInfo"):
            crate = value
        elif name.endswith("%TestCrateInfo"):
            # C-facing static/shared libraries wrap their Rust crate provider.
            crate = value.crate
    if crate == None:
        return ""
    return json.encode({
        "label": str(target.label),
        "root": crate.root.path,
        "cfgs": crate.cfgs,
        "files": sorted({
            file.path: True
            for file in crate.srcs.to_list() + crate.compile_data.to_list() + crate.data.to_list()
            if not file.path.startswith("bazel-out/")
        }.keys()),
    })
