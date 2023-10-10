#![no_std]

use strolle_gpu::prelude::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_diffuse_temporal_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    prev_indirect_diffuse_temporal_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, global_id.xy());
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);
    let reprojection = reprojection_map.get(screen_pos);

    // ---

    let mut reservoir = IndirectReservoir::default();
    let mut reservoir_p_hat = 0.0;

    let d0 = unsafe { *indirect_samples.get_unchecked(3 * screen_idx) };
    let d1 = unsafe { *indirect_samples.get_unchecked(3 * screen_idx + 1) };
    let d2 = unsafe { *indirect_samples.get_unchecked(3 * screen_idx + 2) };

    if d0.w.to_bits() == 1 {
        let sample = IndirectReservoirSample {
            radiance: d1.xyz(),
            hit_point: d0.xyz(),
            sample_point: d2.xyz(),
            sample_normal: Normal::decode(vec2(d1.w, d2.w)),
            frame: params.frame,
        };

        reservoir_p_hat = sample.temporal_p_hat();
        reservoir.add(&mut wnoise, sample, reservoir_p_hat);
    }

    // ---

    let mut sample_idx = 0;
    let mut reservoir_m_sum = 0.0;

    while reservoir.m_sum < 32.0 && sample_idx < 5 {
        let mut rhs_pos = if reprojection.is_some() {
            reprojection.prev_pos_round().as_ivec2()
        } else {
            screen_pos.as_ivec2()
        };

        if reprojection.is_none() {
            rhs_pos += (wnoise.sample_disk() * 16.0).as_ivec2();
        }

        if reprojection.is_none() || sample_idx > 0 {
            let offset = wnoise.sample_int();
            let offset = ivec2((offset % 2) as i32, ((offset >> 2) % 2) as i32);
            let offset = offset - ivec2(1, 1);

            rhs_pos += offset;
            rhs_pos.x ^= 3;
            rhs_pos.y ^= 3;
            rhs_pos -= offset;
        }

        sample_idx += 1;

        let rhs_pos = camera.contain(rhs_pos);

        let rhs_similarity = prev_surface_map
            .get(rhs_pos)
            .evaluate_similarity_to(&surface);

        if rhs_similarity < 0.5 {
            continue;
        }

        let mut rhs = IndirectReservoir::read(
            prev_indirect_diffuse_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        if rhs.is_empty() {
            continue;
        }

        let rhs_p_hat = rhs.sample.temporal_p_hat();

        rhs.m_sum *= rhs_similarity;

        if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat) {
            reservoir_p_hat = rhs_p_hat;
        }

        if sample_idx == 1 {
            reservoir_m_sum = rhs.m_sum;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(reservoir_p_hat);
    reservoir.m_sum = (reservoir_m_sum + 1.0).min(32.0);
    reservoir.write(indirect_diffuse_temporal_reservoirs, screen_idx);
}
