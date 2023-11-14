#![no_std]

use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 4)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 5)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_candidates: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    direct_curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let lights = LightsView::new(lights);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

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

    let candidate = unsafe { *direct_candidates.index_unchecked(screen_idx) };

    let mut res = DirectReservoir::default();
    let mut res_p_hat = 0.0;

    let mut other = DirectReservoir::default();
    let mut other_p_hat = 0.0;
    let mut other_ray = Ray::default();
    let mut other_dist = 0.0;

    // ---

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        other = DirectReservoir::read(
            direct_prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        let other_light_id = other.sample.light_id;

        let filter = BilinearFilter::reproject(reprojection, move |pos| {
            let res = DirectReservoir::read(
                direct_prev_reservoirs,
                camera.screen_to_idx(pos),
            );

            let res_weight = if res.sample.light_id == other_light_id {
                1.0
            } else {
                0.0
            };

            (vec4(res.w, 0.0, 0.0, 0.0), res_weight)
        });

        other.w = filter.x;

        if other.sample.is_valid(lights) {
            if other.cooldown > 0 {
                res.cooldown = other.cooldown - 1;
            }

            other_p_hat = other.sample.p_hat(lights, hit);

            if debug::DIRECT_VALIDATION_ENABLED && params.frame % 2 == 0 {
                (other_ray, other_dist) = other.sample.ray(hit);
            } else {
                if res.merge(&mut wnoise, &other, other_p_hat) {
                    res_p_hat = other_p_hat;
                }

                other.m = 0.0;
            }
        } else {
            other.m = 0.0;
        }
    }

    if other.m == 0.0 && candidate.x > 0.0 {
        let light_id = LightId::new(candidate.z.to_bits());

        (other_ray, other_dist) =
            lights.get(light_id).ray(&mut wnoise, hit.point);

        other = DirectReservoir {
            reservoir: Reservoir {
                sample: DirectReservoirSample {
                    light_id,
                    light_point: other_ray.origin(),
                    visibility: 0,
                },
                w_sum: 0.0,
                m: candidate.x,
                w: candidate.y,
            },
            cooldown: 0,
        };

        other_p_hat = candidate.w;
    }

    // ---

    if other.m > 0.0 {
        let has_changed_visibility;

        let (is_occluded, is_dirty) = other_ray.intersect(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            other_dist,
        );

        if is_occluded {
            has_changed_visibility = other.sample.visibility == 1;

            other.w = 0.0;
            other.sample.visibility = 2;
        } else {
            has_changed_visibility = other.sample.visibility == 2;

            other.sample.visibility = 1;
        }

        if has_changed_visibility && is_dirty {
            res.cooldown = 32;
        }

        if res.merge(&mut wnoise, &other, other_p_hat) {
            res_p_hat = other_p_hat;
        }
    }

    // ---

    res.normalize(res_p_hat);
    res.write(direct_curr_reservoirs, screen_idx);
}
