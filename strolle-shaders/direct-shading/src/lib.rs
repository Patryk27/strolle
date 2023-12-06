#![no_std]

use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 1, uniform)] world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] direct_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    direct_candidates: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let lights = LightsView::new(lights);

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

    if hit.is_none() {
        return;
    }

    // ---

    let mut res = EphemeralReservoir::default();
    let mut res_p_hat = 0.0;

    let light_prob = 1.0 / (world.light_count as f32);
    let mut light_idx = 0;

    while light_idx < world.light_count {
        let light_id = LightId::new(light_idx);
        let light_radiance = lights.get(light_id).radiance(hit);

        let sample = EphemeralReservoirSample {
            light_id,
            light_radiance,
        };

        let sample_p_hat = sample.p_hat();

        if res.update(&mut wnoise, sample, sample_p_hat / light_prob) {
            res_p_hat = sample_p_hat;
        }

        light_idx += 1;
    }

    // ---

    res.normalize(res_p_hat);

    let candidate = vec4(
        res.m,
        res.w,
        f32::from_bits(res.sample.light_id.get()),
        res_p_hat,
    );

    unsafe {
        *direct_candidates.index_unchecked_mut(screen_idx) = candidate;
    }
}
