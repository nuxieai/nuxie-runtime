//! Authoring-only writer using the runtime schema. No test-support builders or
//! runtime construction internals are required by the compiler library.
use crate::Diagnostic;
use nuxie_schema::{FieldKind, definition_by_name};
use std::collections::BTreeMap;

pub(crate) enum Value {
    Uint(u32),
    Float(f32),
    Color(u32),
    String(String),
    Bytes(Vec<u8>),
    Bool(bool),
}
pub(crate) struct Record {
    pub kind: &'static str,
    properties: BTreeMap<u16, Value>,
}

impl Record {
    pub fn new(kind: &'static str) -> Self {
        Self {
            kind,
            properties: BTreeMap::new(),
        }
    }
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), Diagnostic> {
        let mut definition = definition_by_name(self.kind);
        while let Some(d) = definition {
            if let Some(property) = d.properties.iter().find(|p| p.name == name) {
                let valid = matches!(
                    (&value, property.runtime_type),
                    (Value::Uint(_), FieldKind::Uint)
                        | (Value::Float(_), FieldKind::Double)
                        | (Value::Color(_), FieldKind::Color)
                        | (Value::String(_), FieldKind::String)
                        | (Value::Bytes(_), FieldKind::Bytes)
                        | (Value::Bool(_), FieldKind::Bool)
                );
                if !valid {
                    return Err(Diagnostic::new(
                        "schema-mismatch",
                        self.kind,
                        format!("Wrong value type for {name}"),
                    ));
                }
                self.properties.insert(property.key.int, value);
                return Ok(());
            }
            definition = d.runtime_parent.and_then(definition_by_name);
        }
        Err(Diagnostic::new(
            "schema-mismatch",
            self.kind,
            format!("Unknown property {name}"),
        ))
    }
}

pub(crate) fn encode(records: &[Record]) -> Result<Vec<u8>, Diagnostic> {
    // All emitted properties are known by the pinned consumer registry. The
    // header ToC still enumerates its representable field kinds for readers.
    let mut fields = BTreeMap::new();
    for record in records {
        for (&key, value) in &record.properties {
            let kind = match value {
                Value::Uint(_) => 0,
                Value::String(_) | Value::Bytes(_) => 1,
                Value::Float(_) => 2,
                Value::Color(_) => 3,
                Value::Bool(_) => continue,
            };
            fields.insert(key, kind);
        }
    }
    let mut bytes = b"RIVE\x07\x03\x00".to_vec();
    for key in fields.keys() {
        varuint(&mut bytes, u32::from(*key));
    }
    varuint(&mut bytes, 0);
    for group in fields.values().copied().collect::<Vec<u32>>().chunks(4) {
        let packed = group
            .iter()
            .enumerate()
            .fold(0u32, |v, (i, field)| v | (field << (i * 2)));
        bytes.extend(packed.to_le_bytes());
    }
    for record in records {
        let definition = definition_by_name(record.kind).ok_or_else(|| {
            Diagnostic::new("schema-mismatch", record.kind, "Unknown object type")
        })?;
        varuint(&mut bytes, u32::from(definition.type_key.int));
        for (&key, value) in &record.properties {
            varuint(&mut bytes, u32::from(key));
            match value {
                Value::Uint(v) => varuint(&mut bytes, *v),
                Value::Float(v) => bytes.extend(v.to_le_bytes()),
                Value::Color(v) => bytes.extend(v.to_le_bytes()),
                Value::Bool(v) => bytes.push(u8::from(*v)),
                Value::String(v) => {
                    varuint(&mut bytes, v.len() as u32);
                    bytes.extend(v.as_bytes());
                }
                Value::Bytes(v) => {
                    varuint(&mut bytes, v.len() as u32);
                    bytes.extend(v);
                }
            }
        }
        varuint(&mut bytes, 0);
    }
    Ok(bytes)
}

fn varuint(out: &mut Vec<u8>, mut v: u32) {
    loop {
        let byte = (v & 127) as u8;
        v >>= 7;
        out.push(byte | if v != 0 { 128 } else { 0 });
        if v == 0 {
            break;
        }
    }
}
