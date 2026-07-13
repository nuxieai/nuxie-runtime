//! Exact CPU tessellation-span artifact used by the C++ parity oracle.

use crate::gpu::TessVertexSpan;

const MAGIC: &[u8; 8] = b"RIVEATS\0";
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 28;
const RECORD_WORDS: usize = 16;
const RECORD_SIZE: usize = RECORD_WORDS * 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TessSpanArtifact {
    pub first_span: u32,
    pub records: Vec<[u32; RECORD_WORDS]>,
}

impl TessSpanArtifact {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err(format!(
                "truncated header: expected {HEADER_SIZE} bytes, got {}",
                bytes.len()
            ));
        }
        if &bytes[..8] != MAGIC {
            return Err("invalid magic; expected RIVEATS\\0".into());
        }
        let version = read_u32(bytes, 8);
        if version != VERSION {
            return Err(format!("unsupported version {version}; expected {VERSION}"));
        }
        let header_size = read_u32(bytes, 12) as usize;
        if header_size != HEADER_SIZE {
            return Err(format!(
                "header size must be {HEADER_SIZE}, got {header_size}"
            ));
        }
        let first_span = read_u32(bytes, 16);
        let record_count = read_u32(bytes, 20) as usize;
        let record_size = read_u32(bytes, 24) as usize;
        if record_size != RECORD_SIZE {
            return Err(format!(
                "record size must be {RECORD_SIZE}, got {record_size}"
            ));
        }
        let expected = HEADER_SIZE
            .checked_add(
                record_count
                    .checked_mul(RECORD_SIZE)
                    .ok_or("record byte length overflow")?,
            )
            .ok_or("artifact byte length overflow")?;
        if bytes.len() != expected {
            return Err(format!(
                "artifact length must be {expected}, got {}",
                bytes.len()
            ));
        }
        if record_count == 0 {
            return Err("artifact must contain at least one span".into());
        }
        let records = bytes[HEADER_SIZE..]
            .chunks_exact(RECORD_SIZE)
            .map(|record| std::array::from_fn(|word| read_u32(record, word * 4)))
            .collect();
        Ok(Self {
            first_span,
            records,
        })
    }

    pub(crate) fn from_spans(first_span: u32, spans: &[TessVertexSpan]) -> Self {
        let records = spans
            .iter()
            .map(|span| {
                let mut words = [0; RECORD_WORDS];
                for (index, point) in span.points.iter().enumerate() {
                    words[index * 2] = point[0].to_bits();
                    words[index * 2 + 1] = point[1].to_bits();
                }
                words[8] = span.join_tangent[0].to_bits();
                words[9] = span.join_tangent[1].to_bits();
                words[10] = span.y.to_bits();
                words[11] = span.reflection_y.to_bits();
                words[12] = span.x0_x1 as u32;
                words[13] = span.reflection_x0_x1 as u32;
                words[14] = span.segment_counts;
                words[15] = span.contour_id_with_flags;
                words
            })
            .collect();
        Self {
            first_span,
            records,
        }
    }
}

pub(crate) fn compare_exact(cpp: &TessSpanArtifact, rust: &TessSpanArtifact) -> Result<(), String> {
    if cpp.first_span != rust.first_span {
        return Err(format!(
            "first_span differs: C++={}, Rust={}",
            cpp.first_span, rust.first_span
        ));
    }
    if cpp.records.len() != rust.records.len() {
        return Err(format!(
            "span count differs: C++={}, Rust={}",
            cpp.records.len(),
            rust.records.len()
        ));
    }
    for (record, (cpp_record, rust_record)) in cpp.records.iter().zip(&rust.records).enumerate() {
        for (word, (&cpp_word, &rust_word)) in cpp_record.iter().zip(rust_record).enumerate() {
            if cpp_word != rust_word {
                return Err(format!(
                    "span {record} word {word} differs: C++={cpp_word:#010x}, Rust={rust_word:#010x}"
                ));
            }
        }
    }
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::TessVertexSpan;

    fn fixture() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&VERSION.to_le_bytes());
        bytes.extend_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&(RECORD_SIZE as u32).to_le_bytes());
        for word in 0..RECORD_WORDS as u32 {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn parses_fixed_header_and_raw_records() {
        let parsed = TessSpanArtifact::parse(&fixture()).unwrap();
        assert_eq!(parsed.first_span, 4);
        assert_eq!(
            parsed.records,
            vec![std::array::from_fn(|index| index as u32)]
        );
    }

    #[test]
    fn rejects_wrong_stride_and_trailing_data() {
        let mut wrong_stride = fixture();
        wrong_stride[24..28].copy_from_slice(&60u32.to_le_bytes());
        assert!(TessSpanArtifact::parse(&wrong_stride)
            .unwrap_err()
            .contains("record size"));

        let mut trailing = fixture();
        trailing.push(0);
        assert!(TessSpanArtifact::parse(&trailing)
            .unwrap_err()
            .contains("artifact length"));
    }

    #[test]
    fn rust_span_words_preserve_exact_layout_fields() {
        let span = TessVertexSpan::without_reflection(
            [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0], [7.0, 8.0]],
            [9.0, 10.0],
            11.0,
            12,
            13,
            14,
            15,
            16,
            17,
        );
        let artifact = TessSpanArtifact::from_spans(4, &[span]);
        assert_eq!(artifact.first_span, 4);
        assert_eq!(artifact.records[0][0], 1.0f32.to_bits());
        assert_eq!(artifact.records[0][8], 9.0f32.to_bits());
        assert_eq!(artifact.records[0][14], 16 << 20 | 15 << 10 | 14);
        assert_eq!(artifact.records[0][15], 17);
    }
}
