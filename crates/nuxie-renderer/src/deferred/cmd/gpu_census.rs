//! renderer/cmd/gpu_census.hpp at e949498e.
use super::render_replay::{Resident, ResourceTable};
use crate::deferred::ore::ore_make_replay::{OreKind, OreResident};
use nuxie_ore_metal::{gpu_resource::AnyResourceHandle, types::textureFormatBytesPerTexel};
#[derive(Default, Debug, Clone, Copy)]
pub struct GpuCensus {
    pub image_bytes: u64,
    pub buffer_bytes: u64,
    pub ore_texture_bytes: u64,
    pub ore_buffer_bytes: u64,
    pub images: u32,
    pub buffers: u32,
    pub paths: u32,
    pub paints: u32,
    pub shaders: u32,
    pub ore_textures: u32,
    pub ore_buffers: u32,
    pub ore_other: u32,
    pub slots_2d: u32,
    pub slots_ore: u32,
}
impl GpuCensus {
    pub fn total_bytes(&self) -> u64 {
        self.image_bytes + self.buffer_bytes + self.ore_texture_bytes + self.ore_buffer_bytes
    }
    pub fn live_objects(&self) -> u32 {
        self.images
            + self.buffers
            + self.paths
            + self.paints
            + self.shaders
            + self.ore_textures
            + self.ore_buffers
            + self.ore_other
    }
}
pub fn ore_texture_nominal_bytes(texture: &AnyResourceHandle) -> u64 {
    let bytes = textureFormatBytesPerTexel(texture.format().expect("resident texture"));
    if bytes == 0 {
        return 0;
    }
    let mut texels = 0;
    let mut width = texture.width().unwrap();
    let mut height = texture.height().unwrap();
    for _ in 0..texture.numMipmaps().unwrap().max(1) {
        texels += u64::from(width) * u64::from(height);
        if width == 1 && height == 1 {
            break;
        }
        width = (width >> 1).max(1);
        height = (height >> 1).max(1);
    }
    texels
        * u64::from(texture.depthOrArrayLayers().unwrap().max(1))
        * u64::from(texture.sampleCount().unwrap().max(1))
        * u64::from(bytes)
}
fn count_live<T: Clone>(resident: &Resident<T>, slots: &mut u32) -> u32 {
    *slots += resident.objects.len() as u32;
    resident.objects.iter().filter(|o| o.is_some()).count() as u32
}
pub fn take_gpu_census(table: &ResourceTable, ore: &OreResident) -> GpuCensus {
    let mut census = GpuCensus::default();
    census.paths = count_live(&table.paths, &mut census.slots_2d);
    census.paints = count_live(&table.paints, &mut census.slots_2d);
    census.shaders = count_live(&table.shaders, &mut census.slots_2d);
    census.buffers = count_live(&table.buffers, &mut census.slots_2d);
    census.images = count_live(&table.images, &mut census.slots_2d);
    for image in table.images.objects.iter().flatten() {
        census.image_bytes += u64::from(image.width()) * u64::from(image.height()) * 4;
    }
    for buffer in table.buffers.objects.iter().flatten() {
        census.buffer_bytes += buffer.borrow().size_in_bytes() as u64;
    }
    census.slots_ore = ore.objects.len() as u32;
    for (index, object) in ore.objects.iter().enumerate() {
        let Some(object) = object else {
            continue;
        };
        match ore.kinds[index] {
            OreKind::texture => {
                census.ore_textures += 1;
                census.ore_texture_bytes += ore_texture_nominal_bytes(object);
            }
            OreKind::buffer => {
                census.ore_buffers += 1;
                census.ore_buffer_bytes += u64::from(object.size().expect("resident buffer"));
            }
            _ => census.ore_other += 1,
        }
    }
    census
}
