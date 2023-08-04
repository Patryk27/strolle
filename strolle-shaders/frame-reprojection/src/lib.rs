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

    // If camera's mode has changed, force the reprojection to be none in order
    // to reset temporal algorithms (e.g. ReSTIR reservoirs) - this comes handy
    // for debugging
    if camera.mode() != prev_camera.mode() {
        reprojection_map.set(screen_pos, &Default::default());
        return;
    }

    let screen_surface = surface_map.get(screen_pos);

    // We don't need reprojection for sky
    if screen_surface.depth == 0.0 {
        reprojection_map.set(screen_pos, &Default::default());
        return;
    }

    let mut reprojection = Reprojection::default();

    let prev_screen_pos =
        screen_pos.as_vec2() - velocity_map.read(screen_pos).xy();

    let check_neighbour =
        move |reprojection: &mut Reprojection, dx: f32, dy: f32| {
            let prev_screen_pos = prev_screen_pos + vec2(dx, dy);

            if !prev_camera.contains(prev_screen_pos.round().as_ivec2()) {
                return;
            }

            let sample_surface =
                prev_surface_map.get(prev_screen_pos.round().as_uvec2());

            let sample_confidence =
                sample_surface.evaluate_similarity_to(&screen_surface);

            if sample_confidence > reprojection.confidence {
                *reprojection = Reprojection {
                    prev_x: prev_screen_pos.x,
                    prev_y: prev_screen_pos.y,
                    confidence: sample_confidence,
                };
            }
        };

    check_neighbour(&mut reprojection, 0.0, 0.0);

    // TODO
    //
    // if reprojection.confidence < 0.5 {
    //     check_neighbour(&mut reprojection, -1.0, 0.0);
    //     check_neighbour(&mut reprojection, 1.0, 0.0);
    //     check_neighbour(&mut reprojection, 0.0, -1.0);
    //     check_neighbour(&mut reprojection, 0.0, 1.0);
    // }

    reprojection_map.set(screen_pos, &reprojection);
}
