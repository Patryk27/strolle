#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    indirect_diffuse_samples: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_diffuse_spatial_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let reservoir = IndirectReservoir::read(
        indirect_diffuse_spatial_reservoirs,
        camera.screen_to_idx(screen_pos),
    );

    let radiance = reservoir.sample.radiance * reservoir.w;
    let cosine = reservoir.sample.cosine(&hit);
    let brdf = reservoir.sample.diffuse_brdf(&hit);
    let out = (radiance * cosine * brdf.radiance).extend(Default::default());

    unsafe {
        indirect_diffuse_samples.write(screen_pos, out);
    }
}
