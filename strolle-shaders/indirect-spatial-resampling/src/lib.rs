#![no_std]

use core::f32::consts::PI;

use spirv_std::glam::{vec2, UVec2, UVec3, Vec3Swizzles, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use strolle_gpu::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &IndirectSpatialResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    direct_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    geometry_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_temporal_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_spatial_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    past_indirect_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        direct_hits_d0,
        GeometryMap::new(geometry_map),
        ReprojectionMap::new(reprojection_map),
        indirect_temporal_reservoirs,
        indirect_spatial_reservoirs,
        past_indirect_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    params: &IndirectSpatialResamplingPassParams,
    camera: &Camera,
    direct_hits_d0: TexRgba32f,
    geometry_map: GeometryMap,
    reprojection_map: ReprojectionMap,
    indirect_temporal_reservoirs: &[Vec4],
    indirect_spatial_reservoirs: &mut [Vec4],
    past_indirect_spatial_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, global_id);
    let global_idx = camera.half_screen_to_idx(global_id);
    let screen_pos = upsample(global_id, params.frame);
    let half_viewport_size = camera.viewport_size().as_vec2() / 2.0;

    let mut reservoir = IndirectReservoir::default();

    // -------------------------------------------------------------------------

    let reprojection =
        reprojection_map.get(upsample(global_id, params.frame - 1));

    if reprojection.is_valid() {
        let mut prev_reservoir = IndirectReservoir::read(
            past_indirect_spatial_reservoirs,
            camera.half_screen_to_idx(reprojection.prev_screen_pos() / 2),
        );

        prev_reservoir.m_sum *= reprojection.confidence.powi(2).max(0.1);

        reservoir.merge(
            &mut noise,
            &prev_reservoir,
            prev_reservoir.sample.p_hat(),
        );
    }

    // -------------------------------------------------------------------------

    let mut p_hat = reservoir.sample.p_hat();
    let curr_geo = geometry_map.get(screen_pos);

    let direct_hit_point =
        Hit::deserialize_point(direct_hits_d0.read(screen_pos));

    let mut sample_idx = 0;
    let mut sample_radius = 32.0;

    while sample_idx < 6 {
        let rhs_pos_delta = {
            let radius = noise.sample().sqrt() * sample_radius;
            let angle = noise.sample() * 2.0 * PI;

            vec2(angle.cos(), angle.sin()) * radius
        };

        let rhs_pos = global_id.as_vec2() + rhs_pos_delta;

        if rhs_pos.x < 0.0
            || rhs_pos.y < 0.0
            || rhs_pos.x >= half_viewport_size.x
            || rhs_pos.y >= half_viewport_size.y
        {
            sample_idx += 1;
            sample_radius *= 0.5;
            continue;
        }

        let rhs_pos = rhs_pos.as_uvec2();

        let rhs_similarity = geometry_map
            .get(upsample(rhs_pos, params.frame))
            .evaluate_similarity_to(&curr_geo);

        if rhs_similarity < 0.25 {
            sample_idx += 1;
            sample_radius *= 0.5;
            continue;
        }

        let rhs = IndirectReservoir::read(
            indirect_temporal_reservoirs,
            camera.half_screen_to_idx(rhs_pos),
        );

        let rhs_p_hat = rhs.sample.p_hat();
        let rhs_jacobian = rhs.sample.jacobian(direct_hit_point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if rhs_jacobian < 1.0 / 10.0 || rhs_jacobian > 10.0 {
            sample_idx += 1;
            sample_radius *= 0.5;
            continue;
        }

        let rhs_jacobian = rhs_jacobian.clamp(1.0 / 3.0, 3.0);

        if reservoir.merge(
            &mut noise,
            &rhs,
            rhs_p_hat * rhs_jacobian * rhs_similarity,
        ) {
            p_hat = rhs_p_hat;
        }

        sample_idx += 1;
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 10.0, 500.0);
    reservoir.write(indirect_spatial_reservoirs, global_idx);
}
