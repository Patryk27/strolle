use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &GiPassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)] blue_noise_sobol: &[u32],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    blue_noise_scrambling_tile: &[u32],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    blue_noise_ranking_tile: &[u32],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    rt_rays: &mut [Vec4],
) {
    let global_id = global_id.xy();
    let global_idx = 0; // TODO
    let screen_pos = resolve_checkerboard(global_id, params.frame);
    let mut bnoise = LdsBlueNoise::new(
        blue_noise_sobol,
        blue_noise_scrambling_tile,
        blue_noise_ranking_tile,
        screen_pos,
        params.frame / 4,
        0,
    );
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let prim_hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    let needs_shading = if params.is_diff() {
        prim_hit.gbuffer.needs_diff()
    } else {
        prim_hit.gbuffer.needs_spec()
    };

    if prim_hit.is_none() || !needs_shading {
        unsafe {
            *rt_rays.index_unchecked_mut(2 * global_idx) = Vec4::ZERO;
        }

        return;
    }

    // ---

    let gi_ray_direction = if params.is_diff() {
        bnoise.sample_hemisphere(prim_hit.gbuffer.normal)
    } else {
        let sample =
            SpecularBrdf::new(&prim_hit.gbuffer).sample(&mut wnoise, prim_hit);

        if sample.is_invalid() {
            wnoise.sample_hemisphere(prim_hit.gbuffer.normal)
        } else {
            sample.direction
        }
    };

    let ray = Ray::new(
        prim_hit.point + prim_hit.gbuffer.normal * 0.001,
        gi_ray_direction,
    );

    unsafe {
        *rt_rays.index_unchecked_mut(2 * global_idx) =
            ray.origin().extend(Default::default());

        *rt_rays.index_unchecked_mut(2 * global_idx + 1) =
            ray.direction().extend(Default::default());
    }
}
