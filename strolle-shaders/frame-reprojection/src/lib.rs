//! This pass performs camera reprojection, i.e. it finds out where each pixel
//! was located in the previous frame.

#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 2)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)] velocity_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 5)] reprojection_map: TexRgba32f,
) {
    let screen_pos = global_id.xy();
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    // -------------------------------------------------------------------------

    let mut reprojection = Reprojection::default();

    // If camera's mode has changed, force the reprojection to be none in order
    // to reset temporal algorithms (e.g. ReSTIR reservoirs) - this comes handy
    // for debugging
    if camera.mode() != prev_camera.mode() {
        reprojection_map.set(screen_pos, &reprojection);
        return;
    }

    let surface = surface_map.get(screen_pos);

    // We don't need reprojection for the sky
    if surface.depth == 0.0 {
        reprojection_map.set(screen_pos, &reprojection);
        return;
    }

    // -------------------------------------------------------------------------

    let prev_screen_pos =
        screen_pos.as_vec2() - velocity_map.read(screen_pos).xy();

    if prev_camera.contains(prev_screen_pos.round().as_ivec2()) {
        let prev_surface =
            prev_surface_map.get(prev_screen_pos.round().as_uvec2());

        let confidence = prev_surface.evaluate_similarity_to(&surface);

        if confidence > 0.0 {
            reprojection = Reprojection {
                prev_x: prev_screen_pos.x,
                prev_y: prev_screen_pos.y,
                confidence,
                validity: 0,
            };
        }
    }

    // -------------------------------------------------------------------------

    if reprojection.is_some() {
        let check_validity = move |sample_pos| {
            if !camera.contains(sample_pos) {
                return false;
            }

            prev_surface_map
                .get(sample_pos.as_uvec2())
                .evaluate_similarity_to(&surface)
                >= 0.5
        };

        let [p00, p10, p01, p11] = BilinearFilter::reprojection_coords(
            reprojection.prev_x,
            reprojection.prev_y,
        );

        if check_validity(p00) {
            reprojection.validity |= 0b0001;
        }

        if check_validity(p10) {
            reprojection.validity |= 0b0010;
        }

        if check_validity(p01) {
            reprojection.validity |= 0b0100;
        }

        if check_validity(p11) {
            reprojection.validity |= 0b1000;
        }
    }

    // -------------------------------------------------------------------------

    reprojection_map.set(screen_pos, &reprojection);
}
