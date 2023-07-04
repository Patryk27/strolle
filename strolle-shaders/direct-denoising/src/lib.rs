#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    raw_direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 4)]
    direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 5)]
    prev_direct_colors: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        camera,
        ReprojectionMap::new(reprojection_map),
        SurfaceMap::new(surface_map),
        raw_direct_colors,
        direct_colors,
        prev_direct_colors,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    surface_map: SurfaceMap,
    raw_direct_colors: TexRgba16f,
    direct_colors: TexRgba16f,
    prev_direct_colors: TexRgba16f,
) {
    let denoiser = TemporalDenoiser {
        camera,
        reprojection_map,
        surface_map,
        input: raw_direct_colors,
        output: direct_colors,
        prev_output: prev_direct_colors,
    };

    denoiser.run(screen_pos);
}
