use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] curr_camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 2)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] curr_prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4)] curr_prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 5)] prev_prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)] prev_prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 8, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
) {
    let lhs_pos = global_id.xy();
    let lhs_idx = curr_camera.screen_to_idx(lhs_pos);
    let mut wnoise = WhiteNoise::new(params.seed, lhs_pos);
    let lights = LightsView::new(lights);
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
        return;
    }

    // ---

    let mut lhs = DiReservoir::read(curr_reservoirs, lhs_idx);

    if !lhs.is_empty() {
        lhs.sample.pdf = lhs.sample.pdf(lights, lhs_hit);
    }

    // ---

    let mut rhs = DiReservoir::default();
    let mut rhs_hit = Hit::default();
    let mut rhs_killed = false;

    let reprojection = reprojection_map.get(lhs_pos);

    if reprojection.is_some() {
        let rhs_pos = reprojection.prev_pos_round();

        rhs = DiReservoir::read(
            prev_reservoirs,
            curr_camera.screen_to_idx(rhs_pos),
        );

        rhs.clamp_m(64.0);

        if !rhs.is_empty() {
            let rhs_light = lights.get(rhs.sample.light_id);

            if rhs_light.is_slot_killed() {
                rhs.w = 0.0;
                rhs_killed = true;
            } else if rhs_light.is_slot_remapped() {
                rhs.sample.light_id = rhs_light.slot_remapped_to();
            }

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

    let mut main = DiReservoir::default();
    let mut main_pdf = 0.0;

    let mis =
        Mis::di_temporal(lights, lhs, lhs_hit, rhs, rhs_hit, rhs_killed).eval();

    if main.update(&mut wnoise, lhs.sample, mis.lhs_mis * mis.lhs_pdf * lhs.w) {
        main_pdf = mis.lhs_pdf;
    }

    if main.update(&mut wnoise, rhs.sample, mis.rhs_mis * mis.rhs_pdf * rhs.w) {
        main_pdf = mis.rhs_pdf;
    }

    main.m = lhs.m + mis.m;
    main.sample.pdf = main_pdf;
    main.sample.confidence = if rhs_killed { 0.0 } else { 1.0 };
    main.norm_mis(main_pdf);
    main.write(curr_reservoirs, lhs_idx);
}
