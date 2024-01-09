use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 4)] output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);

    if !camera.contains(screen_pos) {
        return;
    }

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    // -------------------------------------------------------------------------

    let res = GiReservoir::read(reservoirs, screen_idx);

    let out = (res.w * res.sample.cosine(&hit) * res.sample.radiance)
        .extend(Default::default());

    unsafe {
        output.write(screen_pos, out);
    }
}
