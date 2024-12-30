use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba16,
    #[spirv(descriptor_set = 0, binding = 3)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    in_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    out_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

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

    let reprojection = reprojection_map.get(screen_pos);

    let mut res = if reprojection.is_some() {
        GiReservoir::read(
            in_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        )
    } else {
        GiReservoir::default()
    };

    res.confidence = 1.0;
    res.sample.v1_point = hit.point;
    res.write(out_reservoirs, screen_idx);
}
