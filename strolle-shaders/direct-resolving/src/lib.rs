#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] direct_hits: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_initial_samples: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5)] direct_samples: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    direct_spatial_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let lights = LightsView::new(lights);

    let mut hit = Hit::from_direct(
        camera.ray(screen_pos),
        direct_hits.read(screen_pos).xyz(),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // TODO describe
    hit.gbuffer.base_color = Vec4::splat(1.0);

    let out = if hit.is_some() {
        let reservoir = DirectReservoir::read(
            direct_spatial_reservoirs,
            camera.screen_to_idx(screen_pos),
        );

        let contribution = if reservoir.sample.is_sky() {
            reservoir.sample.light_contribution
        } else {
            lights
                .get(reservoir.sample.light_id)
                .contribution(hit)
                .diffuse
        };

        contribution * reservoir.w
    } else {
        unsafe { direct_initial_samples.get_unchecked(2 * screen_idx).xyz() }
    };

    unsafe {
        direct_samples.write(screen_pos, out.extend(1.0));
    }
}
