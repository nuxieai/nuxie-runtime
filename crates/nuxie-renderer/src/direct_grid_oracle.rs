//! Exact parser for the C++ interior-triangulation preparation oracle.

const GRID_MAGIC: [u8; 8] = *b"RIVEDGI\0";
const FLOWER_MAGIC: [u8; 8] = *b"RIVEDFI\0";
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 64;
const DRAW_STRIDE: usize = 20;
const CONTOUR_STRIDE: usize = 16;
const TRIANGLE_STRIDE: usize = 12;
const TEXEL_STRIDE: usize = 16;
const DRAW_TYPE_OUTER_CUBICS: u32 = 2;
const DRAW_TYPE_INTERIOR_TRIANGULATION: u32 = 3;
const DRAW_TYPE_INITIALIZE: u32 = 15;
const DRAW_TYPE_RESOLVE: u32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DrawRecord {
    pub(crate) draw_type: u32,
    pub(crate) shader_features: u32,
    pub(crate) shader_misc_flags: u32,
    pub(crate) base_element: u32,
    pub(crate) element_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContourRecord {
    pub(crate) midpoint_x_bits: u32,
    pub(crate) midpoint_y_bits: u32,
    pub(crate) path_id: u32,
    pub(crate) vertex_index0: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TriangleRecord {
    pub(crate) x_bits: u32,
    pub(crate) y_bits: u32,
    pub(crate) weight_path_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DirectGridInputs {
    pub(crate) interlock_mode: u32,
    pub(crate) draws: Vec<DrawRecord>,
    pub(crate) contours: Vec<ContourRecord>,
    pub(crate) triangles: Vec<TriangleRecord>,
    pub(crate) tess_width: u32,
    pub(crate) tess_height: u32,
    pub(crate) texels: Vec<[u32; 4]>,
}

impl DirectGridInputs {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, String> {
        Self::parse_contract(bytes, GRID_MAGIC, 100)
    }

    pub(crate) fn parse_flower(bytes: &[u8]) -> Result<Self, String> {
        Self::parse_contract(bytes, FLOWER_MAGIC, 2)
    }

    fn parse_contract(
        bytes: &[u8],
        magic: [u8; 8],
        expected_contour_count: u32,
    ) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err(format!("truncated RIVEDGI header: {}", bytes.len()));
        }
        if bytes[..8] != magic {
            return Err("invalid direct-input magic".into());
        }
        let version = read_u32(bytes, 8);
        let header_size = read_u32(bytes, 12);
        if version != VERSION || header_size != HEADER_SIZE as u32 {
            return Err(format!(
                "unsupported RIVEDGI header: version={version} bytes={header_size}"
            ));
        }
        let flags = read_u32(bytes, 16);
        let interlock_mode = read_u32(bytes, 20);
        if flags != 1 || interlock_mode != 1 {
            return Err("RIVEDGI is not a clockwise-atomic capture".into());
        }
        let draw_count = read_u32(bytes, 24);
        let tess_width = read_u32(bytes, 28);
        let tess_height = read_u32(bytes, 32);
        let contour_count = read_u32(bytes, 36);
        let triangle_count = read_u32(bytes, 40);
        if contour_count != expected_contour_count {
            return Err(format!(
                "direct-input contour count is {contour_count}, expected {expected_contour_count}"
            ));
        }
        if draw_count < 3 || triangle_count == 0 || triangle_count % 3 != 0 {
            return Err("RIVEDGI draw or triangle count is invalid".into());
        }
        if tess_width == 0 || tess_height == 0 {
            return Err("RIVEDGI tessellation dimensions are empty".into());
        }
        let strides = [
            read_u32(bytes, 44),
            read_u32(bytes, 48),
            read_u32(bytes, 52),
            read_u32(bytes, 56),
        ];
        if strides
            != [
                DRAW_STRIDE as u32,
                CONTOUR_STRIDE as u32,
                TRIANGLE_STRIDE as u32,
                TEXEL_STRIDE as u32,
            ]
        {
            return Err(format!("RIVEDGI stride mismatch: {strides:?}"));
        }
        if read_u32(bytes, 60) != 0 {
            return Err("RIVEDGI reserved field is nonzero".into());
        }

        let draw_count = usize::try_from(draw_count).map_err(|_| "draw count overflow")?;
        let contour_count = usize::try_from(contour_count).map_err(|_| "contour count overflow")?;
        let triangle_count =
            usize::try_from(triangle_count).map_err(|_| "triangle count overflow")?;
        let texel_count = usize::try_from(tess_width)
            .ok()
            .and_then(|width| {
                usize::try_from(tess_height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .ok_or("texel count overflow")?;
        let expected = HEADER_SIZE
            .checked_add(
                draw_count
                    .checked_mul(DRAW_STRIDE)
                    .ok_or("draw size overflow")?,
            )
            .and_then(|size| size.checked_add(contour_count * CONTOUR_STRIDE))
            .and_then(|size| size.checked_add(triangle_count * TRIANGLE_STRIDE))
            .and_then(|size| size.checked_add(texel_count * TEXEL_STRIDE))
            .ok_or("RIVEDGI layout overflow")?;
        if bytes.len() != expected {
            return Err(format!(
                "RIVEDGI length mismatch: actual={} expected={expected}",
                bytes.len()
            ));
        }

        let mut offset = HEADER_SIZE;
        let mut draws = Vec::with_capacity(draw_count);
        for _ in 0..draw_count {
            draws.push(DrawRecord {
                draw_type: read_u32(bytes, offset),
                shader_features: read_u32(bytes, offset + 4),
                shader_misc_flags: read_u32(bytes, offset + 8),
                base_element: read_u32(bytes, offset + 12),
                element_count: read_u32(bytes, offset + 16),
            });
            offset += DRAW_STRIDE;
        }
        let [initialize, outer, interior, resolve] = draws.as_slice() else {
            return Err(format!(
                "RIVEDGI schedule has {} draws, expected 4",
                draws.len()
            ));
        };
        if initialize.draw_type != DRAW_TYPE_INITIALIZE
            || initialize.base_element != 0
            || initialize.element_count != 1
            || outer.draw_type != DRAW_TYPE_OUTER_CUBICS
            || outer.base_element == 0
            || outer.element_count == 0
            || interior.draw_type != DRAW_TYPE_INTERIOR_TRIANGULATION
            || interior.base_element != 0
            || interior.element_count as usize != triangle_count
            || resolve.draw_type != DRAW_TYPE_RESOLVE
            || resolve.base_element != 0
            || resolve.element_count != 1
        {
            return Err("RIVEDGI draw schedule is not initialize/outer/interior/resolve".into());
        }
        let mut contours = Vec::with_capacity(contour_count);
        for _ in 0..contour_count {
            contours.push(ContourRecord {
                midpoint_x_bits: read_u32(bytes, offset),
                midpoint_y_bits: read_u32(bytes, offset + 4),
                path_id: read_u32(bytes, offset + 8),
                vertex_index0: read_u32(bytes, offset + 12),
            });
            offset += CONTOUR_STRIDE;
        }
        let mut triangles = Vec::with_capacity(triangle_count);
        for _ in 0..triangle_count {
            triangles.push(TriangleRecord {
                x_bits: read_u32(bytes, offset),
                y_bits: read_u32(bytes, offset + 4),
                weight_path_id: read_u32(bytes, offset + 8),
            });
            offset += TRIANGLE_STRIDE;
        }
        let mut texels = Vec::with_capacity(texel_count);
        for _ in 0..texel_count {
            texels.push([
                read_u32(bytes, offset),
                read_u32(bytes, offset + 4),
                read_u32(bytes, offset + 8),
                read_u32(bytes, offset + 12),
            ]);
            offset += TEXEL_STRIDE;
        }
        Ok(Self {
            interlock_mode,
            draws,
            contours,
            triangles,
            tess_width,
            tess_height,
            texels,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&GRID_MAGIC);
        for value in [
            VERSION,
            HEADER_SIZE as u32,
            1,
            self.interlock_mode,
            self.draws.len() as u32,
            self.tess_width,
            self.tess_height,
            self.contours.len() as u32,
            self.triangles.len() as u32,
            DRAW_STRIDE as u32,
            CONTOUR_STRIDE as u32,
            TRIANGLE_STRIDE as u32,
            TEXEL_STRIDE as u32,
            0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for draw in &self.draws {
            for value in [
                draw.draw_type,
                draw.shader_features,
                draw.shader_misc_flags,
                draw.base_element,
                draw.element_count,
            ] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        for contour in &self.contours {
            for value in [
                contour.midpoint_x_bits,
                contour.midpoint_y_bits,
                contour.path_id,
                contour.vertex_index0,
            ] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        for triangle in &self.triangles {
            for value in [triangle.x_bits, triangle.y_bits, triangle.weight_path_id] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        for texel in &self.texels {
            for value in texel {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> DirectGridInputs {
        DirectGridInputs {
            interlock_mode: 1,
            draws: vec![
                DrawRecord {
                    draw_type: DRAW_TYPE_INITIALIZE,
                    shader_features: 0,
                    shader_misc_flags: 1,
                    base_element: 0,
                    element_count: 1,
                },
                DrawRecord {
                    draw_type: DRAW_TYPE_OUTER_CUBICS,
                    shader_features: 128,
                    shader_misc_flags: 1,
                    base_element: 1,
                    element_count: 10,
                },
                DrawRecord {
                    draw_type: DRAW_TYPE_INTERIOR_TRIANGULATION,
                    shader_features: 128,
                    shader_misc_flags: 1,
                    base_element: 0,
                    element_count: 6,
                },
                DrawRecord {
                    draw_type: DRAW_TYPE_RESOLVE,
                    shader_features: 0,
                    shader_misc_flags: 1,
                    base_element: 0,
                    element_count: 1,
                },
            ],
            contours: vec![
                ContourRecord {
                    midpoint_x_bits: 0,
                    midpoint_y_bits: 0,
                    path_id: 1,
                    vertex_index0: 8,
                };
                100
            ],
            triangles: vec![
                TriangleRecord {
                    x_bits: 0,
                    y_bits: 0,
                    weight_path_id: 0x0001_0001,
                };
                6
            ],
            tess_width: 2,
            tess_height: 1,
            texels: vec![[0, 1, 2, 3], [4, 5, 6, 7]],
        }
    }

    #[test]
    fn round_trips_exact_little_endian_layout() {
        let fixture = fixture();
        let bytes = fixture.serialize();
        assert_eq!(DirectGridInputs::parse(&bytes).unwrap(), fixture);
    }

    #[test]
    fn rejects_malformed_counts_strides_and_lengths() {
        let bytes = fixture().serialize();
        let mutate = |offset: usize, value: u32| {
            let mut bytes = bytes.clone();
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            bytes
        };
        assert!(DirectGridInputs::parse(&mutate(36, 99)).is_err());
        assert!(DirectGridInputs::parse(&mutate(40, 5)).is_err());
        assert!(DirectGridInputs::parse(&mutate(44, 24)).is_err());
        assert!(DirectGridInputs::parse(&mutate(20, 3)).is_err());
        let schedule_offset = HEADER_SIZE + 2 * DRAW_STRIDE;
        assert!(DirectGridInputs::parse(&mutate(schedule_offset, DRAW_TYPE_OUTER_CUBICS)).is_err());
        assert!(DirectGridInputs::parse(&mutate(schedule_offset + 16, 3)).is_err());
        assert!(DirectGridInputs::parse(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn parses_configured_cpp_artifact() {
        let Ok(path) = std::env::var("RIVE_CPP_DIRECT_GRID_INPUTS") else {
            return;
        };
        let bytes = std::fs::read(path).unwrap();
        let capture = DirectGridInputs::parse(&bytes).unwrap();
        assert_eq!(capture.draws.len(), 4);
        assert_eq!(capture.contours.len(), 100);
        assert_eq!(capture.triangles.len(), 7_500);
        assert_eq!((capture.tess_width, capture.tess_height), (2048, 212));
    }
}
