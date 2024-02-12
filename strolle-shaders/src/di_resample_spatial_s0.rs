use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    input_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    output_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    rt_rays: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    rt_hits: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let lights = LightsView::new(lights);
    let prim_surface_map = SurfaceMap::new(prim_surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

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
    let mut rhs_idx = 0;
    let mut rhs_hit_point = Vec3::ZERO;
    let mut rhs_hit_normal = Vec3::ZERO;

    let max_samples = if params.frame % 3 == 2 { 8 } else { 0 };
    let max_radius = 64.0;

    while rhs_idx < max_samples {
        rhs_idx += 1;

        let rhs_pos = camera.contain(
            (screen_pos.as_vec2() + wnoise.sample_disk() * max_radius)
                .as_ivec2(),
        );

        let rhs_surface = prim_surface_map.get(rhs_pos);

        if rhs_surface.is_sky() {
            continue;
        }

        if (rhs_surface.depth - hit.gbuffer.depth).abs()
            > 0.2 * hit.gbuffer.depth
        {
            continue;
        }

        if rhs_surface.normal.dot(hit.gbuffer.normal) < 0.8 {
            continue;
        }

        rhs =
            DiReservoir::read(input_reservoirs, camera.screen_to_idx(rhs_pos));

        let rhs_pdf = if rhs.is_empty() {
            0.0
        } else {
            rhs.sample.pdf(lights, hit.point, hit.gbuffer.normal)
        };

        if rhs_pdf <= 0.0 {
            rhs.m = 0.0;
            continue;
        }

        rhs_hit_point = camera.ray(rhs_pos).at(rhs_surface.depth)
            + rhs_surface.normal * Hit::NUDGE_OFFSET;

        rhs_hit_normal = rhs_surface.normal;

        if main.merge(&mut wnoise, &rhs, rhs_pdf) {
            main_nth = 2;
            main_pdf = rhs_pdf;
        }

        break;
    }

    // ---

    if rhs.m > 0.0 {
        let ps = main.sample.pdf(lights, rhs_hit_point, rhs_hit_normal);

        let ray = if ps > 0.0 {
            main.sample.ray(rhs_hit_point)
        } else {
            Default::default()
        };

        unsafe {
            *rt_rays.index_unchecked_mut(2 * screen_idx) =
                ray.origin().extend(ray.length());

            *rt_rays.index_unchecked_mut(2 * screen_idx + 1) =
                Normal::encode(ray.direction())
                    .extend(ps)
                    .extend(Default::default());
        }
    } else {
        unsafe {
            *rt_rays.index_unchecked_mut(2 * screen_idx) = Vec4::ZERO;
        }
    }

    unsafe {
        *rt_hits.index_unchecked_mut(2 * screen_idx + 1) =
            vec4(main_pdf, main_nth as f32, lhs.m, rhs.m);
    }

    main.write(output_reservoirs, screen_idx);
}
