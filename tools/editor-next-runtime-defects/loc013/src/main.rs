use anyhow::{Context, Result, bail, ensure};
use harfrust::{
    Direction, FontRef as HarfFontRef, ShapeOptions, ShaperData, ShaperInstance, Tag as HarfTag,
    UnicodeBuffer,
};
use nuxie::{File, OwnedArtboardInstance, RecordingFactory};
use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
use serde::Serialize;
use sha2::{Digest, Sha256};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::pen::PathStyle;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::setting::VariationSetting;
use skrifa::{FontRef as SkrifaFontRef, GlyphId, MetadataProvider, Tag as SkrifaTag};
use std::env;
use std::fs;
use std::sync::Arc;

const FONT_SIZE: f32 = 17.0;
const SCALE: f32 = 2048.0;
const WEIGHTS: [f32; 4] = [400.0, 500.0, 600.0, 700.0];

#[derive(Default)]
struct JsonPen {
    commands: Vec<Vec<serde_json::Value>>,
    current: Option<(f32, f32)>,
    contour_start: Option<(f32, f32)>,
}

impl JsonPen {
    fn normalize(x: f32, y: f32) -> (f32, f32) {
        (x / SCALE, -y / SCALE)
    }

    fn point(x: f32, y: f32) -> [serde_json::Value; 2] {
        [
            serde_json::json!(f64::from(x)),
            serde_json::json!(f64::from(y)),
        ]
    }
}

impl OutlinePen for JsonPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let pair = Self::normalize(x, y);
        let [x, y] = Self::point(pair.0, pair.1);
        self.commands.push(vec![serde_json::json!("M"), x, y]);
        self.current = Some(pair);
        self.contour_start = Some(pair);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let pair = Self::normalize(x, y);
        let [x, y] = Self::point(pair.0, pair.1);
        self.commands.push(vec![serde_json::json!("L"), x, y]);
        self.current = Some(pair);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let Some(current) = self.current else {
            self.move_to(x, y);
            return;
        };
        let control = Self::normalize(cx0, cy0);
        let end = Self::normalize(x, y);
        let t = 2.0 / 3.0;
        let c1 = (
            current.0 + (control.0 - current.0) * t,
            current.1 + (control.1 - current.1) * t,
        );
        let c2 = (
            end.0 + (control.0 - end.0) * t,
            end.1 + (control.1 - end.1) * t,
        );
        let [c1x, c1y] = Self::point(c1.0, c1.1);
        let [c2x, c2y] = Self::point(c2.0, c2.1);
        let [x, y] = Self::point(end.0, end.1);
        self.commands
            .push(vec![serde_json::json!("C"), c1x, c1y, c2x, c2y, x, y]);
        self.current = Some(end);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let control0 = Self::normalize(cx0, cy0);
        let control1 = Self::normalize(cx1, cy1);
        let end = Self::normalize(x, y);
        let [cx0, cy0] = Self::point(control0.0, control0.1);
        let [cx1, cy1] = Self::point(control1.0, control1.1);
        let [x, y] = Self::point(end.0, end.1);
        self.commands
            .push(vec![serde_json::json!("C"), cx0, cy0, cx1, cy1, x, y]);
        self.current = Some(end);
    }

    fn close(&mut self) {
        if let (Some(current), Some(start)) = (self.current, self.contour_start)
            && ((current.0 - start.0).abs() > f32::EPSILON
                || (current.1 - start.1).abs() > f32::EPSILON)
        {
            let [x, y] = Self::point(start.0, start.1);
            self.commands.push(vec![serde_json::json!("L"), x, y]);
        }
        self.commands.push(vec![serde_json::json!("Z")]);
        self.current = self.contour_start;
    }
}

#[derive(Serialize)]
struct GlyphResult {
    id: u32,
    advance: f32,
    outline: Vec<Vec<serde_json::Value>>,
}

#[derive(Serialize)]
struct WeightResult {
    weight: f32,
    axis_value: f32,
    text: String,
    glyphs: Vec<GlyphResult>,
}

#[derive(Serialize)]
struct DirectReport {
    font_sha256: String,
    font_bytes: usize,
    face_index: u32,
    axis_tag: String,
    results: Vec<WeightResult>,
}

fn direct(font_path: &str) -> Result<()> {
    let bytes = fs::read(font_path).with_context(|| format!("read {font_path}"))?;
    let harf_font = HarfFontRef::new(&bytes).context("harfrust parse")?;
    let skrifa_font = SkrifaFontRef::new(&bytes).context("skrifa parse")?;
    let outline_glyphs = skrifa_font.outline_glyphs();
    let mut results = Vec::new();

    for weight in WEIGHTS {
        let text = format!("{weight:.0} Inter sample");
        let variation = [(HarfTag::new(b"wght"), weight)];
        let shaper_instance = ShaperInstance::from_variations(&harf_font, variation);
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data
            .shaper(&harf_font)
            .instance(Some(&shaper_instance))
            .build();
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(&text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.set_script(harfrust::script::LATIN);
        buffer.guess_segment_properties();
        let shaped = shaper.shape(buffer, ShapeOptions::new().scale(Some(SCALE as i32)));

        let location = skrifa_font
            .axes()
            .location([VariationSetting::new(SkrifaTag::new(b"wght"), weight)]);
        let mut glyphs = Vec::new();
        for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
            let mut pen = JsonPen::default();
            if let Some(outline) = outline_glyphs.get(GlyphId::new(info.glyph_id)) {
                let settings =
                    DrawSettings::unhinted(Size::new(SCALE), LocationRef::from(&location))
                        .with_path_style(PathStyle::HarfBuzz);
                outline
                    .draw(settings, &mut pen)
                    .with_context(|| format!("draw glyph {}", info.glyph_id))?;
            }
            glyphs.push(GlyphResult {
                id: info.glyph_id,
                advance: position.x_advance as f32 * FONT_SIZE / SCALE,
                outline: pen.commands,
            });
        }
        results.push(WeightResult {
            weight,
            axis_value: weight,
            text,
            glyphs,
        });
    }

    let report = DirectReport {
        font_sha256: format!("{:x}", Sha256::digest(&bytes)),
        font_bytes: bytes.len(),
        face_index: 0,
        axis_tag: "wght".into(),
        results,
    };
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn encode_authoring_records(records: Vec<AuthoringRecord>) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 0x4e55_5849);
    push_var_uint(&mut bytes, 0);
    for record in records {
        push_var_uint(&mut bytes, u64::from(record.type_key));
        for AuthoringProperty { key, value } in record.properties {
            push_var_uint(&mut bytes, u64::from(key));
            match value {
                AuthoringValue::Bool(value) => bytes.push(u8::from(value)),
                AuthoringValue::Bytes(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(&value);
                }
                AuthoringValue::Color(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                AuthoringValue::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                AuthoringValue::String(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value.as_bytes());
                }
                AuthoringValue::Uint(value) => push_var_uint(&mut bytes, value),
            }
        }
        push_var_uint(&mut bytes, 0);
    }
    bytes
}

fn record(type_name: &str, properties: Vec<(&str, AuthoringValue)>) -> AuthoringRecord {
    let definition = nuxie_schema::definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    let properties = properties
        .into_iter()
        .map(|(property_name, value)| {
            let property = std::iter::once(definition.name)
                .chain(definition.ancestors.iter().copied())
                .filter_map(nuxie_schema::definition_by_name)
                .flat_map(|owner| owner.properties)
                .find(|property| property.name == property_name)
                .unwrap_or_else(|| panic!("missing {type_name}.{property_name}"));
            AuthoringProperty {
                key: property.key.int,
                value,
            }
        })
        .collect();
    AuthoringRecord {
        type_key: definition.type_key.int,
        properties,
    }
}

fn scene(font_path: &str, riv_path: &str, stream_path: &str) -> Result<()> {
    let font_bytes = fs::read(font_path).with_context(|| format!("read {font_path}"))?;
    let mut records = vec![
        record("Backboard", Vec::new()),
        record(
            "FontAsset",
            vec![
                ("name", AuthoringValue::String("Inter Variable".into())),
                ("assetId", AuthoringValue::Uint(0)),
            ],
        ),
        record(
            "FileAssetContents",
            vec![("bytes", AuthoringValue::Bytes(font_bytes.clone()))],
        ),
        record(
            "Artboard",
            vec![
                (
                    "name",
                    AuthoringValue::String("LOC013 Variable Font".into()),
                ),
                ("width", AuthoringValue::Double(240.0)),
                ("height", AuthoringValue::Double(112.0)),
            ],
        ),
    ];
    let mut local_id = 1_u64;
    for (index, weight) in WEIGHTS.into_iter().enumerate() {
        let text_id = local_id;
        let style_id = local_id + 1;
        let fill_id = local_id + 2;
        records.extend([
            record(
                "Text",
                vec![
                    (
                        "name",
                        AuthoringValue::String(format!("Weight {weight:.0}")),
                    ),
                    ("x", AuthoringValue::Double(8.0)),
                    ("y", AuthoringValue::Double(10.0 + index as f32 * 22.0)),
                    ("sizingValue", AuthoringValue::Uint(2)),
                    ("width", AuthoringValue::Double(224.0)),
                    ("height", AuthoringValue::Double(22.0)),
                    ("wrapValue", AuthoringValue::Uint(1)),
                    ("overflowValue", AuthoringValue::Uint(0)),
                ],
            ),
            record(
                "TextStylePaint",
                vec![
                    ("parentId", AuthoringValue::Uint(text_id)),
                    ("fontSize", AuthoringValue::Double(17.0)),
                    ("lineHeight", AuthoringValue::Double(22.0)),
                    ("letterSpacing", AuthoringValue::Double(0.0)),
                    ("fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            record("Fill", vec![("parentId", AuthoringValue::Uint(style_id))]),
            record(
                "SolidColor",
                vec![
                    ("parentId", AuthoringValue::Uint(fill_id)),
                    ("colorValue", AuthoringValue::Color(0xff0f_172a)),
                ],
            ),
            record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(text_id)),
                    (
                        "text",
                        AuthoringValue::String(format!("{weight:.0} Inter sample")),
                    ),
                    ("styleId", AuthoringValue::Uint(style_id)),
                ],
            ),
            record(
                "TextStyleAxis",
                vec![
                    ("parentId", AuthoringValue::Uint(style_id)),
                    ("tag", AuthoringValue::Uint(0x7767_6874)),
                    ("axisValue", AuthoringValue::Double(weight)),
                ],
            ),
        ]);
        local_id += 6;
    }

    let bytes = encode_authoring_records(records);
    fs::write(riv_path, &bytes)?;
    let decoded = nuxie_binary::read_runtime_file(&bytes)?;
    let decoded_axes = (0..decoded.object_count())
        .filter_map(|id| decoded.object(id))
        .filter(|object| object.type_name == "TextStyleAxis")
        .map(|object| {
            (
                object.uint_property("tag"),
                object.double_property("axisValue"),
                object.uint_property("parentId"),
            )
        })
        .collect::<Vec<_>>();
    ensure!(
        decoded_axes
            == vec![
                (Some(0x7767_6874), Some(400.0), Some(2)),
                (Some(0x7767_6874), Some(500.0), Some(8)),
                (Some(0x7767_6874), Some(600.0), Some(14)),
                (Some(0x7767_6874), Some(700.0), Some(20)),
            ],
        "decoded TextStyleAxis records differ: {decoded_axes:?}"
    );

    // OwnedArtboardInstance's public API deliberately accepts Arc<File>; this
    // single-threaded evidence driver does not use the value across threads.
    #[allow(clippy::arc_with_non_send_sync)]
    let file = Arc::new(File::import(&bytes)?);
    let mut instance = OwnedArtboardInstance::instantiate(file, 0)?;
    let mut factory = RecordingFactory::new();
    factory.source("loc013.riv", "LOC013 Variable Font", "LOC013 Variable Font");
    factory.frame_size(240, 112);
    factory.add_sample(0.0);
    let mut renderer = factory.make_renderer();
    instance.draw(&mut factory, &mut renderer)?;
    factory.add_frame();
    fs::write(stream_path, factory.stream())?;
    eprintln!(
        "riv_bytes={} riv_sha256={:x} font_sha256={:x} axes={decoded_axes:?}",
        bytes.len(),
        Sha256::digest(&bytes),
        Sha256::digest(&font_bytes),
    );
    Ok(())
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("direct") => {
            let font_path = args.next().context("usage: loc013-evidence direct FONT")?;
            ensure!(args.next().is_none(), "unexpected direct arguments");
            direct(&font_path)
        }
        Some("scene") => {
            let font_path = args
                .next()
                .context("usage: loc013-evidence scene FONT OUT_RIV OUT_STREAM")?;
            let riv_path = args.next().context("missing OUT_RIV")?;
            let stream_path = args.next().context("missing OUT_STREAM")?;
            ensure!(args.next().is_none(), "unexpected scene arguments");
            scene(&font_path, &riv_path, &stream_path)
        }
        _ => bail!("usage: loc013-evidence direct FONT | scene FONT OUT_RIV OUT_STREAM"),
    }
}
