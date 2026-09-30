//! Supplemental branch coverage for upstream 696345630f860112d0496e0d1844e9d7087250d8.
//! Each tool invocation owns its global argument/name state, as separate Python
//! invocations do. Compile the owner independently for the two output modes.
#[path = "../src/mechanical_port/source/renderer/src/shaders/minify_py.rs"]
mod compact;
#[path = "../src/mechanical_port/source/renderer/src/shaders/minify_py.rs"]
mod human_readable;

fn check_output(human: bool) {
    let cases = [
        ("$OUT($float2)$foo;", "OUT(float2) foo;", "OUT(float2) foo;"),
        (
            "$OUT($float2) \n $foo;",
            "OUT(float2) foo;",
            "OUT(float2) \n  foo;",
        ),
        (
            "$OUT($float2)/*comment*/$foo;",
            "OUT(float2) foo;",
            "OUT(float2)/*comment*/foo;",
        ),
        (
            "$OUT(($float2))$foo;",
            "OUT((float2)) foo;",
            "OUT((float2)) foo;",
        ),
        (
            "$OUT($float2).$rgba;",
            "OUT(float2).rgba;",
            "OUT(float2).rgba;",
        ),
        (
            "$OUT($float2)+$foo;",
            "OUT(float2)+foo;",
            "OUT(float2)+foo;",
        ),
        ("$OUT($float2)1;", "OUT(float2)1;", "OUT(float2)1;"),
        ("$OUT[$float2]$foo;", "OUT[float2]foo;", "OUT[float2]foo;"),
        ("$float2 $foo;", "float2 foo;", "float2 foo;"),
        (
            "#define $M $OUT($float2)$foo\n$M",
            "#define M OUT(float2) foo\nM",
            "#define M  OUT(float2) foo\nM",
        ),
    ];
    let directory = std::env::temp_dir().join(format!(
        "nuxie-minifier-spacing-{}-{}-{}",
        std::process::id(),
        human,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let output = directory.join("output");
    let mut arguments = vec![
        "minify".to_owned(),
        "-o".to_owned(),
        output.to_str().unwrap().to_owned(),
    ];
    if human {
        arguments.push("-H".to_owned());
    }
    for (index, (source, _, _)) in cases.iter().enumerate() {
        let input = directory.join(format!("case{index}.glsl"));
        std::fs::write(&input, source).unwrap();
        arguments.push(input.to_str().unwrap().to_owned());
    }
    if human {
        human_readable::run(arguments).unwrap();
    } else {
        compact::run(arguments).unwrap();
    }
    for (index, (source, compact_expected, human_expected)) in cases.iter().enumerate() {
        let expected = if human {
            human_expected
        } else {
            compact_expected
        };
        let offline =
            std::fs::read_to_string(output.join(format!("case{index}.minified.glsl"))).unwrap();
        assert_eq!(&offline, expected, "{source}");
        let embedded =
            std::fs::read_to_string(output.join(format!("case{index}.glsl.hpp"))).unwrap();
        // Both offline and embedded emission take the same spacing branch.
        assert!(
            embedded.contains(&format!("R\"===({expected}\n)===\";")),
            "{source}: {embedded}"
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn compact_macro_spacing() {
    check_output(false);
}

#[test]
fn human_readable_macro_spacing() {
    check_output(true);
}
