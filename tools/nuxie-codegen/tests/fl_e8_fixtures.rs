use std::fs;
use std::path::Path;
use std::process::Command;

fn emit(name: &str, font: &Path, out: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_nuxie-codegen"))
        .args(["fixture", "--name", name, "--font"])
        .arg(font)
        .arg("--out")
        .arg(out)
        .status()
        .expect("run nuxie-codegen fixture emitter");
    assert!(status.success());
}

#[test]
fn fl_e8_fixtures_are_reproducible_importable_and_schema_typed() {
    let root = std::env::temp_dir().join(format!("nuxie-fl-e8-fixtures-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let font = root.join("font.bin");
    fs::write(&font, b"deterministic fixture font payload").unwrap();

    for (name, file_name, type_key) in [
        ("text-style-feature", "text_style_feature.riv", 164),
        (
            "text-variation-modifier",
            "text_variation_modifier.riv",
            162,
        ),
    ] {
        let first = root.join(file_name);
        let second = root.join(format!("second-{file_name}"));
        emit(name, &font, &first);
        emit(name, &font, &second);
        let first_bytes = fs::read(&first).unwrap();
        assert_eq!(first_bytes, fs::read(&second).unwrap());
        let runtime = nuxie_binary::read_runtime_file(&first_bytes).unwrap();
        assert!(
            runtime
                .objects
                .iter()
                .filter_map(Option::as_ref)
                .any(|object| object.type_key == type_key)
        );
    }

    fs::remove_dir_all(root).unwrap();
}
