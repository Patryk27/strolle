#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)] direct_hits: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    indirect_diffuse_temporal_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 7, storage_buffer)]
    prev_indirect_diffuse_temporal_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        WhiteNoise::new(params.seed, global_id.xy()),
        camera,
        SurfaceMap::new(surface_map),
        SurfaceMap::new(prev_surface_map),
        ReprojectionMap::new(reprojection_map),
        direct_hits,
        indirect_samples,
        indirect_diffuse_temporal_reservoirs,
        prev_indirect_diffuse_temporal_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    params: &PassParams,
    mut wnoise: WhiteNoise,
    camera: &Camera,
    surface_map: SurfaceMap,
    prev_surface_map: SurfaceMap,
    reprojection_map: ReprojectionMap,
    direct_hits: TexRgba32f,
    indirect_samples: &[Vec4],
    indirect_diffuse_temporal_reservoirs: &mut [Vec4],
    prev_indirect_diffuse_temporal_reservoirs: &[Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);
    let screen_surface = surface_map.get(screen_pos);
    let screen_point = direct_hits.read(screen_pos).xyz();

    // -------------------------------------------------------------------------

    let mut p_hat = Default::default();
    let mut reservoir = IndirectReservoir::default();

    let sample_pos = if IndirectReservoir::expects_diffuse_sample(
        screen_pos,
        params.frame,
    ) {
        screen_pos
    } else {
        screen_pos + uvec2(1, 0)
    };

    let can_use_sample = if sample_pos == screen_pos {
        true
    } else if sample_pos.x < camera.screen_size().x {
        surface_map
            .get(sample_pos)
            .evaluate_similarity_to(&screen_surface)
            > 0.5
    } else {
        false
    };

    if can_use_sample {
        let sample_idx = camera.screen_to_idx(sample_pos);

        let d0 = unsafe { *indirect_samples.get_unchecked(3 * sample_idx + 0) };
        let d1 = unsafe { *indirect_samples.get_unchecked(3 * sample_idx + 1) };
        let d2 = unsafe { *indirect_samples.get_unchecked(3 * sample_idx + 2) };

        if d0.w.to_bits() == 1 {
            let sample = IndirectReservoirSample {
                radiance: d1.xyz(),
                hit_point: d0.xyz(),
                sample_point: d2.xyz(),
                sample_normal: Normal::decode(vec2(d1.w, d2.w)),
                frame: params.frame,
            };

            p_hat = sample.temporal_p_hat();

            reservoir.add(&mut wnoise, sample, p_hat);
        }
    }

    // -------------------------------------------------------------------------

    let mut sample_idx = 0;

    let sample_offsets =
        [ivec2(-1, -1), ivec2(1, 1), ivec2(-1, 1), ivec2(1, -1)];

    let sample_xors = [ivec2(3, 3), ivec2(2, 1), ivec2(1, 2), ivec2(3, 3)];
    let sample_xor = sample_xors[(params.frame % 4) as usize];

    while reservoir.m_sum < 30.0 && sample_idx < 5 {
        sample_idx += 1;

        let mut rhs_pos = screen_pos.as_ivec2();

        if sample_idx > 1 {
            rhs_pos += sample_offsets[(params.frame % 4) as usize];

            rhs_pos += sample_offsets
                [((sample_idx + (params.frame ^ 1) - 1) % 4) as usize];

            rhs_pos = rhs_pos ^ sample_xor;
        }

        let rhs_pos = camera.contain(rhs_pos);
        let rhs_reprojection = reprojection_map.get(rhs_pos);

        let rhs_pos = if rhs_reprojection.is_some() {
            rhs_reprojection.prev_screen_pos()
        } else {
            rhs_pos
        };

        let mut rhs = IndirectReservoir::read(
            prev_indirect_diffuse_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        if rhs.is_empty() {
            continue;
        }

        if prev_surface_map
            .get(rhs_pos)
            .evaluate_similarity_to(&screen_surface)
            < 0.5
        {
            continue;
        }

        if rhs.sample.hit_point.distance(screen_point) > 2.0 {
            continue;
        }

        if rhs_reprojection.is_none() {
            rhs.m_sum = rhs.m_sum.powf(0.33);
        }

        let rhs_p_hat = rhs.sample.temporal_p_hat();

        if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat) {
            p_hat = rhs_p_hat;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 10.0, 20.0);
    reservoir.write(indirect_diffuse_temporal_reservoirs, screen_idx);
}
