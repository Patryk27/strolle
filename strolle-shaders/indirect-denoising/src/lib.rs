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
    raw_indirect_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 4)]
    indirect_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 5)]
    prev_indirect_colors: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        camera,
        ReprojectionMap::new(reprojection_map),
        SurfaceMap::new(surface_map),
        raw_indirect_colors,
        indirect_colors,
        prev_indirect_colors,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    surface_map: SurfaceMap,
    raw_indirect_colors: TexRgba16f,
    indirect_colors: TexRgba16f,
    prev_indirect_colors: TexRgba16f,
) {
    let denoiser = TemporalDenoiser {
        camera,
        reprojection_map,
        surface_map,
        input: raw_indirect_colors,
        output: indirect_colors,
        prev_output: prev_indirect_colors,
    };

    denoiser.run(screen_pos);
}
