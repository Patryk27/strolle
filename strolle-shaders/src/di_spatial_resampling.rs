use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn pick(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d1: TexRgba16,
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    reservoirs: &[DiReservoirData],
    #[spirv(descriptor_set = 1, binding = 4)] buf_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 5)] buf_d1: TexRgba32,
) {
    let global_id = global_id.xy();
    let lhs_pos = resolve_checkerboard_alt(global_id, params.frame.get() / 2);
    let lhs_idx = camera.screen_to_idx(lhs_pos);
    let mut wnoise = WhiteNoise::new(params.seed, lhs_pos);
    let lights = LightsView::new(lights);

    let buf_pos_a = global_id * uvec2(2, 1);
    let buf_pos_b = buf_pos_a + uvec2(1, 0);

    if !camera.contains(lhs_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let lhs_hit = Hit::new(
        camera.ray(lhs_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(lhs_pos),
            prim_gbuffer_d1.read(lhs_pos),
        ]),
    );

    if lhs_hit.is_none() {
        return;
    }

    // ---

    let lhs = DiReservoir::read(reservoirs, lhs_idx);

    // ---

    let mut rhs = DiReservoir::default();
    let mut rhs_nth = 0;
    let mut rhs_idx = 0;
    let mut rhs_hit = Hit::default();

    let max_samples = 8;
    let mut max_radius = 128.0;

    while rhs_nth < max_samples {
        rhs_nth += 1;

        let rhs_pos = camera.contain(
            (lhs_pos.as_vec2() + wnoise.sample_disk() * max_radius).as_ivec2(),
        );

        if rhs_pos == lhs_pos {
            continue;
        }

        rhs_hit = Hit::new(
            camera.ray(rhs_pos),
            GBufferEntry::unpack([
                prim_gbuffer_d0.read(rhs_pos),
                prim_gbuffer_d1.read(rhs_pos),
            ]),
        );

        if rhs_hit.is_none() {
            max_radius = (max_radius * 0.5).max(5.0);
            continue;
        }

        if (rhs_hit.gbuffer.depth - lhs_hit.gbuffer.depth).abs()
            > 0.33 * lhs_hit.gbuffer.depth
        {
            max_radius = (max_radius * 0.5).max(5.0);
            continue;
        }

        if rhs_hit.gbuffer.normal.dot(lhs_hit.gbuffer.normal) < 0.33 {
            max_radius = (max_radius * 0.5).max(5.0);
            continue;
        }

        rhs_idx = camera.screen_to_idx(rhs_pos);
        rhs = DiReservoir::read(reservoirs, rhs_idx);

        if !rhs.is_empty() {
            break;
        }
    }

    // ---

    if rhs.is_empty() {
        unsafe {
            buf_d1.write(buf_pos_a, Vec4::ZERO);
            buf_d1.write(buf_pos_b, Vec4::ZERO);
        }

        return;
    }

    let lhs_rhs_pdf = lhs.sample.pdf(lights, rhs_hit);
    let rhs_lhs_pdf = rhs.sample.pdf(lights, lhs_hit);

    let ray_a = if lhs_rhs_pdf > 0.0 {
        lhs.sample.ray(rhs_hit.point)
    } else {
        Default::default()
    };

    let ray_b = if rhs_lhs_pdf > 0.0 {
        rhs.sample.ray(lhs_hit.point)
    } else {
        Default::default()
    };

    unsafe {
        buf_d0.write(buf_pos_a, ray_a.origin().extend(ray_a.len()));

        buf_d1.write(
            buf_pos_a,
            Normal::encode(ray_a.dir())
                .extend(f32::from_bits(rhs_idx as u32 + 1))
                .extend(Default::default()),
        );

        buf_d0.write(buf_pos_b, ray_b.origin().extend(ray_b.len()));

        buf_d1.write(
            buf_pos_b,
            Normal::encode(ray_b.dir())
                .extend(lhs_rhs_pdf)
                .extend(rhs_lhs_pdf),
        );
    }
}

#[spirv(compute(threads(8, 8)))]
pub fn trace(
    #[spirv(global_invocation_id)] global_id: UVec3,
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
    #[spirv(descriptor_set = 1, binding = 1)] buf_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] buf_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] buf_d2: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let ray_d0 = buf_d0.read(screen_pos);
    let ray_d1 = buf_d1.read(screen_pos);

    if ray_d1 == Default::default() {
        unsafe {
            buf_d2.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    let ray =
        Ray::new(ray_d0.xyz(), Normal::decode(ray_d1.xy())).with_len(ray_d0.w);

    let is_occluded = ray.intersect(
        local_idx,
        stack,
        triangles,
        bvh,
        materials,
        atlas_tex,
        atlas_sampler,
    );

    let visibility = if is_occluded { 0.0 } else { 1.0 };

    unsafe {
        buf_d2.write(
            screen_pos,
            vec4(visibility, ray_d1.z, ray_d1.w, Default::default()),
        );
    }
}

#[spirv(compute(threads(8, 8)))]
pub fn sample(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    in_reservoirs: &[DiReservoirData],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    out_reservoirs: &mut [DiReservoirData],
    #[spirv(descriptor_set = 0, binding = 3)] buf_d2: TexRgba32,
) {
    let global_id = global_id.xy();
    let lhs_pos = resolve_checkerboard_alt(global_id, params.frame.get() / 2);
    let lhs_idx = camera.screen_to_idx(lhs_pos);
    let mut wnoise = WhiteNoise::new(params.seed, lhs_pos);

    let buf_pos_a = global_id * uvec2(2, 1);
    let buf_pos_b = buf_pos_a + uvec2(1, 0);

    if !camera.contains(lhs_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let d0: Vec4 = buf_d2.read(buf_pos_a);
    let d1: Vec4 = buf_d2.read(buf_pos_b);

    let lhs_rhs_vis = d0.x;
    let rhs_idx = d0.y.to_bits();

    let rhs_lhs_vis = d1.x;
    let lhs_rhs_pdf = d1.y;
    let rhs_lhs_pdf = d1.z;

    // ---

    let lhs = DiReservoir::read(in_reservoirs, lhs_idx);

    if rhs_idx > 0 {
        let rhs = DiReservoir::read(in_reservoirs, rhs_idx as usize - 1);
        let mut main = DiReservoir::default();
        let mut main_pdf = 0.0;

        let mis = Mis {
            lhs_m: lhs.m,
            rhs_m: rhs.m,
            rhs_jacobian: 1.0,
            lhs_lhs_pdf: lhs.sample.pdf,
            lhs_rhs_pdf: lhs_rhs_pdf * lhs_rhs_vis,
            rhs_lhs_pdf: rhs_lhs_pdf * rhs_lhs_vis,
            rhs_rhs_pdf: rhs.sample.pdf,
        }
        .eval();

        if main.update(
            &mut wnoise,
            lhs.sample,
            mis.lhs_mis * mis.lhs_pdf * lhs.w,
        ) {
            main_pdf = mis.lhs_pdf;
        }

        if main.update(
            &mut wnoise,
            rhs.sample,
            mis.rhs_mis * mis.rhs_pdf * rhs.w,
        ) {
            main_pdf = mis.rhs_pdf;
            main.sample.is_occluded = lhs_rhs_vis == 0.0;
        }

        main.m = lhs.m + mis.m;
        main.sample.pdf = main_pdf;
        main.norm_mis(main_pdf);
        main.write(out_reservoirs, lhs_idx);
    } else {
        lhs.write(out_reservoirs, lhs_idx);
    }

    // ---

    let other_idx = camera
        .screen_to_idx(resolve_checkerboard(global_id, params.frame.get() / 2));

    DiReservoir::copy(in_reservoirs, out_reservoirs, other_idx);
}
