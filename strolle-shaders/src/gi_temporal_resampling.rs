use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] curr_camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 2)] curr_prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] curr_prim_gbuffer_d1: TexRgba16,
    #[spirv(descriptor_set = 0, binding = 4)] prev_prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 5)] prev_prim_gbuffer_d1: TexRgba16,
    #[spirv(descriptor_set = 0, binding = 6)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 7, storage_buffer)]
    prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 8, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
) {
    let lhs_pos = global_id.xy();
    let lhs_idx = curr_camera.screen_to_idx(lhs_pos);
    let mut wnoise = WhiteNoise::new(params.seed, global_id.xy());
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !curr_camera.contains(lhs_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let lhs_hit = Hit::new(
        curr_camera.ray(lhs_pos),
        GBufferEntry::unpack([
            curr_prim_gbuffer_d0.read(lhs_pos),
            curr_prim_gbuffer_d1.read(lhs_pos),
        ]),
    );

    if lhs_hit.is_none() {
        GiReservoir::default().write(curr_reservoirs, lhs_idx);
        return;
    }

    // ---

    let got_sample = if params.frame.is_gi_tracing() {
        params.frame.get() % 2 == 0
            && got_checkerboard_at(lhs_pos, params.frame.get() / 2)
    } else {
        got_checkerboard_at(lhs_pos, params.frame.get())
    };

    let lhs = if got_sample {
        GiReservoir::read(curr_reservoirs, lhs_idx)
    } else {
        GiReservoir::default()
    };

    // ---

    let mut rhs = GiReservoir::default();
    let mut rhs_hit = Hit::default();

    let reprojection = reprojection_map.get(lhs_pos);

    if reprojection.is_some() {
        rhs = GiReservoir::read(prev_reservoirs, lhs_idx);
        rhs.confidence = 1.0;
        rhs.clamp_m(128.0);

        if params.frame.is_gi_validation()
            && !lhs.is_empty()
            && !rhs.is_empty()
            && rhs.sample.exists()
        {
            if lhs.sample.radiance.distance(rhs.sample.radiance) > 0.33 {
                rhs.confidence = 0.0;
            }

            // Resampling stale reservoirs is tricky, because we can't really
            // calculate proper MIS weights here - that would require taking the
            // current sample and tracing it in the previous frame's BVH etc.,
            // which is infeasible even for direct lighting.
            //
            // ReSTIR GI paper suggests discarding stale reservoirs by setting
            // their M to 0, which is also biased (as it overestimates newly
            // discovered samples), but it's managable when one uses separate
            // temporal and spatial reservoirs, since it doesn't cause the bias
            // to propagate to other temporal reservoirs.
            //
            // Somewhat unfortunately, because we use one set of reservoirs,
            // without distinguishing between temporals and spatials, resetting
            // a reservoir causes an unmanagable bias that quickly spreads to
            // hundreds of nearby pixels and cannot be remedied.
            //
            // So instead of resetting the reservoir, we simply update the
            // sample that's within it - it's also biased (and not as reactive
            // as resetting would be), but in a managable way.
            rhs.sample.radiance = lhs.sample.radiance;
            rhs.sample.v2_point = lhs.sample.v2_point;
            rhs.sample.v2_normal = lhs.sample.v2_normal;
        }

        if !rhs.is_empty() {
            let rhs_pos = reprojection.prev_pos_round();

            rhs_hit = Hit::new(
                prev_camera.ray(rhs_pos),
                GBufferEntry::unpack([
                    prev_prim_gbuffer_d0.read(rhs_pos),
                    prev_prim_gbuffer_d1.read(rhs_pos),
                ]),
            );
        }
    }

    // ---

    let mut main = GiReservoir::default();
    let mut main_pdf = 0.0;

    if params.frame.is_gi_tracing() {
        let mis = Mis::gi_temporal(lhs, lhs_hit, rhs, rhs_hit).eval();

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
        }

        main.m = lhs.m + mis.m;
        main.confidence = 1.0;
        main.norm_mis(main_pdf);
    } else {
        if main.merge(&mut wnoise, &rhs, rhs.sample.pdf) {
            main_pdf = rhs.sample.pdf;
        }

        main.confidence = rhs.confidence;
        main.norm_avg(main_pdf);
    }

    main.sample.pdf = main_pdf;
    main.sample.v1_point = lhs_hit.point;
    main.clamp_w(5.0);
    main.write(curr_reservoirs, lhs_idx);
}
