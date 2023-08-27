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

    let velocity = {
        let mut velocity =
            velocity_map.read(screen_pos).xy().extend(surface.depth);

        // TODO
        if false {
            let mut delta = ivec2(-1, -1);

            loop {
                if delta != ivec2(0, 0) {
                    let sample_pos = screen_pos.as_ivec2() + delta;

                    if camera.contains(sample_pos) {
                        let sample_pos = sample_pos.as_uvec2();
                        let sample_depth = surface_map.get(sample_pos).depth;

                        if sample_depth < velocity.z {
                            velocity = velocity_map
                                .read(sample_pos)
                                .xy()
                                .extend(sample_depth);
                        }
                    }
                }

                // ---

                delta.x += 1;

                if delta.x > 1 {
                    delta.x = -1;
                    delta.y += 1;

                    if delta.y > 1 {
                        break;
                    }
                }
            }
        }

        velocity
    };

    // ---

    let prev_screen_pos = screen_pos.as_vec2() - velocity.xy();

    let check_neighbour = move |reprojection: &mut Reprojection,
                                dx: f32,
                                dy: f32| {
        let sample_pos = prev_screen_pos + vec2(dx, dy);

        if !prev_camera.contains(sample_pos.round().as_ivec2()) {
            return;
        }

        let sample_surface =
            prev_surface_map.get(sample_pos.round().as_uvec2());

        let sample_confidence = sample_surface.evaluate_similarity_to(&surface);

        if sample_confidence > reprojection.confidence {
            *reprojection = Reprojection {
                prev_x: sample_pos.x,
                prev_y: sample_pos.y,
                confidence: sample_confidence,
                validity: 0,
            };
        }
    };

    // TODO consider checking other neighbours as well (?)
    check_neighbour(&mut reprojection, 0.0, 0.0);

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

        let [p00, p10, p01, p11] = BilinearFilter::find_reprojection_coords(
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
