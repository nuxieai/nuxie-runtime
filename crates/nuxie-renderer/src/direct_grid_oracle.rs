//! Exact parser for the C++ interior-triangulation preparation oracle.

const GRID_MAGIC: [u8; 8] = *b"RIVEDGI\0";
const FLOWER_MAGIC: [u8; 8] = *b"RIVEDFI\0";
const BAD_SKIN_MAGIC: [u8; 8] = *b"RIVEDBI\0";
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

    pub(crate) fn parse_bad_skin(bytes: &[u8]) -> Result<Self, String> {
        Self::parse_contract(bytes, BAD_SKIN_MAGIC, 1)
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
        self.serialize_contract(GRID_MAGIC)
    }

    fn serialize_bad_skin(&self) -> Vec<u8> {
        self.serialize_contract(BAD_SKIN_MAGIC)
    }

    fn serialize_contract(&self, magic: [u8; 8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&magic);
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

    fn canonical_triangles(records: &[TriangleRecord]) -> Vec<[(u32, u32, u32); 3]> {
        assert_eq!(records.len() % 3, 0);
        let mut triangles = records
            .chunks_exact(3)
            .map(|triangle| {
                let mut vertices = [triangle[0], triangle[1], triangle[2]]
                    .map(|vertex| (vertex.x_bits, vertex.y_bits, vertex.weight_path_id));
                vertices.sort_unstable();
                vertices
            })
            .collect::<Vec<_>>();
        triangles.sort_unstable();
        triangles
    }

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
    fn bad_skin_round_trips_and_rejects_malformed_header_and_facts() {
        let mut fixture = fixture();
        fixture.contours.truncate(1);
        let bytes = fixture.serialize_bad_skin();
        assert_eq!(DirectGridInputs::parse_bad_skin(&bytes).unwrap(), fixture);
        assert!(DirectGridInputs::parse(&bytes).is_err());

        let mutate = |offset: usize, value: u32| {
            let mut bytes = bytes.clone();
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            bytes
        };
        assert!(DirectGridInputs::parse_bad_skin(&mutate(8, VERSION + 1)).is_err());
        assert!(DirectGridInputs::parse_bad_skin(&mutate(12, 60)).is_err());
        assert!(DirectGridInputs::parse_bad_skin(&mutate(36, 2)).is_err());
        assert!(DirectGridInputs::parse_bad_skin(&mutate(40, 5)).is_err());
        let schedule_offset = HEADER_SIZE + 2 * DRAW_STRIDE;
        assert!(DirectGridInputs::parse_bad_skin(
            &mutate(schedule_offset, DRAW_TYPE_OUTER_CUBICS,)
        )
        .is_err());
        assert!(DirectGridInputs::parse_bad_skin(&bytes[..bytes.len() - 1]).is_err());
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

    fn bad_skin_path() -> nuxie_render_api::RawPath {
        let mut path = nuxie_render_api::RawPath::new();
        path.move_to(-81.5608521, -65.8947601);
        path.cubic_to(
            -81.5608521,
            -65.8947601,
            -105.346954,
            -143.298767,
            -31.1603794,
            -150.845825,
        );
        path.cubic_to(
            -10.2647314,
            -146.784561,
            -5.52482748,
            -138.151382,
            3.95540905,
            -135.285233,
        );
        path.cubic_to(
            13.435585,
            -132.419128,
            45.7343864,
            -139.608963,
            67.0147171,
            -116.697594,
        );
        path.cubic_to(
            88.295105,
            -93.7862701,
            101.926819,
            -73.546402,
            114.122017,
            -9.98956394,
        );
        path.cubic_to(
            127.370155, 43.0014648, 179.42717, 98.3242035, 219.327179, 105.924187,
        );
        path.cubic_to(
            259.227203, 113.524231, 219.327179, 296.024231, 219.327179, 296.024231,
        );
        path.cubic_to(
            219.327179,
            296.024231,
            14.4906435,
            346.763397,
            -28.0207996,
            347.893097,
        );
        path.cubic_to(
            -70.5313416,
            348.922821,
            -254.072845,
            382.324249,
            -283.272827,
            345.224243,
        );
        path.cubic_to(
            -312.472809,
            308.024261,
            -167.772812,
            149.824234,
            -167.772812,
            149.824234,
        );
        path.cubic_to(
            -167.772812,
            149.824234,
            -173.372818,
            117.124199,
            -171.672791,
            98.2241821,
        );
        path.cubic_to(
            -169.270737,
            76.4746246,
            -171.432159,
            54.370945,
            -137.79129,
            30.8296814,
        );
        path.cubic_to(
            -104.150368,
            7.28838682,
            -136.469254,
            -24.0374756,
            -120.125793,
            -40.795002,
        );
        path.cubic_to(
            -98.6691818,
            -62.6391525,
            -81.5608521,
            -65.8947601,
            -81.5608521,
            -65.8947601,
        );
        path.close();
        path
    }

    #[test]
    #[ignore = "requires RIVE_CPP_DIRECT_BAD_SKIN_INPUTS from the C++ WebGPU oracle"]
    fn configured_cpp_bad_skin_preparation_matches_record_for_record() {
        use bytemuck::Zeroable;
        use nuxie_render_api::{FillRule, Mat2D};

        let path = std::env::var_os("RIVE_CPP_DIRECT_BAD_SKIN_INPUTS").expect(
            "RIVE_CPP_DIRECT_BAD_SKIN_INPUTS is required for the ignored direct-bad-skin test",
        );
        assert!(!path.is_empty(), "RIVE_CPP_DIRECT_BAD_SKIN_INPUTS is empty");
        let path = std::path::PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "RIVE_CPP_DIRECT_BAD_SKIN_INPUTS must be absolute"
        );
        let bytes = std::fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "failed to read C++ direct-bad-skin inputs at {}: {error}",
                path.display()
            )
        });
        let capture = DirectGridInputs::parse_bad_skin(&bytes).unwrap_or_else(|error| {
            panic!(
                "malformed C++ direct-bad-skin inputs at {}: {error}",
                path.display()
            )
        });
        assert_eq!(capture.draws.len(), 4);
        assert_eq!(capture.contours.len(), 1);
        let transform = Mat2D([
            1.005_015_73,
            0.116_219_193,
            -0.116_219_17,
            1.005_015_61,
            550.433_167,
            361.510_925,
        ]);
        let tessellation = crate::draw::build_interior_tessellation(
            &bad_skin_path(),
            transform,
            FillRule::NonZero,
            true,
        )
        .expect("bad_skin hair must prepare an interior tessellation");
        let rust_contours = tessellation
            .contours
            .iter()
            .map(|contour| ContourRecord {
                midpoint_x_bits: contour.midpoint[0].to_bits(),
                midpoint_y_bits: contour.midpoint[1].to_bits(),
                path_id: contour.path_id,
                vertex_index0: contour.vertex_index0,
            })
            .collect::<Vec<_>>();
        assert_eq!(rust_contours.len(), capture.contours.len());
        for (index, (rust, cpp)) in rust_contours.iter().zip(&capture.contours).enumerate() {
            assert_eq!(rust, cpp, "bad-skin contour record {index} differs");
        }

        let factory = crate::WgpuFactory::new(999, 720).unwrap();
        let height = crate::draw::tessellation_texture_height(&tessellation.spans);
        assert_eq!([capture.tess_width, capture.tess_height], [2048, height]);
        let uniforms = crate::analytic_uniforms(999, 720, height);
        let paths = [crate::gpu::PathData::zeroed(), tessellation.path];
        let mut encoder =
            factory
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("nuxie-direct-bad-skin-tessellation-encoder"),
                });
        let mut tessellation_uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let texture = factory.context.tessellator.encode(
            &factory.context.device,
            &mut tessellation_uploads,
            &mut encoder,
            &factory.context.feather_lut.view,
            &tessellation.spans,
            &uniforms,
            &paths,
            &tessellation.contours,
            height,
        );
        let bytes_per_row = capture.tess_width * TEXEL_STRIDE as u32;
        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-direct-bad-skin-tessellation-readback"),
                size: u64::from(bytes_per_row) * u64::from(height),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        tessellation_uploads.flush(&factory.context.queue);
        factory.context.queue.submit(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap()
        });
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        receiver.recv().unwrap().unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        let rust_texels = mapped
            .chunks_exact(TEXEL_STRIDE)
            .map(|texel| {
                [
                    u32::from_le_bytes(texel[0..4].try_into().unwrap()),
                    u32::from_le_bytes(texel[4..8].try_into().unwrap()),
                    u32::from_le_bytes(texel[8..12].try_into().unwrap()),
                    u32::from_le_bytes(texel[12..16].try_into().unwrap()),
                ]
            })
            .collect::<Vec<_>>();
        assert_eq!(rust_texels.len(), capture.texels.len());
        for (index, (rust, cpp)) in rust_texels.iter().zip(&capture.texels).enumerate() {
            assert_eq!(rust, cpp, "bad-skin tessellation texel {index} differs");
        }
        drop(mapped);
        readback.unmap();

        let rust_triangles = tessellation
            .triangles
            .iter()
            .map(|vertex| TriangleRecord {
                x_bits: vertex.point[0].to_bits(),
                y_bits: vertex.point[1].to_bits(),
                weight_path_id: vertex.weight_path_id as u32,
            })
            .collect::<Vec<_>>();
        assert_eq!(rust_triangles.len(), capture.triangles.len());
        let rust_canonical = canonical_triangles(&rust_triangles);
        let cpp_canonical = canonical_triangles(&capture.triangles);
        for (index, (rust, cpp)) in rust_canonical.iter().zip(&cpp_canonical).enumerate() {
            assert_eq!(rust, cpp, "bad-skin canonical triangle {index} differs");
        }
        for (index, (rust, cpp)) in rust_triangles.iter().zip(&capture.triangles).enumerate() {
            assert_eq!(rust, cpp, "bad-skin TriangleVertex record {index} differs");
        }
    }
}
