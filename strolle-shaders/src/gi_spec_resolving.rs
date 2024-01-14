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
    #[spirv(descriptor_set = 0, binding = 4)] samples: TexRgba32,
) {
    let screen_pos = global_id.xy();

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let res = GiReservoir::read(reservoirs, camera.screen_to_idx(screen_pos));

    let out = if res.is_empty() {
        Vec3::ZERO
    } else {
        let hit = Hit::new(
            camera.ray(screen_pos),
            GBufferEntry::unpack([
                prim_gbuffer_d0.read(screen_pos),
                prim_gbuffer_d1.read(screen_pos),
            ]),
        );

        let radiance = res.sample.radiance * res.w;
        let cosine = res.sample.cosine(&hit);
        let brdf = res.sample.spec_brdf(&hit);

        if brdf.probability > 0.0 {
            (radiance * cosine * brdf.radiance) / brdf.probability
        } else {
            Vec3::ZERO
        }
    };

    unsafe {
        samples.write(screen_pos, out.extend(Default::default()));
    }
}
