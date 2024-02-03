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
    #[spirv(descriptor_set = 1, binding = 1)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    input_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    output_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let lights = LightsView::new(lights);
    let prim_surface_map = SurfaceMap::new(prim_surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    // TODO optimization: reading just the surface map should be sufficient
    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.is_none() {
        return;
    }

    // ---

    let mut main = DiReservoir::default();
    let mut main_nth = 0;
    let mut main_pdf = 0.0;

    // ---

    let lhs = DiReservoir::read(input_reservoirs, screen_idx);

    let lhs_pdf = if lhs.is_empty() {
        0.0
    } else {
        lhs.sample.pdf(lights, hit.point, hit.gbuffer.normal)
    };

    if main.merge(&mut wnoise, &lhs, lhs_pdf) {
        main_nth = 1;
        main_pdf = lhs_pdf;
    }

    // ---

    let mut rhs = DiReservoir::default();
    let mut rhs_hit_point = Vec3::ZERO;
    let mut rhs_hit_normal = Vec3::ZERO;

    let mut sample_idx = 0;
    let max_samples = if params.frame % 3 == 0 { 8 } else { 0 };
    let max_radius = 64.0;

    while sample_idx < max_samples {
        let sample_dist = wnoise.sample_disk() * max_radius;

        let sample_pos =
            camera.contain((screen_pos.as_vec2() + sample_dist).as_ivec2());

        sample_idx += 1;

        let sample_surface = prim_surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            continue;
        }

        if (sample_surface.depth - hit.gbuffer.depth).abs()
            > 0.2 * hit.gbuffer.depth
        {
            continue;
        }

        if sample_surface.normal.dot(hit.gbuffer.normal) < 0.8 {
            continue;
        }

        rhs = DiReservoir::read(
            input_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        let rhs_pdf = if rhs.is_empty() {
            0.0
        } else {
            rhs.sample.pdf(lights, hit.point, hit.gbuffer.normal)
        };

        if rhs_pdf <= 0.0 {
            rhs.m = 0.0;
            continue;
        }

        rhs_hit_point = camera.ray(sample_pos).at(sample_surface.depth)
            + sample_surface.normal * Hit::NUDGE_OFFSET;

        rhs_hit_normal = sample_surface.normal;

        if main.merge(&mut wnoise, &rhs, rhs_pdf) {
            main_nth = 2;
            main_pdf = rhs_pdf;
        }

        break;
    }

    // ---

    let mut pi = main_pdf;
    let mut pi_sum = main_pdf * lhs.m;

    if rhs.m > 0.0 {
        let mut ps = main.sample.pdf(lights, rhs_hit_point, rhs_hit_normal);

        if ps > 0.0 {
            let is_occluded = main.sample.ray(rhs_hit_point).intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
            );

            if is_occluded {
                ps = 0.0;
            }
        }

        pi = if main_nth == 2 { ps } else { pi };
        pi_sum += ps * rhs.m;
    }

    main.normalize_ex(main_pdf, pi, pi_sum);

    // ---

    main.write(output_reservoirs, screen_idx);
}
