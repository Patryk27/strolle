#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)] direct_samples: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 5)] direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 6)] prev_direct_colors: TexRgba16f,
) {
    let denoiser = TemporalDenoiser {
        camera,
        reprojection_map: ReprojectionMap::new(reprojection_map),
        surface_map: SurfaceMap::new(surface_map),
        prev_surface_map: SurfaceMap::new(prev_surface_map),
        samples: direct_samples,
        image: direct_colors,
        prev_image: prev_direct_colors,
    };

    denoiser.run(global_id.xy());
}
