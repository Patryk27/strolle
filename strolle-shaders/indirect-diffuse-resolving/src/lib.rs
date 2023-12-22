#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)]
    indirect_diffuse_samples: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_diffuse_spatial_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);

    if !camera.contains(screen_pos) {
        return;
    }

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // -------------------------------------------------------------------------

    let res = IndirectReservoir::read(
        indirect_diffuse_spatial_reservoirs,
        screen_idx,
    );

    let res_cosine = res.sample.cosine(&hit);

    let out = rgb_to_ycocg(res.sample.radiance * res.w * res_cosine)
        .extend(Default::default());

    unsafe {
        indirect_diffuse_samples.write(screen_pos, out);
    }
}
