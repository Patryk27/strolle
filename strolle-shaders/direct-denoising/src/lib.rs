#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)]
    raw_direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 1)]
    direct_colors: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        raw_direct_colors,
        direct_colors,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    raw_direct_colors: TexRgba16f,
    direct_colors: TexRgba16f,
) {
    // TODO implement denoising; maybe SVGF? (i.e. edge-avoiding À-Trous,
    //      separately on demodulated diffuse and specular channels)

    unsafe {
        direct_colors.write(screen_pos, raw_direct_colors.read(screen_pos));
    }
}
