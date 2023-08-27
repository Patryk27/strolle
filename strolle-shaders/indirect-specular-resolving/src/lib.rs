#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    indirect_specular_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 4)]
    indirect_specular_samples: TexRgba16f,
) {
    let screen_pos = global_id.xy();

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let mut out = Vec4::ZERO;

    let reservoir = IndirectReservoir::read(
        indirect_specular_reservoirs,
        camera.screen_to_idx(screen_pos),
    );

    let radiance = reservoir.sample.radiance * reservoir.w;
    let cosine = reservoir.sample.cosine(&hit);
    let brdf = reservoir.sample.specular_brdf(&hit);

    if reservoir.sample.is_within_specular_lobe_of(&hit)
        && brdf.probability > 0.0
    {
        out += (radiance * cosine * brdf.radiance).extend(brdf.probability);
    }

    let out = if out.w > 0.0 {
        out.xyz() / out.w
    } else {
        Vec3::ZERO
    };

    unsafe {
        indirect_specular_samples
            .write(screen_pos, out.extend(reservoir.m_sum / 8.0));
    }
}
