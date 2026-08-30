#![cfg(feature = "scripting")]

use nuxie::{
    ArtboardInstance, FileImportLimits, PersistentFactory, RecordingFactory, ScriptExecutionLimits,
    ScriptValue, import_unsigned_scripted,
};
use sha2::{Digest as _, Sha256};

const FIXTURE: &str =
    include_str!("../../../fixtures/parity/nested-scripted-vector-source-identity.riv.b64");
const FIXTURE_SHA256: &str = "cded91dc4cda831614ac4017a6d97db5d5c220ac2b63b18e57dc2c9c7026d780";

fn decode_base64_fixture(encoded: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut word = 0_u32;
    let mut sextets = 0_u8;
    for byte in encoded.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => byte.checked_sub(b'A').expect("matched ASCII range"),
            b'a'..=b'z' => byte
                .checked_sub(b'a')
                .and_then(|value| value.checked_add(26))
                .expect("matched ASCII range"),
            b'0'..=b'9' => byte
                .checked_sub(b'0')
                .and_then(|value| value.checked_add(52))
                .expect("matched ASCII range"),
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64 fixture byte"),
        };
        word = (word << 6) | u32::from(value);
        sextets = sextets.checked_add(1).expect("base64 quantum overflow");
        if sextets == 4 {
            output.extend_from_slice(&word.to_be_bytes()[1..]);
            word = 0;
            sextets = 0;
        }
    }
    if sextets == 2 {
        output.push((word >> 4) as u8);
    } else if sextets == 3 {
        output.extend_from_slice(&(word >> 2).to_be_bytes()[2..]);
    }
    output
}

fn draw(factory: &PersistentFactory<RecordingFactory>, artboard: &ArtboardInstance) -> String {
    factory.borrow_mut().clear();
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    factory.borrow().canonical_recording().stream().to_owned()
}

#[test]
fn nested_script_input_uses_source_artboard_and_local_object_identity() {
    let bytes = decode_base64_fixture(FIXTURE);
    assert_eq!(bytes.len(), 1_617);
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), FIXTURE_SHA256);

    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let scripted = import_unsigned_scripted(
        &bytes,
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("nested scripted source-identity fixture imports");
    let mut root = ArtboardInstance::from_native(scripted.native_file().clone(), 0)
        .expect("root source artboard instance");

    root.advance(0.0).expect("root initializes");
    let initial = draw(&factory, &root);
    assert!(
        initial.contains("transform matrix=[1,0,0,1,24,18]"),
        "authored offset must draw through the nested scripted occurrence:\n{initial}"
    );

    assert_eq!(
        root.set_script_input_for_source_occurrences_if_changed(
            99,
            1,
            "offset",
            ScriptValue::Number(48.0),
        )
        .expect("missing source lookup is not a script error"),
        None,
    );
    assert_eq!(
        root.set_script_input_for_source_occurrences_if_changed(
            1,
            99,
            "offset",
            ScriptValue::Number(48.0),
        )
        .expect("missing source-local object lookup is not a script error"),
        None,
    );
    assert_eq!(
        root.set_script_input_for_source_occurrences_if_changed(
            1,
            1,
            "offset",
            ScriptValue::Number(24.0),
        )
        .expect("authored input is readable on the retained nested occurrence"),
        Some(false),
    );
    assert_eq!(
        root.set_script_input_for_source_occurrences_if_changed(
            1,
            1,
            "offset",
            ScriptValue::Number(48.0),
        )
        .expect("source-local nested script input accepts its changed value"),
        Some(true),
    );
    assert_eq!(
        root.set_script_input_for_source_occurrences_if_changed(
            1,
            1,
            "offset",
            ScriptValue::Number(48.0),
        )
        .expect("unchanged nested script input remains readable"),
        Some(false),
    );

    let changed = draw(&factory, &root);
    assert!(
        changed.contains("transform matrix=[1,0,0,1,48,18]"),
        "changed input must alter the retained nested scripted draw:\n{changed}"
    );
    assert_ne!(initial, changed);
}
