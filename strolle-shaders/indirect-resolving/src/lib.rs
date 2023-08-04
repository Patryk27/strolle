#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_hits: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)]
    indirect_diffuse_samples: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_diffuse_spatial_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 6)]
    indirect_specular_samples: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 7, storage_buffer)]
    indirect_specular_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();

    let direct_hit = Hit::from_direct(
        camera.ray(screen_pos),
        direct_hits.read(screen_pos).xyz(),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let diffuse = {
        let reservoir = IndirectReservoir::read(
            indirect_diffuse_spatial_reservoirs,
            camera.screen_to_idx(screen_pos),
        );

        let radiance = reservoir.sample.radiance * reservoir.w;
        let cosine = reservoir.sample.cosine(&direct_hit);
        let brdf = reservoir.sample.diffuse_brdf();

        (radiance * cosine * brdf).extend(reservoir.m_sum / 500.0)
    };

    let specular = {
        let reservoir = IndirectReservoir::read(
            indirect_specular_reservoirs,
            camera.screen_to_idx(screen_pos),
        );

        let radiance = reservoir.sample.radiance * reservoir.w;
        let cosine = reservoir.sample.cosine(&direct_hit);
        let brdf = reservoir.sample.specular_brdf(&direct_hit);

        (radiance * cosine * brdf).extend(reservoir.m_sum / 20.0)
    };

    unsafe {
        indirect_diffuse_samples.write(screen_pos, diffuse);
        indirect_specular_samples.write(screen_pos, specular);
    }
}
