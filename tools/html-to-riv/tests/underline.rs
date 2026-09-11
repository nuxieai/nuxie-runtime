use nuxie_html_to_riv::{Asset, CompileInput, CompileOutput, UnderlineSkipInk, compile};
fn scene(css: &str) -> CompileOutput {
    compile(&CompileInput {
        html: "<div id=root><p id=a>agypqj M</p><p id=b>child</p></div>".into(),
        css: css.into(),
        width: 390.0,
        height: 320.0,
        assets: [(
            "font".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        )]
        .into(),
    })
    .unwrap()
}
#[test]
fn propagated_none_retains_origin_and_local_color_adds_an_ordered_line() {
    let output = scene(
        "#root{color:red;text-decoration-line:underline;text-decoration-thickness:2px}#a{color:blue;text-decoration-line:none}#b{color:green;text-decoration-line:underline}",
    );
    let records = &output.runtime_requirements.text_underlines;
    assert_eq!(output.runtime_requirements.version, 11);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].lines.len(), 1);
    assert_eq!(records[0].lines[0].color, 0xffff0000);
    assert_eq!(records[1].lines.len(), 2);
    assert_eq!(records[1].lines[0].color, 0xffff0000);
    assert_eq!(records[1].lines[1].color, 0xff008000);
}
#[test]
fn metrics_preserve_computed_lengths_but_resolve_percentage_at_child_font() {
    for (value, expected) in [
        ("auto", 2.0),
        ("10%", 2.0),
        ("0.1em", 4.0),
        ("0.25rem", 4.0),
        ("3px", 3.0),
        ("0", 1.0),
    ] {
        let output = scene(&format!(
            "#root{{font-size:40px;line-height:60px;text-decoration-line:underline;text-decoration-thickness:{value}}}p{{font-size:20px}}"
        ));
        assert_eq!(
            output.runtime_requirements.text_underlines[0].lines[0].thickness, expected,
            "{value}"
        );
    }
    let a = scene(
        "p{text-decoration-thickness:.1em;font-size:40px;text-decoration-line:underline;font-size:20px;line-height:60px}",
    );
    let b = scene(
        "p{font-size:20px;line-height:60px;text-decoration-line:underline;text-decoration-thickness:.1em}",
    );
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
}
#[test]
fn offset_and_skip_ink_inherit_without_inheriting_the_decoration_line() {
    let output = scene(
        "#root{font-size:40px;text-underline-offset:-.1em;text-decoration-skip-ink:none;line-height:60px}p{font-size:20px;text-decoration-line:underline}#b{text-underline-offset:10%;text-decoration-skip-ink:all}",
    );
    let entries = &output.runtime_requirements.text_underlines;
    assert_eq!(entries[0].lines[0].offset, -4.0);
    assert_eq!(entries[0].lines[0].skip_ink, UnderlineSkipInk::None);
    assert_eq!(entries[1].lines[0].offset, 2.0);
    assert_eq!(entries[1].lines[0].skip_ink, UnderlineSkipInk::All);
    let output = scene(
        "#root{text-decoration-line:underline;text-underline-offset:4px}p{text-underline-offset:0}",
    );
    assert_eq!(
        output.runtime_requirements.text_underlines[0].lines[0].offset, 4.0,
        "ancestor origin keeps its offset"
    );
}
#[test]
fn wide_values_and_multiple_wrap_policies_keep_intrinsic_version() {
    let output = scene(
        "#root{text-decoration-line:underline;text-decoration-color:red}#a{text-decoration-line:inherit;text-decoration-color:initial;text-underline-offset:initial;color:blue}#b{white-space:pre-wrap;text-decoration-line:unset}",
    );
    assert_eq!(output.runtime_requirements.version, 11);
    assert_eq!(
        output.runtime_requirements.text_underlines[0].lines.len(),
        2
    );
    assert_eq!(
        output.runtime_requirements.text_underlines[0].lines[1].color,
        0xff0000ff
    );
    assert_eq!(
        output.runtime_requirements.text_underlines[1].lines.len(),
        1
    );
    output
        .runtime_requirements
        .ensure_supported(
            &output
                .runtime_requirements
                .capabilities
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        )
        .unwrap();
}
#[test]
fn unsupported_decoration_modes_and_malformed_metrics_are_rejected() {
    for css in [
        "text-decoration-line:overline",
        "text-decoration-style:dashed",
        "text-decoration-style:wavy",
        "text-underline-position:under",
        "text-underline-offset:from-font",
        "text-decoration-thickness:-1px",
        "text-decoration-thickness:2",
        "text-decoration-thickness:calc(1px + 2px)",
        "text-decoration-skip-ink:future",
    ] {
        assert!(
            compile(&CompileInput {
                html: "<div></div>".into(),
                css: format!("div{{{css}}}"),
                width: 390.0,
                height: 320.0,
                ..Default::default()
            })
            .is_err(),
            "{css}"
        );
    }
}

#[test]
fn shorthand_resets_its_longhands_but_preserves_offset_and_skip_ink() {
    let a = scene(
        "p{font:20px/30px Inter;text-underline-offset:3px;text-decoration-skip-ink:none;text-decoration:rgb(255 0 0) .1em solid underline}",
    );
    let b = scene(
        "p{font:20px/30px Inter;text-underline-offset:3px;text-decoration-skip-ink:none;text-decoration-line:underline;text-decoration-color:red;text-decoration-thickness:.1em}",
    );
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
    let a = scene("p{text-decoration:underline red 4px;text-decoration:underline}");
    let b = scene("p{text-decoration-line:underline}");
    assert_eq!(a.runtime_requirements, b.runtime_requirements);
    for value in [
        "underline underline",
        "red blue",
        "2px 3px",
        "solid solid",
        "underline wavy",
        "underline overline",
    ] {
        assert!(
            compile(&CompileInput {
                html: "<div></div>".into(),
                css: format!("div{{text-decoration:{value}}}"),
                width: 390.0,
                height: 320.0,
                ..Default::default()
            })
            .is_err()
        );
    }
}

#[test]
fn decoration_keywords_and_currentcolor_are_ascii_case_insensitive() {
    for css in [
        "p{text-decoration-line:UNDERLINE;text-decoration-color:currentColor;text-decoration-style:SOLID;text-decoration-thickness:AUTO;text-underline-offset:AUTO;text-decoration-skip-ink:AUTO;text-underline-position:AUTO}",
        "p{text-decoration:UNDERLINE SOLID currentColor AUTO}",
        "p{text-decoration:UNDERLINE RED 4PX;text-decoration:INITIAL;text-decoration:UNDERLINE}",
    ] {
        let expected = scene("p{text-decoration:underline}");
        assert_eq!(
            scene(css).runtime_requirements,
            expected.runtime_requirements,
            "{css}"
        );
    }
    let invisible =
        scene("p{color:transparent;text-decoration:underline;text-decoration-color:currentColor}");
    assert!(
        invisible
            .runtime_requirements
            .text_underlines
            .iter()
            .flat_map(|entry| &entry.lines)
            .all(|line| line.color == 0)
    );
}

#[test]
fn from_font_distinguishes_signed_metrics_from_absent_metrics() {
    let original = include_bytes!("assets/NuxieJapaneseFixture-Regular.otf");
    let table = |tag: &[u8; 4]| {
        let count = u16::from_be_bytes(original[4..6].try_into().unwrap()) as usize;
        (0..count)
            .find_map(|index| {
                let entry = 12 + index * 16;
                (&original[entry..entry + 4] == tag).then(|| {
                    (
                        entry,
                        u32::from_be_bytes(original[entry + 8..entry + 12].try_into().unwrap())
                            as usize,
                        u32::from_be_bytes(original[entry + 12..entry + 16].try_into().unwrap())
                            as usize,
                    )
                })
            })
            .unwrap()
    };
    let checksum = |bytes: &[u8]| {
        bytes.chunks(4).fold(0_u32, |sum, chunk| {
            let mut word = [0_u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    };
    let post = table(b"post");
    let head = table(b"head");
    for thickness in [Some(-200_i16), Some(-20), Some(0), Some(1), Some(50), None] {
        let mut bytes = original.to_vec();
        let mut head_entry = head.0;
        if let Some(thickness) = thickness {
            bytes[post.1 + 10..post.1 + 12].copy_from_slice(&thickness.to_be_bytes());
        } else {
            let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) - 1;
            let end = 12 + usize::from(count) * 16;
            bytes.copy_within(post.0 + 16..end + 16, post.0);
            bytes[end..end + 16].fill(0);
            bytes[4..6].copy_from_slice(&count.to_be_bytes());
            let exponent = count.ilog2() as u16;
            let search_range = 16 * 2_u16.pow(u32::from(exponent));
            bytes[6..8].copy_from_slice(&search_range.to_be_bytes());
            bytes[8..10].copy_from_slice(&exponent.to_be_bytes());
            bytes[10..12].copy_from_slice(&(count * 16 - search_range).to_be_bytes());
            if head_entry > post.0 {
                head_entry -= 16;
            }
        }
        bytes[head.1 + 8..head.1 + 12].fill(0);
        let mut changed_tables = vec![(head_entry, head.1, head.2)];
        if thickness.is_some() {
            changed_tables.push(post);
        }
        for (entry, offset, length) in changed_tables {
            let sum = checksum(&bytes[offset..offset + length]);
            bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
        }
        let adjustment = 0xb1b0afba_u32.wrapping_sub(checksum(&bytes));
        bytes[head.1 + 8..head.1 + 12].copy_from_slice(&adjustment.to_be_bytes());
        assert_eq!(
            ttf_parser::Face::parse(&bytes, 0)
                .unwrap()
                .underline_metrics()
                .map(|m| m.thickness),
            thickness
        );
        for mode in ["auto", "from-font"] {
            let output = compile(&CompileInput {
                html: "<p>agypqj</p>".into(),
                css: format!(
                    "p{{font-size:24px;line-height:40px;text-decoration:underline {mode}}}"
                ),
                width: 240.0,
                height: 80.0,
                assets: [(
                    "font".into(),
                    Asset::Font {
                        family: "Inter".into(),
                        weight: 400,
                        bytes: bytes.clone(),
                    },
                )]
                .into(),
            })
            .unwrap();
            let expected = match (mode, thickness) {
                ("auto", _) | (_, None) => 2.4,
                (_, Some(thickness)) => (f32::from(thickness) * 24.0 / 1000.0).max(1.0),
            };
            assert!(
                (output.runtime_requirements.text_underlines[0].lines[0].thickness - expected)
                    .abs()
                    < 0.00001
            );
        }
    }
}
