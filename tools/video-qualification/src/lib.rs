fn video_scene() -> Vec<u8> {
    video_scene_with_media(None)
}
fn video_scene_with_media(embedded: Option<&[u8]>) -> Vec<u8> {
    video_scene_with_dimensions(embedded, 64, 32)
}
fn video_scene_with_dimensions(embedded: Option<&[u8]>, width: u32, height: u32) -> Vec<u8> {
    use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
    let mut records = vec![
        R {
            type_key: 23,
            properties: vec![],
        },
        R {
            type_key: 60000,
            properties: vec![
                P {
                    key: 208,
                    value: V::Double(width as f32),
                },
                P {
                    key: 207,
                    value: V::Double(height as f32),
                },
            ],
        },
        R {
            type_key: 1,
            properties: vec![
                P {
                    key: 7,
                    value: V::Double(64.0),
                },
                P {
                    key: 8,
                    value: V::Double(32.0),
                },
            ],
        },
        R {
            type_key: 60001,
            properties: vec![
                P { key: 60013, value: V::String(r#"{"version":1,"language":"en","cues":[{"start":0,"end":1,"text":"Red scene"},{"start":1,"end":2,"text":"Blue scene"}]}"#.into()) },
                P {
                    key: 5,
                    value: V::Uint(0),
                },
                P {
                    key: 16,
                    value: V::Double(32.0 / width as f32),
                },
                P {
                    key: 17,
                    value: V::Double(16.0 / height as f32),
                },
                P {
                    key: 206,
                    value: V::Uint(0),
                },
                P {
                    key: 13,
                    value: V::Double(32.0),
                },
                P {
                    key: 14,
                    value: V::Double(16.0),
                },
            ],
        },
        // Vector UI occupies a patch inside the transformed video's bounds.
        R {
            type_key: 3,
            properties: vec![P {
                key: 5,
                value: V::Uint(0),
            }],
        },
        R {
            type_key: 20,
            properties: vec![P {
                key: 5,
                value: V::Uint(1),
            }],
        },
        R {
            type_key: 18,
            properties: vec![
                P {
                    key: 5,
                    value: V::Uint(2),
                },
                P {
                    key: 37,
                    value: V::Color(0xff00ff00),
                },
            ],
        },
        R {
            type_key: 7,
            properties: vec![
                P {
                    key: 5,
                    value: V::Uint(1),
                },
                P {
                    key: 13,
                    value: V::Double(20.0),
                },
                P {
                    key: 14,
                    value: V::Double(12.0),
                },
                P {
                    key: 20,
                    value: V::Double(8.0),
                },
                P {
                    key: 21,
                    value: V::Double(8.0),
                },
            ],
        },
    ];
    // RIV's drawable order places the earlier sibling above later siblings.
    let mut video = records.remove(3);
    // Artboard object IDs: UI shape 1, fill 2, color 3, rectangle 4,
    // video 5, clipping group 6, mask shape 7, mask rectangle 8, clip 9.
    video
        .properties
        .iter_mut()
        .find(|p| p.key == 5)
        .unwrap()
        .value = V::Uint(6);
    records.push(video);
    records.extend([
        R {
            type_key: 2,
            properties: vec![P {
                key: 5,
                value: V::Uint(0),
            }],
        },
        R {
            type_key: 3,
            properties: vec![P {
                key: 5,
                value: V::Uint(0),
            }],
        },
        R {
            type_key: 7,
            properties: vec![
                P {
                    key: 5,
                    value: V::Uint(7),
                },
                P {
                    key: 13,
                    value: V::Double(32.0),
                },
                P {
                    key: 14,
                    value: V::Double(16.0),
                },
                P {
                    key: 20,
                    value: V::Double(24.0),
                },
                P {
                    key: 21,
                    value: V::Double(12.0),
                },
            ],
        },
        R {
            type_key: 42,
            properties: vec![
                P {
                    key: 5,
                    value: V::Uint(6),
                },
                P {
                    key: 92,
                    value: V::Uint(7),
                },
            ],
        },
    ]);
    if let Some(bytes) = embedded {
        records.insert(
            2,
            R {
                type_key: 106,
                properties: vec![P {
                    key: 212,
                    value: V::Bytes(bytes.to_vec()),
                }],
            },
        );
    }
    nuxie_binary::encode_runtime_file(
        &nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap(),
    )
    .unwrap()
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use apple::*;
#[cfg(target_os = "android")]
mod android;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod browser;

/// Independent pixel locations prove the decoded surface obeys scene scale
/// and the vector UI remains above it, rather than being a player overlay.
fn verify_composition(pixels: &[u8]) -> Result<(), String> {
    if pixels.len() != 64 * 32 * 4 {
        return Err("wrong surface size".into());
    }
    let outside = &pixels[(2 * 64 + 2) * 4..][..4];
    // (18,16) is inside the video's scaled bounds but outside the clip.
    // (32,9) tests the other clipping axis, away from antialiasing edges.
    for (x, y) in [(18, 16), (32, 9), (46, 16), (32, 23)] {
        let clipped = &pixels[(y * 64 + x) * 4..][..4];
        if clipped[..3].iter().any(|v| *v > 8) {
            return Err(format!(
                "video escaped authored clip at ({x},{y}): {clipped:?}"
            ));
        }
    }
    let overlay = &pixels[(12 * 64 + 20) * 4..][..4];
    if outside[..3].iter().any(|v| *v > 8) {
        return Err(format!("video escaped authored scale: {outside:?}"));
    }
    if overlay[0] > 8 || overlay[1] < 240 || overlay[2] > 8 {
        let green: Vec<_> = pixels
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, p)| p[1] > 240 && p[0] < 8 && p[2] < 8)
            .map(|(i, _)| (i % 64, i / 64))
            .take(8)
            .collect();
        return Err(format!(
            "vector UI did not overlay video: pixel={overlay:?}, green pixels={green:?}"
        ));
    }
    Ok(())
}
