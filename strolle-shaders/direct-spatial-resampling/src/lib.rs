#![no_std]

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
    #[spirv(descriptor_set = 1, binding = 1)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_curr_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_next_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let lights = LightsView::new(lights);
    let surface_map = SurfaceMap::new(surface_map);

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
        DirectReservoir::default().write(direct_next_reservoirs, screen_idx);
        return;
    }

    let surface = hit.as_surface();

    // ---

    let (lhs, lhs_p_hat) = {
        let mut res = DirectReservoir::read(
            direct_curr_reservoirs,
            camera.screen_to_idx(screen_pos),
        );

        let res_p_hat = res.sample.p_hat(lights, hit);

        res.clamp_m(512.0);

        (res, res_p_hat)
    };

    let mut cooldown = lhs.cooldown;

    // ---

    let (rhs, rhs_p_hat) = {
        let mut res = DirectReservoir::default();
        let mut res_p_hat = 0.0;
        let mut sample_idx = 0;

        while sample_idx < 5 {
            sample_idx += 1;

            let sample_pos = screen_pos.as_vec2() + wnoise.sample_disk() * 10.0;
            let sample_pos = camera.contain(sample_pos.as_ivec2());

            if sample_pos == screen_pos {
                continue;
            }

            let sample_surface = surface_map.get(sample_pos);

            if sample_surface.is_sky() {
                continue;
            }

            if (sample_surface.depth - surface.depth).abs()
                > 0.2 * surface.depth
            {
                continue;
            }

            if sample_surface.normal.dot(surface.normal) < 0.8 {
                continue;
            }

            let sample = DirectReservoir::read(
                direct_curr_reservoirs,
                camera.screen_to_idx(sample_pos),
            );

            if sample.cooldown > cooldown {
                cooldown = sample.cooldown - 1;
            }

            let sample_p_hat = sample.sample.p_hat(lights, hit);

            if res.merge(&mut wnoise, &sample, sample_p_hat) {
                res_p_hat = sample_p_hat;
            }
        }

        if res.m > 0.0 {
            let (ray, ray_distance) = res.sample.ray(hit);

            let (is_occluded, _) = ray.intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
                ray_distance,
            );

            if is_occluded {
                res.w = 0.0;
                res.sample.visibility = 2;
            } else {
                res.normalize(res_p_hat);
                res.sample.visibility = 1;
            }

            res.clamp_m(64.0);
        }

        (res, res_p_hat)
    };

    // -------------------------------------------------------------------------

    let mut res = DirectReservoir::default();
    let mut res_p_hat = 0.0;

    if res.merge(&mut wnoise, &lhs, lhs_p_hat) {
        res_p_hat = lhs_p_hat;
    }

    if res.merge(&mut wnoise, &rhs, rhs_p_hat) {
        res_p_hat = rhs_p_hat;
    }

    res.normalize(res_p_hat);

    if cooldown > 0 {
        res.m = 1.0;
        res.cooldown = cooldown;
    }

    res.write(direct_next_reservoirs, screen_idx);
}
