//! Text-only inline content and stable identities for explicit line breaks.
use crate::{
    Diagnostic, SourceBreak, css,
    style::{Style, WhiteSpace},
};
use scraper::ElementRef;
use std::collections::BTreeSet;

pub(crate) fn collect(
    element: ElementRef<'_>,
    style: &Style,
    path: &str,
    rules: &[css::Rule],
    ids: &mut BTreeSet<String>,
) -> Result<(String, Vec<SourceBreak>), Diagnostic> {
    let has_break = element.child_elements().any(|e| e.value().name() == "br");
    let other_elements = element.child_elements().any(|e| e.value().name() != "br");
    if has_break && !style.block_text && !style.hidden {
        return Err(Diagnostic::new(
            "unsupported-break-context",
            path,
            "Explicit br requires display:block on its text container",
        ));
    }
    if style.block_text && other_elements {
        return Err(Diagnostic::new(
            "unsupported-block-context",
            path,
            "Block display currently supports text and br only, not child containers",
        ));
    }
    let mut lines = vec![String::new()];
    let mut breaks = Vec::new();
    let mut element_index = 0;
    for child in element.children() {
        if let Some(text) = child.value().as_text() {
            lines.last_mut().unwrap().push_str(text);
        }
        if let Some(child) = ElementRef::wrap(child) {
            let child_path = format!("{path}/{element_index}");
            element_index += 1;
            if child.value().name() != "br" {
                continue;
            }
            for (name, _) in child.value().attrs() {
                if !matches!(name, "id" | "data-nuxie-id" | "lang") {
                    return Err(Diagnostic::new(
                        "unsupported-attribute",
                        &child_path,
                        format!("Unsupported br attribute: {name}"),
                    ));
                }
            }
            if !css::cascade(rules, child)?.is_empty() {
                return Err(Diagnostic::new(
                    "unsupported-break-style",
                    &child_path,
                    "Styling br is unsupported; set typography on the text container",
                ));
            }
            let id = child
                .attr("data-nuxie-id")
                .or(child.attr("id"))
                .unwrap_or(&child_path)
                .to_owned();
            if ids.len() >= crate::MAX_SOURCE_IDENTITIES {
                return Err(Diagnostic::new(
                    "input-limit",
                    &child_path,
                    "Document exceeds 8192 element and break identities",
                ));
            }
            if id.is_empty() || !ids.insert(id.clone()) {
                return Err(Diagnostic::new(
                    "duplicate-id",
                    &child_path,
                    "Empty or duplicate line-break identity",
                ));
            }
            if breaks.len() >= 4096 {
                return Err(Diagnostic::new(
                    "input-limit",
                    &child_path,
                    "Text container exceeds 4096 explicit breaks",
                ));
            }
            breaks.push(SourceBreak {
                id,
                path: child_path,
                text_offset: 0,
            });
            lines.push(String::new());
        }
    }
    // Whitespace-only anonymous flex items do not participate in layout, even
    // when the inherited white-space mode would preserve that text in a block.
    if !style.block_text
        && breaks.is_empty()
        && lines
            .iter()
            .all(|line| line.chars().all(|c| matches!(c, ' ' | '\t' | '\n' | '\r')))
    {
        return Ok((String::new(), breaks));
    }
    let mut text = String::new();
    let mut scalar_offset = 0;
    for (index, line) in lines.into_iter().enumerate() {
        if index != 0 {
            breaks[index - 1].text_offset = scalar_offset;
            scalar_offset += 1;
            text.push('\n');
        }
        let line = if style.white_space.preserves_spaces()
            || style.white_space == WhiteSpace::PreLine
        {
            if line
                .chars()
                .any(|c| c.is_ascii_control() && !matches!(c, '\t' | '\n' | '\r'))
            {
                return Err(Diagnostic::new(
                    "unsupported-text",
                    path,
                    "Preserved control characters other than tabs/line feeds/carriage returns need explicit glyph support",
                ));
            }
            let line = line.replace('\r', " ");
            if style.white_space == WhiteSpace::PreLine {
                line.split('\n')
                    .map(|segment| {
                        segment
                            .split_ascii_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                line
            }
        } else {
            line.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
        };
        scalar_offset += line.chars().count() as u32;
        text.push_str(&line);
    }
    if style.block_text && text.is_empty() && style.white_space != WhiteSpace::PreLine {
        return Err(Diagnostic::new(
            "unsupported-block-context",
            path,
            "Block display currently requires text or explicit br content",
        ));
    }
    Ok((text, breaks))
}

// Case tables are part of the compiler's output contract. A toolchain update
// must qualify native/WASM/browser behavior before changing this version.
const _: () = assert!(
    char::UNICODE_VERSION.0 == 17 && char::UNICODE_VERSION.1 == 0 && char::UNICODE_VERSION.2 == 0,
    "Requalify text-transform before changing the Unicode 17.0 case tables"
);

/// Apply presentation casing after whitespace processing. Whole-string lower
/// casing preserves contextual final sigma. Default conditional casing changes
/// scalar values but not their expansion lengths relative to char mappings.
pub(crate) fn transform(
    source: &str,
    mode: crate::style::CaseTransform,
    language: &str,
) -> Option<crate::SourceTextTransform> {
    use crate::style::CaseTransform;
    if mode == CaseTransform::None || source.is_empty() {
        return None;
    }
    // Blink applies special upper/lower rules for these primary language tags.
    // Untagged and other languages keep the existing default case tables.
    let primary = language
        .split('-')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if mode != CaseTransform::Capitalize && matches!(primary.as_str(), "tr" | "az" | "lt" | "el") {
        let lang: icu_locale_core::LanguageIdentifier =
            primary.parse().expect("known language identifier");
        let mapper = icu_casemap::CaseMapper::new();
        let (rendered, offsets) = match mode {
            CaseTransform::Uppercase => mapper.uppercase_with_scalar_offsets(source, &lang),
            CaseTransform::Lowercase => mapper.lowercase_with_scalar_offsets(source, &lang),
            _ => unreachable!(),
        };
        return Some(crate::SourceTextTransform {
            source: source.into(),
            rendered,
            scalar_offsets: offsets.into_iter().map(|offset| offset as u32).collect(),
        });
    }
    let rendered = match mode {
        CaseTransform::Uppercase => source.to_uppercase(),
        CaseTransform::Lowercase => source.to_lowercase(),
        CaseTransform::Capitalize => capitalize(source),
        CaseTransform::None => unreachable!(),
    };
    let mut scalar_offsets = Vec::with_capacity(source.chars().count() + 1);
    let mut offset = 0;
    scalar_offsets.push(offset);
    for ch in source.chars() {
        offset += match mode {
            CaseTransform::Uppercase => ch.to_uppercase().count(),
            CaseTransform::Lowercase => ch.to_lowercase().count(),
            CaseTransform::Capitalize => 1,
            CaseTransform::None => unreachable!(),
        } as u32;
        scalar_offsets.push(offset);
    }
    debug_assert_eq!(offset as usize, rendered.chars().count());
    Some(crate::SourceTextTransform {
        source: source.into(),
        rendered,
        scalar_offsets,
    })
}

fn capitalize(source: &str) -> String {
    let mapper = icu_casemap::CaseMapper::new();
    let segmenter = icu_segmenter::WordSegmenter::new_dictionary(Default::default());
    // Chromium's word iterator separates NBSP and these punctuation forms.
    // Normalize only the segmentation input; rendered text retains every source
    // character. Digits remain word heads and do not capitalize following text.
    let normalized: String = source
        .chars()
        .map(|c| {
            if matches!(
                c,
                '\u{a0}' | '.' | ':' | '\u{fe55}' | '\u{ff1a}' | '\u{ff0e}'
            ) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let mut boundaries = segmenter.segment_str(&normalized).peekable();
    let mut result = String::with_capacity(source.len());
    for ((byte, _), ch) in normalized.char_indices().zip(source.chars()) {
        while boundaries.peek().is_some_and(|boundary| *boundary < byte) {
            boundaries.next();
        }
        result.push(
            // Chromium's untagged path titlecases one UTF-16 code unit. Preserve
            // supplementary characters rather than silently using a different rule.
            if boundaries.peek() == Some(&byte) && ch <= '\u{ffff}' {
                mapper.simple_titlecase(ch)
            } else {
                ch
            },
        );
    }
    result
}

#[cfg(test)]
mod case_reference_tests {
    #[test]
    fn capitalize_matches_pinned_chromium_reference_strings() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../validation/capitalize-reference.json")).unwrap();
        let mismatches: Vec<_> = reference["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|case| {
                let source = case["source"].as_str().unwrap();
                let expected = case["rendered"].as_str().unwrap();
                let actual = super::capitalize(source);
                (actual != expected)
                    .then(|| format!("{source:?}: expected {expected:?}, got {actual:?}"))
            })
            .collect();
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }
}

#[cfg(test)]
mod locale_reference_tests {
    #[test]
    fn language_casing_matches_chromium_reference() {
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../validation/locale-reference.json")).unwrap();
        for case in reference["cases"].as_array().unwrap() {
            let mode = match case["mode"].as_str().unwrap() {
                "uppercase" => crate::style::CaseTransform::Uppercase,
                "lowercase" => crate::style::CaseTransform::Lowercase,
                _ => crate::style::CaseTransform::Capitalize,
            };
            let result = super::transform(
                case["source"].as_str().unwrap(),
                mode,
                case["lang"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(
                result.rendered, case["rendered"],
                "lang={} mode={}",
                case["lang"], case["mode"]
            );
            assert_eq!(
                result.scalar_offsets.len(),
                result.source.chars().count() + 1
            );
            assert_eq!(
                *result.scalar_offsets.last().unwrap() as usize,
                result.rendered.chars().count()
            );
            assert!(
                result
                    .scalar_offsets
                    .windows(2)
                    .all(|pair| pair[0] <= pair[1])
            );
        }
    }
}

#[cfg(test)]
mod contextual_offset_tests {
    #[test]
    fn traced_case_mapping_preserves_upstream_output_and_exact_boundaries() {
        let mapper = icu_casemap::CaseMapper::new();
        for (lang, source, upper, expected, offsets) in [
            ("tr", "I\u{307}", false, "i", vec![0, 1, 1]),
            ("lt", "I\u{301}", false, "i\u{307}\u{301}", vec![0, 2, 3]),
            ("lt", "i\u{307}", true, "I", vec![0, 1, 1]),
            ("und", "ß", true, "SS", vec![0, 2]),
            ("el", "ΟΣ", false, "ος", vec![0, 1, 2]),
            ("tr", "", false, "", vec![0]),
        ] {
            let language = lang.parse().unwrap();
            let (text, actual_offsets) = if upper {
                mapper.uppercase_with_scalar_offsets(source, &language)
            } else {
                mapper.lowercase_with_scalar_offsets(source, &language)
            };
            let original = if upper {
                mapper.uppercase_to_string(source, &language)
            } else {
                mapper.lowercase_to_string(source, &language)
            };
            assert_eq!(text, original, "upstream output changed for {lang}");
            assert_eq!(text, expected);
            assert_eq!(actual_offsets, offsets);
        }
    }
}
