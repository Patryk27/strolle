#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] direct_samples: TexRgba16,
    #[spirv(descriptor_set = 0, binding = 3)] direct_colors: TexRgba16,
) {
    let screen_pos = global_id.xy();
    let surface_map = SurfaceMap::new(surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);

    if surface.is_sky() {
        unsafe {
            direct_colors.write(screen_pos, direct_samples.read(screen_pos));
        }

        return;
    }

    // TODO
    unsafe {
        direct_colors.write(screen_pos, direct_samples.read(screen_pos));
    }
}
