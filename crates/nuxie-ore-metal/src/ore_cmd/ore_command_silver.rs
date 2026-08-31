//! renderer/ore/cmd/ore_command_silver.hpp at e949498e.
#![allow(non_snake_case, non_upper_case_globals)]
use super::{
    ore_command_buffer::{OreCommandBuffer, OreCommandReader},
    ore_commands::*,
    ore_make_replay::decodePods,
};
use crate::cmd::command_stream::WirePod;
pub const kSilverMagic: &[u8; 4] = b"ORES";
pub const kSilverVersion: u64 = 1;
pub const kSilverEpsilon: f32 = 0.0001;
fn var(out: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        out.push(n as u8 | 128);
        n >>= 7;
    }
    out.push(n as u8);
}
fn ints(out: &mut Vec<u8>, values: &[u32]) {
    for &v in values {
        var(out, u64::from(v));
    }
}
fn floats(out: &mut Vec<u8>, values: &[f32]) {
    for &v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
}
pub fn serializeSilver(buffer: &OreCommandBuffer, out: &mut Vec<u8>) {
    out.extend_from_slice(kSilverMagic);
    var(out, kSilverVersion);
    let mut reader = OreCommandReader::new(buffer.command_bytes(), buffer.blob_bytes());
    while let Some(kind) = reader.next::<CommandType>() {
        var(out, kind as u64);
        match kind {
            CommandType::beginRenderPass => {
                let c: BeginRenderPassCmd = reader.read();
                var(out, c.colorCount.into());
                for color in &c.colors[..c.colorCount as usize] {
                    ints(
                        out,
                        &[
                            color.view,
                            color.resolveTarget,
                            color.loadOp as u32,
                            color.storeOp as u32,
                        ],
                    );
                    floats(
                        out,
                        &[color.clearR, color.clearG, color.clearB, color.clearA],
                    );
                }
                let d = c.depthStencil;
                ints(out, &[d.view, d.depthLoadOp as u32, d.depthStoreOp as u32]);
                floats(out, &[d.depthClearValue]);
                ints(
                    out,
                    &[
                        d.stencilLoadOp as u32,
                        d.stencilStoreOp as u32,
                        d.stencilClearValue,
                    ],
                );
            }
            CommandType::setPipeline => {
                let c: SetPipelineCmd = reader.read();
                ints(out, &[c.pipeline]);
            }
            CommandType::setVertexBuffer => {
                let c: SetVertexBufferCmd = reader.read();
                ints(out, &[c.slot, c.buffer, c.offset]);
            }
            CommandType::setIndexBuffer => {
                let c: SetIndexBufferCmd = reader.read();
                ints(out, &[c.buffer, c.format as u32, c.offset]);
            }
            CommandType::setBindGroup => {
                let c: SetBindGroupCmd = reader.read();
                ints(out, &[c.groupIndex, c.bindGroup, c.dynamicOffsetCount]);
                ints(
                    out,
                    &decodePods::<u32>(
                        reader.blob_at(c.dynamicOffsetStart, c.dynamicOffsetCount.wrapping_mul(4)),
                        c.dynamicOffsetCount,
                    ),
                );
            }
            CommandType::setViewport => {
                let c: SetViewportCmd = reader.read();
                floats(out, &[c.x, c.y, c.width, c.height, c.minDepth, c.maxDepth]);
            }
            CommandType::setScissorRect => {
                let c: SetScissorRectCmd = reader.read();
                ints(out, &[c.x, c.y, c.width, c.height]);
            }
            CommandType::setStencilReference => {
                let c: SetStencilReferenceCmd = reader.read();
                ints(out, &[c.reference]);
            }
            CommandType::setBlendColor => {
                let c: SetBlendColorCmd = reader.read();
                floats(out, &[c.r, c.g, c.b, c.a]);
            }
            CommandType::draw => {
                let c: DrawCmd = reader.read();
                ints(
                    out,
                    &[
                        c.vertexCount,
                        c.instanceCount,
                        c.firstVertex,
                        c.firstInstance,
                    ],
                );
            }
            CommandType::drawIndexed => {
                let c: DrawIndexedCmd = reader.read();
                ints(
                    out,
                    &[
                        c.indexCount,
                        c.instanceCount,
                        c.firstIndex,
                        c.baseVertex as u32,
                        c.firstInstance,
                    ],
                );
            }
            CommandType::finish => {}
            CommandType::makeBuffer
            | CommandType::makeTexture
            | CommandType::makeSampler
            | CommandType::makeShaderModule
            | CommandType::makeBindGroupLayout
            | CommandType::makeTextureView
            | CommandType::makePipeline
            | CommandType::makeBindGroup => {
                let m: MakeResourcePOD = reader.read();
                reader.skip(ore_payload_size_of(kind) - MakeResourcePOD::SIZE);
                ints(out, &[m.id, m.generation]);
            }
            CommandType::bufferUpdate => {
                let c: BufferUpdatePOD = reader.read();
                ints(out, &[c.handle, c.offset, c.bytes.size]);
            }
            CommandType::textureUpload => {
                let c: TextureUploadPOD = reader.read();
                ints(out, &[c.handle, c.bytes.size]);
            }
            CommandType::destroyResource => {
                let c: DestroyResourcePOD = reader.read();
                ints(out, &[c.handle, c.generation]);
            }
            CommandType::wrapCanvasView => {
                let c: WrapCanvasViewPOD = reader.read();
                ints(out, &[c.id, c.generation, c.canvasId]);
            }
        }
    }
}
// Restricted BinaryReader projection used by the portable comparator. Overflow
// advances to the end and returns zero, exactly as the upstream reader does.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn end(&self) -> bool {
        self.pos == self.bytes.len()
    }
    fn byte(&mut self) -> u8 {
        if self.end() {
            0
        } else {
            let b = self.bytes[self.pos];
            self.pos += 1;
            b
        }
    }
    fn var(&mut self) -> u64 {
        let mut n = 0u64;
        let mut shift = 0u32;
        loop {
            if self.end() {
                return 0;
            }
            let b = self.byte();
            n |= u64::from(b & 127).wrapping_shl(shift);
            shift = shift.wrapping_add(7);
            if b & 128 == 0 {
                return n;
            }
        }
    }
    fn float(&mut self) -> f32 {
        if self.bytes.len() - self.pos < 4 {
            self.pos = self.bytes.len();
            return 0.;
        }
        let f = f32::from_le_bytes(self.bytes[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        f
    }
}
fn varMatch(field: &str, a: &mut Reader<'_>, b: &mut Reader<'_>) -> Option<u64> {
    let va = a.var();
    let vb = b.var();
    if va != vb {
        eprintln!("ore silver: {} differs {} != {}", field, va, vb);
        None
    } else {
        Some(va)
    }
}
fn floatMatch(field: &str, a: &mut Reader<'_>, b: &mut Reader<'_>) -> bool {
    let va = a.float();
    let vb = b.float();
    if (va - vb).abs() > kSilverEpsilon {
        eprintln!("ore silver: {} differs {} != {}", field, va, vb);
        false
    } else {
        true
    }
}
pub fn silverMatch(expected: &[u8], actual: &[u8]) -> bool {
    let mut a = Reader::new(expected);
    let mut b = Reader::new(actual);
    for &magic in kSilverMagic {
        if a.byte() != magic || b.byte() != magic {
            eprintln!("ore silver: bad magic");
            return false;
        }
    }
    if varMatch("version", &mut a, &mut b).is_none() {
        return false;
    }
    macro_rules! vi {($($field:expr),* $(,)?)=>{$(if varMatch($field,&mut a,&mut b).is_none(){return false;})*};}
    macro_rules! fl {($($field:expr),* $(,)?)=>{$(if !floatMatch($field,&mut a,&mut b){return false;})*};}
    while !a.end() {
        if b.end() {
            eprintln!("ore silver: actual stream is shorter");
            return false;
        }
        let Some(op) = varMatch("opcode", &mut a, &mut b) else {
            return false;
        };
        match op {
            0 => {
                let Some(count) = varMatch("colorCount", &mut a, &mut b) else {
                    return false;
                };
                for _ in 0..count {
                    vi!(
                        "color.view",
                        "color.resolveTarget",
                        "color.loadOp",
                        "color.storeOp"
                    );
                    fl!(
                        "color.clearR",
                        "color.clearG",
                        "color.clearB",
                        "color.clearA"
                    );
                }
                vi!("ds.view", "ds.depthLoadOp", "ds.depthStoreOp");
                fl!("ds.depthClearValue");
                vi!(
                    "ds.stencilLoadOp",
                    "ds.stencilStoreOp",
                    "ds.stencilClearValue"
                );
            }
            1 => {
                vi!("pipeline");
            }
            2 => {
                vi!("vb.slot", "vb.buffer", "vb.offset");
            }
            3 => {
                vi!("ib.buffer", "ib.format", "ib.offset");
            }
            4 => {
                vi!("bg.groupIndex", "bg.bindGroup");
                let Some(count) = varMatch("bg.dynamicOffsetCount", &mut a, &mut b) else {
                    return false;
                };
                for _ in 0..count {
                    vi!("bg.dynamicOffset");
                }
            }
            5 => {
                fl!(
                    "vp.x",
                    "vp.y",
                    "vp.width",
                    "vp.height",
                    "vp.minDepth",
                    "vp.maxDepth"
                );
            }
            6 => {
                vi!("sc.x", "sc.y", "sc.width", "sc.height");
            }
            7 => {
                vi!("stencilRef");
            }
            8 => {
                fl!("blend.r", "blend.g", "blend.b", "blend.a");
            }
            9 => {
                vi!(
                    "draw.vertexCount",
                    "draw.instanceCount",
                    "draw.firstVertex",
                    "draw.firstInstance"
                );
            }
            10 => {
                vi!(
                    "drawIndexed.indexCount",
                    "drawIndexed.instanceCount",
                    "drawIndexed.firstIndex",
                    "drawIndexed.baseVertex",
                    "drawIndexed.firstInstance"
                );
            }
            11 => {}
            _ => {
                eprintln!("ore silver: unknown opcode {}", op);
                return false;
            }
        }
    }
    if !b.end() {
        eprintln!("ore silver: actual stream is longer");
        return false;
    }
    true
}
