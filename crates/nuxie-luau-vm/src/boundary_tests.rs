//! The layering rule (docs/luau-safe-vm-embedding-core.md, section 5.8):
//! the embedding layer (`src/embed/`) uses the core; the core never uses
//! the layer. Rust privacy keeps the layer's host type out of reach (its
//! module is private to the layer), but not the public `Vm` alias, so this
//! test reads the code of every core source file and fails if it names
//! the layer's module, `Vm` or `Compat`. Comments may mention them.
//!
//! The sources are compiled in (`include_str!`). Each module the crate
//! root and the core's directory modules declare must be listed, so a new
//! core file cannot escape the check.

/// Every core source, by its path under `src/`.
const CORE: &[(&str, &str)] = &[
    ("budget.rs", include_str!("budget.rs")),
    ("bytecode/mod.rs", include_str!("bytecode/mod.rs")),
    ("bytecode/load.rs", include_str!("bytecode/load.rs")),
    ("bytecode/verify.rs", include_str!("bytecode/verify.rs")),
    ("call.rs", include_str!("call.rs")),
    ("error.rs", include_str!("error.rs")),
    ("frames.rs", include_str!("frames.rs")),
    ("gc.rs", include_str!("gc.rs")),
    ("heap.rs", include_str!("heap.rs")),
    ("host.rs", include_str!("host.rs")),
    ("hosted.rs", include_str!("hosted.rs")),
    ("hosted_tests.rs", include_str!("hosted_tests.rs")),
    ("interp.rs", include_str!("interp.rs")),
    ("libs/mod.rs", include_str!("libs/mod.rs")),
    ("libs/base.rs", include_str!("libs/base.rs")),
    ("libs/bit32.rs", include_str!("libs/bit32.rs")),
    ("libs/buffer.rs", include_str!("libs/buffer.rs")),
    ("libs/coroutine.rs", include_str!("libs/coroutine.rs")),
    ("libs/debug.rs", include_str!("libs/debug.rs")),
    ("libs/fast.rs", include_str!("libs/fast.rs")),
    ("libs/math.rs", include_str!("libs/math.rs")),
    ("libs/os.rs", include_str!("libs/os.rs")),
    ("libs/sort.rs", include_str!("libs/sort.rs")),
    ("libs/string.rs", include_str!("libs/string.rs")),
    ("libs/strpack.rs", include_str!("libs/strpack.rs")),
    ("libs/table.rs", include_str!("libs/table.rs")),
    ("libs/utf8.rs", include_str!("libs/utf8.rs")),
    ("libs/vector.rs", include_str!("libs/vector.rs")),
    ("limits.rs", include_str!("limits.rs")),
    ("meta.rs", include_str!("meta.rs")),
    ("native.rs", include_str!("native.rs")),
    ("num.rs", include_str!("num.rs")),
    ("roots.rs", include_str!("roots.rs")),
    ("stack.rs", include_str!("stack.rs")),
    ("table.rs", include_str!("table.rs")),
    ("thread.rs", include_str!("thread.rs")),
    ("userdata.rs", include_str!("userdata.rs")),
    ("value.rs", include_str!("value.rs")),
    ("vm.rs", include_str!("vm.rs")),
];

/// The files that declare modules, with the directory their modules live in.
const DECLARERS: &[(&str, &str)] = &[
    ("", include_str!("lib.rs")),
    ("bytecode/", include_str!("bytecode/mod.rs")),
    ("libs/", include_str!("libs/mod.rs")),
];

/// Modules of the crate root that are not core: the layer and this test.
const NOT_CORE: &[&str] = &["embed", "boundary_tests"];

const LAYER_NAMES: &[&str] = &["embed", "Vm", "Compat"];

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `line` names `word` as a whole identifier.
fn names(line: &str, word: &str) -> bool {
    line.match_indices(word).any(|(at, _)| {
        let before = line.get(..at).and_then(|s| s.chars().next_back());
        let after = line.get(at.saturating_add(word.len())..).and_then(|s| s.chars().next());
        !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
    })
}

/// The code of a line, without its comment.
fn code(line: &str) -> &str {
    line.split("//").next().unwrap_or_default()
}

#[test]
fn every_core_module_is_checked() -> Result<(), String> {
    let mut declared = 0usize;
    for (dir, source) in DECLARERS {
        for line in source.lines().map(str::trim) {
            let Some(rest) = line.strip_suffix(';') else {
                continue;
            };
            let Some(name) = rest.split_whitespace().last().filter(|_| {
                rest.starts_with("mod ") || rest.contains(" mod ")
            }) else {
                continue;
            };
            if dir.is_empty() && NOT_CORE.contains(&name) {
                continue;
            }
            let file = format!("{dir}{name}.rs");
            let dir_file = format!("{dir}{name}/mod.rs");
            if !CORE.iter().any(|(path, _)| *path == file || *path == dir_file) {
                return Err(format!("core module `{dir}{name}` is not in CORE"));
            }
            declared = declared.saturating_add(1);
        }
    }
    // One declaration per listed file: none is stale, and the parse saw them.
    if declared != CORE.len() {
        return Err(format!("{declared} core modules declared, {} listed", CORE.len()));
    }
    Ok(())
}

#[test]
fn the_core_never_names_the_embedding_layer() -> Result<(), String> {
    let mut found = Vec::new();
    for (path, source) in CORE {
        for (n, line) in source.lines().enumerate() {
            let code = code(line);
            for word in LAYER_NAMES {
                if names(code, word) {
                    found.push(format!("src/{path}:{}: `{word}`", n.saturating_add(1)));
                }
            }
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!("the core names the embedding layer:\n{}", found.join("\n")))
    }
}

#[test]
fn the_check_sees_a_mention() -> Result<(), String> {
    let cases = [
        ("use crate::embed::Vm;", true),
        ("fn f(vm: &mut Vm) {}", true),
        ("let host: Compat = x;", true),
        ("struct HostedVm<H>;", false),
        ("let embedded = 1;", false),
        ("x(); // the Vm alias", false),
    ];
    for (line, expected) in cases {
        let hit = LAYER_NAMES.iter().any(|w| names(code(line), w));
        if hit != expected {
            return Err(format!("{line:?}: expected {expected}"));
        }
    }
    Ok(())
}
