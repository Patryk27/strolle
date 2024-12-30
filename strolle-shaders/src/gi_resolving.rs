use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &GiResolvingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba16,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    in_reservoirs_a: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    in_reservoirs_b: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    out_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6)] diff_output: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 7)] spec_output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);

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

    let res = GiReservoir::read(out_reservoirs, screen_idx);

    let confidence;
    let radiance;

    if hit.is_some() {
        confidence = res.confidence;
        radiance = res.w * res.sample.cosine(hit) * res.sample.radiance;
    } else {
        confidence = 1.0;
        radiance = Vec3::ZERO;
    };

    unsafe {
        let diff_brdf = (1.0 - hit.gbuffer.metallic) / PI;
        let spec_brdf = res.sample.spec_brdf(hit);

        diff_output
            .write(screen_pos, (radiance * diff_brdf).extend(confidence));

        spec_output
            .write(screen_pos, (radiance * spec_brdf).extend(confidence));
    }

    // ---

    if params.source == 0 {
        GiReservoir::copy(in_reservoirs_a, out_reservoirs, screen_idx);
    } else {
        GiReservoir::copy(in_reservoirs_b, out_reservoirs, screen_idx);
    }
}
