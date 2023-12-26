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
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
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

    let mut main = DirectReservoir::default();
    let mut main_pdf = 0.0;

    let mut curr_m = 0.0;
    let mut prev_m = 0.0;

    let mut selected = 0;

    // ---

    let curr = DirectReservoir::read(direct_curr_reservoirs, screen_idx);

    if curr.m > 0.0 {
        let curr_pdf = curr.sample.pdf(lights, hit);

        if main.merge(&mut wnoise, &curr, curr_pdf) {
            main_pdf = curr_pdf;
            selected = 1;
        }

        curr_m = curr.m;
    }

    // ---

    let reprojection = reprojection_map.get(screen_pos);
    let mut prev = DirectReservoir::default();

    if reprojection.is_some() {
        prev = DirectReservoir::read(
            direct_prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        prev.clamp_m(20.0 * curr_m.max(1.0));

        let prev_pdf = if prev.sample.exists {
            prev.sample.pdf(lights, hit)
        } else {
            0.0
        };

        if main.merge(&mut wnoise, &prev, prev_pdf) {
            main_pdf = prev_pdf;
            selected = 2;
        }

        prev_m = prev.m;
    }

    // ---

    let mut pi = main_pdf;
    let mut pi_sum = main_pdf * curr_m;

    if (prev_m > 0.0) & main.sample.exists {
        let (ray, dist) = main.sample.ray(hit);
        let mut is_occluded = false;

        if (main.sample.light_id == prev.sample.light_id) & (prev.w == 0.0) {
            is_occluded = true;
        }

        if !is_occluded & (selected == 2) {
            is_occluded |= ray.intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
                dist,
            );
        }

        let ps = if is_occluded {
            0.0
        } else {
            main.sample.pdf(lights, hit)
        };

        pi = if selected == 2 { ps } else { pi };
        pi_sum += ps * prev_m;
    }

    main.normalize_ex(main_pdf, pi, pi_sum);
    main.write(direct_curr_reservoirs, screen_idx);
}
