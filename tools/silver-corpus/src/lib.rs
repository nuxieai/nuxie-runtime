use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

pub const EXPECTED_ENTRIES: usize = 238;
pub const EXPECTED_RUNTIME: usize = 195;
pub const EXPECTED_SCRIPTED: usize = 41;
pub const MAX_PROVENANCE_UNKNOWN: usize = 2;
pub const SRIV_EPSILON: f32 = 0.001;
pub const UPSTREAM_REF: &str = "d788e8ec6e8b598526607d6a1e8818e8b637b60c";

#[derive(Debug, Clone, PartialEq)]
pub struct Sriv {
    pub version: u64,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub offset: usize,
    pub frame: usize,
    pub kind: OpKind,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum OpKind {
    MakeRenderBuffer = 0,
    MakeLinearGradient = 1,
    MakeRadialGradient = 2,
    MakeRenderPath = 3,
    MakeRenderPaint = 5,
    DecodeImage = 6,
    Save = 7,
    Restore = 8,
    Transform = 9,
    DrawPath = 10,
    ClipPath = 11,
    DrawImage = 12,
    DrawImageMesh = 13,
    SetVertexBufferData = 14,
    SetIndexBufferData = 15,
    AddRawPath = 16,
    Rewind = 17,
    FillRule = 18,
    Style = 20,
    Color = 21,
    Thickness = 22,
    Join = 23,
    Cap = 24,
    Feather = 25,
    BlendMode = 26,
    Shader = 27,
    Frame = 28,
    FrameSize = 29,
    ModulateOpacity = 30,
}

impl OpKind {
    fn parse(value: u64, offset: usize) -> Result<Self, ParseError> {
        Ok(match value {
            0 => Self::MakeRenderBuffer,
            1 => Self::MakeLinearGradient,
            2 => Self::MakeRadialGradient,
            3 => Self::MakeRenderPath,
            5 => Self::MakeRenderPaint,
            6 => Self::DecodeImage,
            7 => Self::Save,
            8 => Self::Restore,
            9 => Self::Transform,
            10 => Self::DrawPath,
            11 => Self::ClipPath,
            12 => Self::DrawImage,
            13 => Self::DrawImageMesh,
            14 => Self::SetVertexBufferData,
            15 => Self::SetIndexBufferData,
            16 => Self::AddRawPath,
            17 => Self::Rewind,
            18 => Self::FillRule,
            20 => Self::Style,
            21 => Self::Color,
            22 => Self::Thickness,
            23 => Self::Join,
            24 => Self::Cap,
            25 => Self::Feather,
            26 => Self::BlendMode,
            27 => Self::Shader,
            28 => Self::Frame,
            29 => Self::FrameSize,
            30 => Self::ModulateOpacity,
            _ => {
                return Err(ParseError::new(
                    offset,
                    format!("unknown SRIV operation {value}"),
                ));
            }
        })
    }
}

impl Display for OpKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::MakeRenderBuffer => "makeRenderBuffer",
            Self::MakeLinearGradient => "makeLinearGradient",
            Self::MakeRadialGradient => "makeRadialGradient",
            Self::MakeRenderPath => "makeRenderPath",
            Self::MakeRenderPaint => "makeRenderPaint",
            Self::DecodeImage => "decodeImage",
            Self::Save => "save",
            Self::Restore => "restore",
            Self::Transform => "transform",
            Self::DrawPath => "drawPath",
            Self::ClipPath => "clipPath",
            Self::DrawImage => "drawImage",
            Self::DrawImageMesh => "drawImageMesh",
            Self::SetVertexBufferData => "setVertexBufferData",
            Self::SetIndexBufferData => "setIndexBufferData",
            Self::AddRawPath => "addRawPath",
            Self::Rewind => "rewind",
            Self::FillRule => "fillRule",
            Self::Style => "style",
            Self::Color => "color",
            Self::Thickness => "thickness",
            Self::Join => "join",
            Self::Cap => "cap",
            Self::Feather => "feather",
            Self::BlendMode => "blendMode",
            Self::Shader => "shader",
            Self::Frame => "frame",
            Self::FrameSize => "frameSize",
            Self::ModulateOpacity => "modulateOpacity",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: &'static str,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Uint(u64),
    Float(u32),
    Vec2(u32, u32),
    Bytes(Vec<u8>),
}

impl Value {
    fn describe(&self) -> String {
        match self {
            Self::Uint(value) => value.to_string(),
            Self::Float(bits) => describe_float(*bits),
            Self::Vec2(x, y) => format!("({}, {})", describe_float(*x), describe_float(*y)),
            Self::Bytes(bytes) => format!("{} bytes", bytes.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl ParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Difference {
    pub frame: usize,
    pub operation: usize,
    pub kind: Option<OpKind>,
    pub field: Option<&'static str>,
    pub message: String,
}

impl Display for Difference {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "frame {}, op {}", self.frame, self.operation)?;
        if let Some(kind) = self.kind {
            write!(formatter, " ({kind})")?;
        }
        if let Some(field) = self.field {
            write!(formatter, ", field {field}")?;
        }
        write!(formatter, ": {}", self.message)
    }
}

pub fn parse_sriv(bytes: &[u8]) -> Result<Sriv, ParseError> {
    let mut reader = Reader::new(bytes);
    let header = reader.read_bytes(4, "header")?;
    if header != b"SRIV" {
        return Err(ParseError::new(0, "invalid header; expected SRIV"));
    }
    let version = reader.read_varuint("version")?;
    if version != 1 {
        return Err(ParseError::new(
            4,
            format!("unsupported SRIV version {version}"),
        ));
    }

    let mut operations = Vec::new();
    let mut buffers = HashMap::<u64, BufferInfo>::new();
    let mut frame = 0usize;
    while !reader.is_empty() {
        let offset = reader.position;
        let raw_kind = reader.read_varuint("operation")?;
        let kind = OpKind::parse(raw_kind, offset)?;
        let mut fields = Vec::new();
        parse_fields(kind, &mut reader, &mut fields, &mut buffers)?;
        operations.push(Operation {
            offset,
            frame,
            kind,
            fields,
        });
        if kind == OpKind::Frame {
            frame += 1;
        }
    }
    Ok(Sriv {
        version,
        operations,
    })
}

pub fn compare_sriv(expected: &Sriv, actual: &Sriv) -> Result<(), Difference> {
    if expected.version != actual.version {
        return Err(Difference {
            frame: 0,
            operation: 0,
            kind: None,
            field: Some("version"),
            message: format!("expected {}, got {}", expected.version, actual.version),
        });
    }

    for (index, (expected_op, actual_op)) in expected
        .operations
        .iter()
        .zip(&actual.operations)
        .enumerate()
    {
        if expected_op.kind != actual_op.kind {
            return Err(Difference {
                frame: expected_op.frame,
                operation: index,
                kind: Some(expected_op.kind),
                field: None,
                message: format!("expected {}, got {}", expected_op.kind, actual_op.kind),
            });
        }
        if expected_op.fields.len() != actual_op.fields.len() {
            return Err(Difference {
                frame: expected_op.frame,
                operation: index,
                kind: Some(expected_op.kind),
                field: None,
                message: format!(
                    "expected {} fields, got {}",
                    expected_op.fields.len(),
                    actual_op.fields.len()
                ),
            });
        }
        for (expected_field, actual_field) in expected_op.fields.iter().zip(&actual_op.fields) {
            if expected_field.name != actual_field.name {
                return Err(Difference {
                    frame: expected_op.frame,
                    operation: index,
                    kind: Some(expected_op.kind),
                    field: Some(expected_field.name),
                    message: format!(
                        "expected field {}, got {}",
                        expected_field.name, actual_field.name
                    ),
                });
            }
            if !values_match(&expected_field.value, &actual_field.value) {
                return Err(Difference {
                    frame: expected_op.frame,
                    operation: index,
                    kind: Some(expected_op.kind),
                    field: Some(expected_field.name),
                    message: format!(
                        "expected {}, got {}",
                        expected_field.value.describe(),
                        actual_field.value.describe()
                    ),
                });
            }
        }
    }

    if expected.operations.len() != actual.operations.len() {
        let index = expected.operations.len().min(actual.operations.len());
        let operation = expected
            .operations
            .get(index)
            .or_else(|| actual.operations.get(index));
        return Err(Difference {
            frame: operation.map_or(0, |operation| operation.frame),
            operation: index,
            kind: operation.map(|operation| operation.kind),
            field: None,
            message: format!(
                "expected {} operations, got {}",
                expected.operations.len(),
                actual.operations.len()
            ),
        });
    }
    Ok(())
}

fn values_match(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Uint(expected), Value::Uint(actual)) => expected == actual,
        (Value::Bytes(expected), Value::Bytes(actual)) => expected == actual,
        (Value::Float(expected), Value::Float(actual)) => floats_match(*expected, *actual),
        (Value::Vec2(expected_x, expected_y), Value::Vec2(actual_x, actual_y)) => {
            let expected_x = f32::from_bits(*expected_x);
            let expected_y = f32::from_bits(*expected_y);
            let actual_x = f32::from_bits(*actual_x);
            let actual_y = f32::from_bits(*actual_y);
            if !ordinary_finite(expected_x, actual_x) || !ordinary_finite(expected_y, actual_y) {
                expected_x.to_bits() == actual_x.to_bits()
                    && expected_y.to_bits() == actual_y.to_bits()
            } else {
                (expected_x - actual_x).hypot(expected_y - actual_y) <= SRIV_EPSILON
            }
        }
        _ => false,
    }
}

fn floats_match(expected: u32, actual: u32) -> bool {
    let expected_value = f32::from_bits(expected);
    let actual_value = f32::from_bits(actual);
    if ordinary_finite(expected_value, actual_value) {
        (expected_value - actual_value).abs() <= SRIV_EPSILON
    } else {
        expected == actual
    }
}

fn ordinary_finite(expected: f32, actual: f32) -> bool {
    expected.is_finite()
        && actual.is_finite()
        && !(expected == 0.0 && expected.is_sign_negative())
        && !(actual == 0.0 && actual.is_sign_negative())
}

fn describe_float(bits: u32) -> String {
    let value = f32::from_bits(bits);
    if value.is_nan() || value.is_infinite() || (value == 0.0 && value.is_sign_negative()) {
        format!("{value:?} (0x{bits:08x})")
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
struct BufferInfo {
    size: usize,
    buffer_type: u64,
}

fn parse_fields(
    kind: OpKind,
    reader: &mut Reader<'_>,
    fields: &mut Vec<Field>,
    buffers: &mut HashMap<u64, BufferInfo>,
) -> Result<(), ParseError> {
    match kind {
        OpKind::MakeRenderBuffer => {
            let id = push_uint(reader, fields, "id")?;
            let size = push_uint(reader, fields, "size")?;
            let buffer_type = push_uint(reader, fields, "type")?;
            push_uint(reader, fields, "flags")?;
            let size = usize::try_from(size).map_err(|_| {
                ParseError::new(reader.position, "render buffer size exceeds usize")
            })?;
            if buffer_type > 1 {
                return Err(ParseError::new(
                    reader.position,
                    format!("invalid render buffer type {buffer_type}"),
                ));
            }
            if buffers
                .insert(id, BufferInfo { size, buffer_type })
                .is_some()
            {
                return Err(ParseError::new(
                    reader.position,
                    format!("duplicate render buffer id {id}"),
                ));
            }
        }
        OpKind::MakeLinearGradient => {
            parse_gradient(reader, fields)?;
            push_vec2(reader, fields, "start")?;
            push_vec2(reader, fields, "end")?;
        }
        OpKind::MakeRadialGradient => {
            parse_gradient(reader, fields)?;
            push_vec2(reader, fields, "center")?;
            push_float(reader, fields, "radius")?;
        }
        OpKind::MakeRenderPath | OpKind::MakeRenderPaint => {
            push_uint(reader, fields, "id")?;
        }
        OpKind::DecodeImage => {
            push_uint(reader, fields, "id")?;
            let size = push_uint(reader, fields, "size")?;
            let size = usize::try_from(size)
                .map_err(|_| ParseError::new(reader.position, "image size exceeds usize"))?;
            let bytes = reader.read_bytes(size, "image bytes")?.to_vec();
            fields.push(Field {
                name: "bytes",
                value: Value::Bytes(bytes),
            });
        }
        OpKind::Save | OpKind::Restore | OpKind::Frame => {}
        OpKind::Transform => {
            for name in ["xx", "yx", "xy", "yy", "tx", "ty"] {
                push_float(reader, fields, name)?;
            }
        }
        OpKind::DrawPath => {
            push_uint(reader, fields, "path_id")?;
            push_uint(reader, fields, "paint_id")?;
        }
        OpKind::ClipPath => {
            push_uint(reader, fields, "path_id")?;
        }
        OpKind::DrawImage => {
            push_uint(reader, fields, "image_id")?;
            push_uint(reader, fields, "blend_mode")?;
            push_float(reader, fields, "opacity")?;
        }
        OpKind::DrawImageMesh => {
            push_uint(reader, fields, "image_id")?;
            push_uint(reader, fields, "blend_mode")?;
            push_float(reader, fields, "opacity")?;
            push_uint(reader, fields, "positions_id")?;
            push_uint(reader, fields, "uvs_id")?;
            push_uint(reader, fields, "indices_id")?;
        }
        OpKind::SetVertexBufferData | OpKind::SetIndexBufferData => {
            let id = push_uint(reader, fields, "id")?;
            let info = buffers.get(&id).copied().ok_or_else(|| {
                ParseError::new(
                    reader.position,
                    format!("{kind} references unknown render buffer id {id}"),
                )
            })?;
            let (expected_type, stride) = match kind {
                OpKind::SetVertexBufferData => (1, 4),
                OpKind::SetIndexBufferData => (0, 2),
                _ => unreachable!(),
            };
            if info.buffer_type != expected_type {
                return Err(ParseError::new(
                    reader.position,
                    format!("{kind} uses buffer {id} with type {}", info.buffer_type),
                ));
            }
            if info.size % stride != 0 {
                return Err(ParseError::new(
                    reader.position,
                    format!(
                        "buffer {id} size {} is not divisible by {stride}",
                        info.size
                    ),
                ));
            }
            let count = info.size / stride;
            for _ in 0..count {
                match kind {
                    OpKind::SetVertexBufferData => {
                        push_float(reader, fields, "value")?;
                    }
                    OpKind::SetIndexBufferData => {
                        push_uint(reader, fields, "value")?;
                    }
                    _ => unreachable!(),
                }
            }
        }
        OpKind::AddRawPath => {
            push_uint(reader, fields, "path_id")?;
            let verb_count = push_uint(reader, fields, "verb_count")?;
            let verb_count = usize::try_from(verb_count)
                .map_err(|_| ParseError::new(reader.position, "verb count exceeds usize"))?;
            for _ in 0..verb_count {
                push_uint(reader, fields, "verb")?;
            }
            let point_count = push_uint(reader, fields, "point_count")?;
            let point_count = usize::try_from(point_count)
                .map_err(|_| ParseError::new(reader.position, "point count exceeds usize"))?;
            for _ in 0..point_count {
                push_vec2(reader, fields, "point")?;
            }
        }
        OpKind::Rewind => {
            push_uint(reader, fields, "path_id")?;
        }
        OpKind::FillRule => {
            push_uint(reader, fields, "path_id")?;
            push_uint(reader, fields, "value")?;
        }
        OpKind::Style
        | OpKind::Color
        | OpKind::Join
        | OpKind::Cap
        | OpKind::BlendMode
        | OpKind::Shader => {
            push_uint(reader, fields, "paint_id")?;
            push_uint(reader, fields, "value")?;
        }
        OpKind::Thickness | OpKind::Feather => {
            push_uint(reader, fields, "paint_id")?;
            push_float(reader, fields, "value")?;
        }
        OpKind::FrameSize => {
            push_uint(reader, fields, "width")?;
            push_uint(reader, fields, "height")?;
        }
        OpKind::ModulateOpacity => {
            push_float(reader, fields, "opacity")?;
        }
    }
    Ok(())
}

fn parse_gradient(reader: &mut Reader<'_>, fields: &mut Vec<Field>) -> Result<(), ParseError> {
    push_uint(reader, fields, "id")?;
    let count = push_uint(reader, fields, "count")?;
    let count = usize::try_from(count)
        .map_err(|_| ParseError::new(reader.position, "gradient stop count exceeds usize"))?;
    for _ in 0..count {
        push_uint(reader, fields, "color")?;
        push_float(reader, fields, "stop")?;
    }
    Ok(())
}

fn push_uint(
    reader: &mut Reader<'_>,
    fields: &mut Vec<Field>,
    name: &'static str,
) -> Result<u64, ParseError> {
    let value = reader.read_varuint(name)?;
    fields.push(Field {
        name,
        value: Value::Uint(value),
    });
    Ok(value)
}

fn push_float(
    reader: &mut Reader<'_>,
    fields: &mut Vec<Field>,
    name: &'static str,
) -> Result<u32, ParseError> {
    let value = reader.read_float_bits(name)?;
    fields.push(Field {
        name,
        value: Value::Float(value),
    });
    Ok(value)
}

fn push_vec2(
    reader: &mut Reader<'_>,
    fields: &mut Vec<Field>,
    name: &'static str,
) -> Result<(), ParseError> {
    let x = reader.read_float_bits(name)?;
    let y = reader.read_float_bits(name)?;
    fields.push(Field {
        name,
        value: Value::Vec2(x, y),
    });
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn is_empty(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn read_bytes(&mut self, count: usize, context: &str) -> Result<&'a [u8], ParseError> {
        let end = self.position.checked_add(count).ok_or_else(|| {
            ParseError::new(self.position, format!("{context} length overflows usize"))
        })?;
        let bytes = self.bytes.get(self.position..end).ok_or_else(|| {
            ParseError::new(
                self.position,
                format!("truncated {context}: need {count} bytes"),
            )
        })?;
        self.position = end;
        Ok(bytes)
    }

    fn read_float_bits(&mut self, context: &str) -> Result<u32, ParseError> {
        let offset = self.position;
        let bytes = self.read_bytes(4, context)?;
        let bytes: [u8; 4] = bytes
            .try_into()
            .map_err(|_| ParseError::new(offset, format!("truncated {context}")))?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_varuint(&mut self, context: &str) -> Result<u64, ParseError> {
        let offset = self.position;
        let mut value = 0u64;
        for index in 0..10 {
            let byte = *self.bytes.get(self.position).ok_or_else(|| {
                ParseError::new(offset, format!("truncated varuint for {context}"))
            })?;
            self.position += 1;
            if index == 9 && byte > 1 {
                return Err(ParseError::new(
                    offset,
                    format!("varuint overflow for {context}"),
                ));
            }
            value |= u64::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                if index > 0 && byte == 0 {
                    return Err(ParseError::new(
                        offset,
                        format!("non-canonical varuint for {context}"),
                    ));
                }
                return Ok(value);
            }
        }
        Err(ParseError::new(
            offset,
            format!("varuint overflow for {context}"),
        ))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub corpus: CorpusConfig,
    #[serde(rename = "case")]
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusConfig {
    pub version: u32,
    pub upstream_ref: String,
    pub expected_entries: usize,
    pub expected_runtime: usize,
    pub expected_scripted: usize,
    pub max_provenance_unknown: usize,
    pub min_cpp_rust_exact: usize,
    #[serde(default)]
    pub cpp_rust_exact_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Lane {
    Runtime,
    Scripted,
    Unknown,
}

impl Display for Lane {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Runtime => "runtime",
            Self::Scripted => "scripted",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Exact,
    Pending,
    PendingScripted,
    Diverges,
    UnsupportedFeature,
    ProvenanceUnknown,
}

impl Display for Status {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Exact => "exact",
            Self::Pending => "pending",
            Self::PendingScripted => "pending-scripted",
            Self::Diverges => "diverges",
            Self::UnsupportedFeature => "unsupported-feature",
            Self::ProvenanceUnknown => "provenance-unknown",
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub expected: String,
    pub source: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub artboard: String,
    pub animation: String,
    pub state_machine: String,
    pub lane: Lane,
    pub deterministic: String,
    pub random: String,
    pub view_model: String,
    #[serde(default)]
    pub sample_times: Vec<f32>,
    pub actions: String,
    pub verification: String,
    pub status: Status,
    pub producer_class: String,
    pub provenance_file: String,
    pub provenance_test: String,
    pub producer_line: usize,
    pub note: String,
}

#[derive(Debug, Default)]
pub struct CorpusSummary {
    pub entries: usize,
    pub provenanced: usize,
    pub runtime: usize,
    pub scripted: usize,
    pub cpp_baseline_exact: usize,
    pub cpp_rust_exact: usize,
    pub statuses: BTreeMap<Status, usize>,
    pub operations: usize,
    pub bytes: u64,
}

impl CorpusSummary {
    pub fn status(&self, status: Status) -> usize {
        self.statuses.get(&status).copied().unwrap_or(0)
    }
}

pub fn read_manifest(path: &Path) -> anyhow::Result<Manifest> {
    let contents = fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
    toml::from_str(&contents)
        .map_err(|error| anyhow::anyhow!("failed to parse {}: {error}", path.display()))
}

pub fn validate_manifest(manifest: &Manifest, runtime_dir: &Path) -> anyhow::Result<CorpusSummary> {
    let config = &manifest.corpus;
    anyhow::ensure!(config.version == 1, "manifest version must be 1");
    anyhow::ensure!(
        config.upstream_ref == UPSTREAM_REF,
        "manifest upstream_ref must remain {UPSTREAM_REF}"
    );
    anyhow::ensure!(
        config.expected_entries == EXPECTED_ENTRIES,
        "expected_entries ratchet must remain {EXPECTED_ENTRIES}"
    );
    anyhow::ensure!(
        config.expected_runtime == EXPECTED_RUNTIME,
        "expected_runtime ratchet must remain {EXPECTED_RUNTIME}"
    );
    anyhow::ensure!(
        config.expected_scripted == EXPECTED_SCRIPTED,
        "expected_scripted ratchet must remain {EXPECTED_SCRIPTED}"
    );
    anyhow::ensure!(
        config.max_provenance_unknown <= MAX_PROVENANCE_UNKNOWN,
        "max_provenance_unknown may not exceed {MAX_PROVENANCE_UNKNOWN}"
    );
    anyhow::ensure!(
        manifest.cases.len() == config.expected_entries,
        "manifest has {} entries; expected {}",
        manifest.cases.len(),
        config.expected_entries
    );

    let mut ids = BTreeSet::new();
    let mut expected_paths = BTreeSet::new();
    let mut exact_ids = BTreeSet::new();
    let mut summary = CorpusSummary {
        entries: manifest.cases.len(),
        ..CorpusSummary::default()
    };

    for case in &manifest.cases {
        anyhow::ensure!(!case.id.is_empty(), "manifest contains an empty id");
        anyhow::ensure!(
            ids.insert(case.id.as_str()),
            "duplicate case id {}",
            case.id
        );
        anyhow::ensure!(
            expected_paths.insert(case.expected.as_str()),
            "duplicate expected path {}",
            case.expected
        );
        let canonical_expected = format!("tests/unit_tests/silvers/{}.sriv", case.id);
        anyhow::ensure!(
            case.expected == canonical_expected,
            "{} expected path must be {}",
            case.id,
            canonical_expected
        );
        anyhow::ensure!(
            case.verification == "sriv-v1-epsilon",
            "{} has unsupported verification {}",
            case.id,
            case.verification
        );
        anyhow::ensure!(
            !case.note.trim().is_empty(),
            "{} must include a note",
            case.id
        );
        anyhow::ensure!(
            !case.deterministic.is_empty()
                && !case.random.is_empty()
                && !case.view_model.is_empty()
                && !case.actions.is_empty(),
            "{} is missing producer settings",
            case.id
        );

        match case.lane {
            Lane::Runtime => summary.runtime += 1,
            Lane::Scripted => summary.scripted += 1,
            Lane::Unknown => {}
        }
        *summary.statuses.entry(case.status).or_default() += 1;

        if case.status == Status::ProvenanceUnknown {
            anyhow::ensure!(
                case.lane == Lane::Unknown,
                "{} provenance-unknown entry must use unknown lane",
                case.id
            );
        } else {
            summary.provenanced += 1;
            anyhow::ensure!(
                case.lane != Lane::Unknown,
                "{} provenanced entry may not use unknown lane",
                case.id
            );
            let provenance = runtime_dir.join(&case.provenance_file);
            anyhow::ensure!(
                provenance.is_file(),
                "{} provenance file is missing: {}",
                case.id,
                provenance.display()
            );
        }
        if case.status == Status::Exact {
            exact_ids.insert(case.id.as_str());
        }

        let expected = runtime_dir.join(&case.expected);
        let bytes = fs::read(&expected).map_err(|error| {
            anyhow::anyhow!(
                "{}: failed to read {}: {error}",
                case.id,
                expected.display()
            )
        })?;
        let parsed = parse_sriv(&bytes).map_err(|error| {
            anyhow::anyhow!("{}: invalid {}: {error}", case.id, expected.display())
        })?;
        summary.cpp_baseline_exact += 1;
        summary.operations += parsed.operations.len();
        summary.bytes += bytes.len() as u64;

        for source in std::iter::once(&case.source).chain(&case.dependencies) {
            if source == "inline-script" || source == "provenance-unknown" {
                continue;
            }
            let path = runtime_dir.join("tests/unit_tests/assets").join(source);
            anyhow::ensure!(
                path.is_file(),
                "{} source is missing: {}",
                case.id,
                path.display()
            );
        }
    }

    anyhow::ensure!(
        summary.runtime == config.expected_runtime,
        "runtime lane has {} entries; expected {}",
        summary.runtime,
        config.expected_runtime
    );
    anyhow::ensure!(
        summary.scripted == config.expected_scripted,
        "scripted lane has {} entries; expected {}",
        summary.scripted,
        config.expected_scripted
    );
    anyhow::ensure!(
        summary.status(Status::ProvenanceUnknown) <= config.max_provenance_unknown,
        "provenance-unknown={} exceeds ratchet {}",
        summary.status(Status::ProvenanceUnknown),
        config.max_provenance_unknown
    );

    let silver_dir = runtime_dir.join("tests/unit_tests/silvers");
    let actual_paths = fs::read_dir(&silver_dir)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", silver_dir.display()))?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|extension| extension.to_str()) == Some("sriv")).then(|| {
                format!(
                    "tests/unit_tests/silvers/{}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let manifest_paths = expected_paths
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    let missing = actual_paths.difference(&manifest_paths).collect::<Vec<_>>();
    let extra = manifest_paths.difference(&actual_paths).collect::<Vec<_>>();
    anyhow::ensure!(
        missing.is_empty() && extra.is_empty(),
        "silver coverage mismatch; unrepresented={missing:?}, missing-files={extra:?}"
    );

    let ratchet_ids = config
        .cpp_rust_exact_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let downgraded = ratchet_ids.difference(&exact_ids).collect::<Vec<_>>();
    anyhow::ensure!(
        downgraded.is_empty(),
        "exact entries were downgraded without a ledger change: {downgraded:?}"
    );
    summary.cpp_rust_exact = exact_ids.len();
    anyhow::ensure!(
        summary.cpp_rust_exact >= config.min_cpp_rust_exact,
        "cpp-rust-exact={} is below ratchet {}",
        summary.cpp_rust_exact,
        config.min_cpp_rust_exact
    );
    Ok(summary)
}

pub fn compare_files(expected: &Path, actual: &Path) -> anyhow::Result<()> {
    let expected_bytes = fs::read(expected)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", expected.display()))?;
    let actual_bytes = fs::read(actual)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", actual.display()))?;
    let expected_sriv = parse_sriv(&expected_bytes)
        .map_err(|error| anyhow::anyhow!("invalid expected {}: {error}", expected.display()))?;
    let actual_sriv = parse_sriv(&actual_bytes)
        .map_err(|error| anyhow::anyhow!("invalid actual {}: {error}", actual.display()))?;
    compare_sriv(&expected_sriv, &actual_sriv).map_err(|difference| anyhow::anyhow!("{difference}"))
}

pub fn resolve_expected(runtime_dir: &Path, case: &Case) -> PathBuf {
    runtime_dir.join(&case.expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEST_CORPUS: AtomicUsize = AtomicUsize::new(0);

    fn header() -> Vec<u8> {
        b"SRIV\x01".to_vec()
    }

    fn varuint(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                return bytes;
            }
        }
    }

    fn float(value: f32) -> [u8; 4] {
        value.to_bits().to_le_bytes()
    }

    #[test]
    fn parses_every_sriv_v1_operation() {
        let mut bytes = header();
        bytes.extend([0, 0, 4, 1, 0]); // buffer 0, 4 bytes, vertex, flags 0
        bytes.extend([14, 0]);
        bytes.extend(float(3.0));
        bytes.extend([1, 0, 1, 0xff, 0x01]); // linear gradient, id 0, 1 stop, color 255
        bytes.extend(float(0.5));
        for value in [0.0, 1.0, 2.0, 3.0] {
            bytes.extend(float(value));
        }
        bytes.extend([2, 1, 0]); // radial gradient, id 1, no stops
        for value in [4.0, 5.0, 6.0] {
            bytes.extend(float(value));
        }
        bytes.extend([3, 0, 5, 0, 6, 0, 2, 0xaa, 0xbb]);
        bytes.extend([7, 8, 9]);
        for value in [1.0, 0.0, 0.0, 1.0, 2.0, 3.0] {
            bytes.extend(float(value));
        }
        bytes.extend([10, 0, 0, 11, 0, 12, 0, 3]);
        bytes.extend(float(0.75));
        bytes.extend([13, 0, 3]);
        bytes.extend(float(0.5));
        bytes.extend([0, 0, 0]);
        bytes.extend([16, 0, 1, 2, 1]);
        bytes.extend(float(7.0));
        bytes.extend(float(8.0));
        bytes.extend([17, 0, 18, 0, 1]);
        for op in [20, 21, 23, 24, 26, 27] {
            bytes.extend([op, 0, 1]);
        }
        for op in [22, 25] {
            bytes.extend([op, 0]);
            bytes.extend(float(2.0));
        }
        bytes.extend([28, 29]);
        bytes.extend(varuint(640));
        bytes.extend(varuint(480));
        bytes.push(30);
        bytes.extend(float(0.25));

        let parsed = parse_sriv(&bytes).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.operations.len(), 28);
        assert_eq!(parsed.operations.last().unwrap().frame, 1);
    }

    #[test]
    fn rejects_bad_header_version_unknown_op_and_truncation() {
        assert!(
            parse_sriv(b"NOPE\x01")
                .unwrap_err()
                .message
                .contains("header")
        );
        assert!(
            parse_sriv(b"SRIV\x02")
                .unwrap_err()
                .message
                .contains("version")
        );
        assert!(
            parse_sriv(b"SRIV\x01\x04")
                .unwrap_err()
                .message
                .contains("unknown")
        );
        assert!(
            parse_sriv(b"SRIV\x01\x09\0")
                .unwrap_err()
                .message
                .contains("truncated")
        );
    }

    #[test]
    fn rejects_noncanonical_and_overflowing_varuints() {
        assert!(
            parse_sriv(b"SRIV\x81\0")
                .unwrap_err()
                .message
                .contains("non-canonical")
        );
        let mut bytes = b"SRIV".to_vec();
        bytes.extend([0xff; 10]);
        assert!(parse_sriv(&bytes).unwrap_err().message.contains("overflow"));
    }

    #[test]
    fn comparator_uses_epsilon_and_reports_first_field() {
        fn thickness(value: f32) -> Sriv {
            let mut bytes = header();
            bytes.extend([22, 0]);
            bytes.extend(float(value));
            parse_sriv(&bytes).unwrap()
        }
        assert!(compare_sriv(&thickness(1.0), &thickness(1.0009)).is_ok());
        let difference = compare_sriv(&thickness(1.0), &thickness(1.002)).unwrap_err();
        assert_eq!(difference.operation, 0);
        assert_eq!(difference.field, Some("value"));
        assert!(difference.to_string().contains("thickness"));
    }

    #[test]
    fn comparator_compares_special_float_bits() {
        assert!(floats_match(f32::NAN.to_bits(), f32::NAN.to_bits()));
        assert!(!floats_match(
            f32::NAN.to_bits(),
            f32::from_bits(f32::NAN.to_bits() + 1).to_bits()
        ));
        assert!(!floats_match((-0.0f32).to_bits(), 0.0f32.to_bits()));
        assert!(floats_match(
            f32::INFINITY.to_bits(),
            f32::INFINITY.to_bits()
        ));
        assert!(!floats_match(
            f32::INFINITY.to_bits(),
            f32::NEG_INFINITY.to_bits()
        ));
    }

    struct TestCorpus {
        root: PathBuf,
    }

    impl TestCorpus {
        fn new() -> (Self, Manifest) {
            let root = std::env::temp_dir().join(format!(
                "nuxie-silver-corpus-test-{}-{}",
                std::process::id(),
                NEXT_TEST_CORPUS.fetch_add(1, Ordering::Relaxed),
            ));
            if root.exists() {
                fs::remove_dir_all(&root).unwrap();
            }
            let silvers = root.join("tests/unit_tests/silvers");
            let runtime = root.join("tests/unit_tests/runtime");
            fs::create_dir_all(&silvers).unwrap();
            fs::create_dir_all(&runtime).unwrap();
            fs::write(runtime.join("producer.cpp"), "// fixture").unwrap();

            let mut cases = Vec::new();
            for index in 0..EXPECTED_ENTRIES {
                let id = format!("case-{index:03}");
                fs::write(silvers.join(format!("{id}.sriv")), b"SRIV\x01").unwrap();
                let (lane, status) = if index < EXPECTED_RUNTIME {
                    (Lane::Runtime, Status::Pending)
                } else if index < EXPECTED_RUNTIME + EXPECTED_SCRIPTED {
                    (Lane::Scripted, Status::PendingScripted)
                } else {
                    (Lane::Unknown, Status::ProvenanceUnknown)
                };
                cases.push(Case {
                    id: id.clone(),
                    expected: format!("tests/unit_tests/silvers/{id}.sriv"),
                    source: if lane == Lane::Unknown {
                        "provenance-unknown".to_owned()
                    } else {
                        "inline-script".to_owned()
                    },
                    dependencies: Vec::new(),
                    artboard: "default".to_owned(),
                    animation: "none".to_owned(),
                    state_machine: "default".to_owned(),
                    lane,
                    deterministic: "cpp-test-defined".to_owned(),
                    random: "cpp-test-defined".to_owned(),
                    view_model: "none".to_owned(),
                    sample_times: Vec::new(),
                    actions: "cpp-test-body".to_owned(),
                    verification: "sriv-v1-epsilon".to_owned(),
                    status,
                    producer_class: status.to_string(),
                    provenance_file: if lane == Lane::Unknown {
                        String::new()
                    } else {
                        "tests/unit_tests/runtime/producer.cpp".to_owned()
                    },
                    provenance_test: "fixture".to_owned(),
                    producer_line: 1,
                    note: "fixture".to_owned(),
                });
            }
            let manifest = Manifest {
                corpus: CorpusConfig {
                    version: 1,
                    upstream_ref: UPSTREAM_REF.to_owned(),
                    expected_entries: EXPECTED_ENTRIES,
                    expected_runtime: EXPECTED_RUNTIME,
                    expected_scripted: EXPECTED_SCRIPTED,
                    max_provenance_unknown: MAX_PROVENANCE_UNKNOWN,
                    min_cpp_rust_exact: 0,
                    cpp_rust_exact_ids: Vec::new(),
                },
                cases,
            };
            (Self { root }, manifest)
        }
    }

    impl Drop for TestCorpus {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn manifest_validation_enforces_full_corpus_and_lane_ratchets() {
        let (fixture, manifest) = TestCorpus::new();
        let summary = validate_manifest(&manifest, &fixture.root).unwrap();
        assert_eq!(summary.entries, EXPECTED_ENTRIES);
        assert_eq!(summary.runtime, EXPECTED_RUNTIME);
        assert_eq!(summary.scripted, EXPECTED_SCRIPTED);
        assert_eq!(summary.cpp_baseline_exact, EXPECTED_ENTRIES);
        assert_eq!(summary.status(Status::ProvenanceUnknown), 2);
    }

    #[test]
    fn manifest_validation_rejects_duplicate_ids() {
        let (fixture, mut manifest) = TestCorpus::new();
        manifest.cases[1].id = manifest.cases[0].id.clone();
        let error = validate_manifest(&manifest, &fixture.root).unwrap_err();
        assert!(error.to_string().contains("duplicate case id"));
    }
}
