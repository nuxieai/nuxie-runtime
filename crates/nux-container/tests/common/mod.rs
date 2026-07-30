#![allow(clippy::arithmetic_side_effects)]

use std::sync::LazyLock;

use nux_container::test_support::TEST_ONLY_DEV_KEYPAIR;
use nux_container::{
    AssetLocation, Assets, EmbeddedMember, Entry, FontAsset, FontContentType, FontFormat,
    FontStyle, FontStyleValue, Geometry, Identity, ImageAsset, ImageContentType, JourneyMember,
    LuauProducer, NuxPackageManifestV1, NuxPackageModel, Producer, SceneFormat, SceneMember,
    Screen, SignatureSource, TextInput, TextInputStyle,
};
use sha2::{Digest as _, Sha256};

pub const SCENE_BYTES: &[u8] = b"RIVE\x07\0synthetic-scene";
pub const JOURNEY_BYTES: &[u8] = br#"{"schemaVersion":1,"steps":[]}"#;
pub const EMBEDDED_BYTES: &[u8] = b"tiny embedded image";

pub struct GoldenSource {
    pub manifest: NuxPackageManifestV1,
    pub embedded_name: String,
}

impl GoldenSource {
    pub fn new() -> Self {
        let external_image_hash = sha256_hex(b"external image fixture");
        let external_font_hash = sha256_hex(b"external font fixture");
        let embedded_hash = sha256_hex(EMBEDDED_BYTES);
        let embedded_name = format!("assets/sha256/{embedded_hash}.png");

        Self {
            manifest: NuxPackageManifestV1 {
                version: 1,
                identity: Identity {
                    experience_id: "experience-fixture".to_owned(),
                    build_id: "build-fixture".to_owned(),
                    app_id: "app-fixture".to_owned(),
                    environment: "test".to_owned(),
                },
                producer: Producer {
                    compiler_commit: "0123456789abcdef".to_owned(),
                    compiler_version: "1.0.0-test".to_owned(),
                    runtime_revision: "runtime-test".to_owned(),
                    luau: LuauProducer {
                        revision: "luau-test".to_owned(),
                        bytecode_versions: vec![3, 6],
                    },
                    min_runtime: "0.1.0".to_owned(),
                },
                scene_format: SceneFormat { major: 7, minor: 0 },
                required_capabilities: vec![],
                scene: SceneMember {
                    member: "scene".to_owned(),
                    sha256: "0".repeat(64),
                    size_bytes: 0,
                },
                journey: JourneyMember {
                    member: "journey".to_owned(),
                    sha256: "0".repeat(64),
                    size_bytes: 0,
                    schema_version: 1,
                },
                entry: Entry {
                    screen_id: "screen-home".to_owned(),
                },
                screens: vec![Screen {
                    screen_id: "screen-home".to_owned(),
                    artboard_id: "artboard-home".to_owned(),
                    artboard_name: "Home".to_owned(),
                    width: 390.0,
                    height: 844.0,
                }],
                text_inputs: vec![TextInput {
                    input_id: "email".to_owned(),
                    screen_id: "screen-home".to_owned(),
                    artboard_id: "artboard-home".to_owned(),
                    view_node_id: "view-email".to_owned(),
                    rendered_node_id: "rendered-email".to_owned(),
                    rive_text_object_key: "text-key".to_owned(),
                    rive_text_run_object_key: "run-key".to_owned(),
                    rive_text_name: "Email".to_owned(),
                    rive_text_run_name: "EmailRun".to_owned(),
                    value: String::new(),
                    placeholder: Some("you@example.com".to_owned()),
                    editable: true,
                    geometry: Geometry {
                        x_path: "x".to_owned(),
                        y_path: "y".to_owned(),
                        width_path: "width".to_owned(),
                        height_path: "height".to_owned(),
                        rotation_path: "rotation".to_owned(),
                        scale_xpath: "scaleX".to_owned(),
                        scale_ypath: "scaleY".to_owned(),
                    },
                    style: TextInputStyle {
                        font_family: "Inter".to_owned(),
                        font_weight: "400".to_owned(),
                        font_style: FontStyleValue::Normal,
                        font_size: 16.0,
                        line_height: 20.0,
                        letter_spacing: 0.0,
                        color: 0xff11_2233,
                        font_asset_rive_unique_name: "Inter-Regular".to_owned(),
                        text_align: Some("left".to_owned()),
                    },
                    keyboard_type: Some("emailAddress".to_owned()),
                    secure_text_entry: Some(false),
                    multiline: Some(false),
                    max_length: Some(320),
                    response_field_key: Some("email".to_owned()),
                }],
                assets: Assets {
                    images: vec![
                        ImageAsset {
                            location: AssetLocation::External {
                                key: format!("assets/sha256/{external_image_hash}.png"),
                            },
                            rive_asset_id: 1,
                            rive_unique_name: "hero".to_owned(),
                            sha256: external_image_hash,
                            size_bytes: 1234,
                            content_type: ImageContentType::Png,
                            required: true,
                        },
                        ImageAsset {
                            location: AssetLocation::Embedded {
                                member: embedded_name.clone(),
                            },
                            rive_asset_id: 2,
                            rive_unique_name: "badge".to_owned(),
                            sha256: embedded_hash,
                            size_bytes: EMBEDDED_BYTES.len() as u64,
                            content_type: ImageContentType::Png,
                            required: true,
                        },
                    ],
                    fonts: vec![FontAsset {
                        location: AssetLocation::External {
                            key: format!("assets/sha256/{external_font_hash}.ttf"),
                        },
                        rive_asset_id: 3,
                        rive_unique_name: "Inter-Regular".to_owned(),
                        family: "Inter".to_owned(),
                        weight: "400".to_owned(),
                        style: FontStyle::Normal,
                        sha256: external_font_hash,
                        size_bytes: 5678,
                        content_type: FontContentType::Ttf,
                        format: FontFormat::Ttf,
                        required: true,
                    }],
                },
                members: vec![],
            },
            embedded_name,
        }
    }

    pub fn model(&self) -> NuxPackageModel<'_> {
        NuxPackageModel {
            manifest: self.manifest.clone(),
            scene: SCENE_BYTES,
            journey: JOURNEY_BYTES,
            embedded_assets: vec![EmbeddedMember {
                name: &self.embedded_name,
                bytes: EMBEDDED_BYTES,
            }],
            signature: SignatureSource::Signer(LazyLock::force(&TEST_ONLY_DEV_KEYPAIR)),
        }
    }
}

pub fn golden_bytes() -> Vec<u8> {
    let source = GoldenSource::new();
    nux_container::write_package(&source.model()).expect("write golden package")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn toc_field_positions(bytes: &[u8]) -> Vec<(String, usize, usize)> {
    let count_bytes: [u8; 4] = bytes
        .get(12..16)
        .expect("fixture header")
        .try_into()
        .expect("member count");
    let count = u32::from_le_bytes(count_bytes);
    let mut cursor = 16usize;
    let mut positions = Vec::new();
    for _ in 0..count {
        let name_len_bytes: [u8; 2] = bytes
            .get(cursor..cursor + 2)
            .expect("name length")
            .try_into()
            .expect("name length array");
        let name_len = usize::from(u16::from_le_bytes(name_len_bytes));
        cursor += 2;
        let name = std::str::from_utf8(bytes.get(cursor..cursor + name_len).expect("member name"))
            .expect("UTF-8 member name")
            .to_owned();
        cursor += name_len;
        positions.push((name, cursor, cursor + 8));
        cursor += 16;
    }
    positions
}

pub fn encode_raw(members: &[(&str, &[u8])]) -> Vec<u8> {
    let toc_size: usize = members
        .iter()
        .map(|(name, _)| 2usize + name.len() + 16)
        .sum();
    let mut offset = align(16 + toc_size);
    let mut offsets = Vec::new();
    for (_, payload) in members {
        offsets.push(offset);
        offset = align(offset + payload.len());
    }
    let final_len = members
        .last()
        .zip(offsets.last())
        .map_or(16 + toc_size, |((_, bytes), start)| start + bytes.len());
    let mut output = Vec::with_capacity(final_len);
    output.extend_from_slice(b"\x89NUX\r\n\x1a\n");
    output.extend_from_slice(&1u32.to_le_bytes());
    output.extend_from_slice(&(members.len() as u32).to_le_bytes());
    for ((name, payload), start) in members.iter().zip(&offsets) {
        output.extend_from_slice(&(name.len() as u16).to_le_bytes());
        output.extend_from_slice(name.as_bytes());
        output.extend_from_slice(&(*start as u64).to_le_bytes());
        output.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    }
    for ((_, payload), start) in members.iter().zip(&offsets) {
        output.resize(*start, 0);
        output.extend_from_slice(payload);
    }
    output
}

fn align(value: usize) -> usize {
    value.div_ceil(16) * 16
}
