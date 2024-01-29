use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &GiDiffSpatialResamplingPassParams,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    input_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    output_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let prim_surface_map = SurfaceMap::new(prim_surface_map);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

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

    if hit.gbuffer.depth == 0.0 {
        GiReservoir::default().write(output_reservoirs, screen_idx);
        return;
    }

    // -------------------------------------------------------------------------

    let mut main = GiReservoir::default();
    let mut main_pdf = 0.0;

    // ---

    let mut lhs_chosen = false;

    let lhs = GiReservoir::read(input_reservoirs, screen_idx);
    let lhs_pdf = lhs.sample.diff_pdf(hit.point, hit.gbuffer.normal);
    let lhs_m = lhs.m;

    if !lhs.is_empty() {
        if main.merge(&mut wnoise, &lhs, lhs_pdf) {
            lhs_chosen = true;
            main_pdf = lhs_pdf;
        }
    }

    // ---

    let mut rhs_m = 0.0;
    let mut rhs_hit_point = Vec3::ZERO;
    let mut rhs_hit_normal = Vec3::ZERO;

    let mut sample_idx = 0;
    let max_samples = if params.frame % 3 == 0 { 8 } else { 0 };
    let mut max_radius = 128.0;

    while sample_idx < max_samples {
        let sample_dist = wnoise.sample_disk() * max_radius;

        let sample_pos =
            camera.contain((screen_pos.as_vec2() + sample_dist).as_ivec2());

        sample_idx += 1;

        if sample_pos == screen_pos {
            continue;
        }

        let sample_surface = prim_surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            max_radius = (max_radius * 0.5).max(5.0);
            continue;
        }

        if (sample_surface.depth - hit.gbuffer.depth).abs()
            > 0.2 * hit.gbuffer.depth
        {
            max_radius = (max_radius * 0.5).max(5.0);
            continue;
        }

        if sample_surface.normal.dot(hit.gbuffer.normal) < 0.8 {
            max_radius = (max_radius * 0.5).max(5.0);
            continue;
        }

        let mut sample = GiReservoir::read(
            input_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        if sample.is_empty() {
            continue;
        }

        sample.clamp_m(16.0);

        let mut sample_pdf =
            sample.sample.diff_pdf(hit.point, hit.gbuffer.normal);

        let sample_jacobian = sample.sample.jacobian(hit.point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if sample_jacobian < 1.0 / 10.0 || sample_jacobian > 10.0 {
            continue;
        }

        let sample_jacobian = sample_jacobian.clamp(1.0 / 3.0, 3.0).sqrt();

        if sample.w * sample_pdf > 0.0 {
            let is_occluded = sample.sample.ray(hit.point).intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
            );

            if is_occluded {
                sample_pdf = 0.0;
            }
        }

        rhs_m = sample.m;
        rhs_hit_point = sample.sample.v1_point;
        rhs_hit_normal = sample_surface.normal;

        if main.merge(&mut wnoise, &sample, sample_pdf * sample_jacobian) {
            lhs_chosen = false;
            main_pdf = sample_pdf;
            main.sample.v1_point = hit.point;
        }

        break;
    }

    // -------------------------------------------------------------------------

    if lhs_chosen {
        if rhs_m > 0.0 {
            if main.sample.diff_pdf(rhs_hit_point, rhs_hit_normal) <= 0.0 {
                rhs_m = 0.0;
            }
        }

        if rhs_m > 0.0 {
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
                rhs_m = 0.0;
            }
        }
    }

    // -------------------------------------------------------------------------

    main.m = lhs_m + rhs_m;
    main.normalize(main_pdf);
    main.write(output_reservoirs, screen_idx);
}
