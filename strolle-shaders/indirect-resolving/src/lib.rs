//! This pass performs indirect lightning resolving, i.e. it takes the spatial
//! reservoirs (rendered at half-res) and upscales them into a full-res picture.
//!
//! Later this picture is also fed to a dedicated indirect lightning denoiser.

#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &IndirectResolvingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    raw_indirect_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    indirect_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        SurfaceMap::new(surface_map),
        raw_indirect_colors,
        indirect_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    params: &IndirectResolvingPassParams,
    camera: &Camera,
    surface_map: SurfaceMap,
    raw_indirect_colors: TexRgba16f,
    indirect_spatial_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, screen_pos);
    let mut out = Vec4::ZERO;
    let mut sample_idx = 0;

    let screen_surface = surface_map.get(screen_pos);

    while sample_idx < 8 {
        let reservoir_distance = sample_idx as f32;

        // Because we render reservoirs at half-res, if we just upscaled them
        // bilinearly, a single bad reservoir (i.e. too bright) could affect 4+
        // nearby pixels - that happens and it looks just bad.
        //
        // That's why instead of doing basic upscaling, we sample our pixel's
        // neighbourhood and select a few reservoirs at random; later denoising
        // hides any artifacts of that pretty well.
        let reservoir_pos = screen_pos.as_vec2() * 0.5
            + noise.sample_circle() * reservoir_distance;

        let reservoir_pos = reservoir_pos.as_ivec2();

        if reservoir_pos.x < 0 || reservoir_pos.y < 0 {
            sample_idx += 1;
            continue;
        }

        let reservoir_pos = reservoir_pos.as_uvec2();
        let reservoir_screen_pos = upsample(reservoir_pos, params.frame);

        if !camera.contains(reservoir_screen_pos.as_ivec2()) {
            sample_idx += 1;
            continue;
        }

        let reservoir = IndirectReservoir::read(
            indirect_spatial_reservoirs,
            camera.half_screen_to_idx(reservoir_pos),
        );

        let reservoir_radiance = reservoir.sample.radiance * reservoir.w;

        // How useful our candidate-reservoir is; <0.0, 1.0>
        let mut reservoir_weight = 1.0;

        // Since we're looking at our neighbourhood, we might stumble upon a
        // reservoir that's useless for our current pixel - e.g. if that
        // reservoir shades a different object.
        //
        // If that happens, we can't reuse this reservoir's radiance.
        reservoir_weight *= screen_surface
            .evaluate_similarity_to(surface_map.get(reservoir_screen_pos));

        // What's more, we can incorporate here a very useful metric: m_sum.
        //
        // It defines a number of samples this reservoir has seen and so the
        // greater m_sum, the more confident we can be that this reservoir
        // estimates its surrounding correctly.
        //
        // In practice, this reduces variance by assigning weight to less
        // confident reservoirs.
        reservoir_weight *= reservoir.m_sum.powf(0.25).max(1.0).min(5.0);

        out += (reservoir_radiance * reservoir_weight).extend(reservoir_weight);
        sample_idx += 1;
    }

    let out = out.xyz() / out.w.max(1.0);

    unsafe {
        raw_indirect_colors.write(screen_pos, out.extend(1.0));
    }
}
