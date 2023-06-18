//! This pass performs camera reprojection, i.e. it finds out where each pixel
//! was located in the previous frame.

#![no_std]

use spirv_std::glam::{vec2, UVec2, UVec3, Vec3Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use strolle_gpu::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    past_camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    direct_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    past_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)]
    reprojection_map: TexRgba32f,
) {
    main_inner(
        global_id.xy(),
        past_camera,
        direct_hits_d0,
        SurfaceMap::new(surface_map),
        SurfaceMap::new(past_surface_map),
        ReprojectionMap::new(reprojection_map),
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    past_camera: &Camera,
    direct_hits_d0: TexRgba32f,
    surface_map: SurfaceMap,
    past_surface_map: SurfaceMap,
    reprojection_map: ReprojectionMap,
) {
    let mut reprojection = Reprojection::default();
    let viewport_size = past_camera.viewport_size().as_vec2();

    // Look at the current world-space hit-point and re-project it using the
    // previous camera's view-projection matrix.
    //
    // tl;dr this gives us a screen-space coordinates of where this hit-point
    //       would be displayed on the past frame camera's viewport
    let past_screen_pos = past_camera.world_to_screen(Hit::deserialize_point(
        direct_hits_d0.read(screen_pos),
    ));

    let screen_surface = surface_map.get(screen_pos);
    let mut sample_delta = vec2(-1.0, -1.0);

    loop {
        let sample_screen_pos = past_screen_pos + sample_delta;

        if sample_screen_pos.x >= 0.0
            && sample_screen_pos.y >= 0.0
            && sample_screen_pos.x < viewport_size.x
            && sample_screen_pos.y < viewport_size.y
        {
            let sample_screen_pos = sample_screen_pos.as_uvec2();

            // TODO optimization opportunity: preload neighbours into shared
            //      memory
            let sample_surface = past_surface_map.get(sample_screen_pos);

            // Compute the difference between normals; this allows us to reject
            // candidates that have wildly different geometric features from our
            // center pixel.
            //
            // Note that we're using `.max(0.0)` on the cosine similarity result
            // so that we clamp negative values down to zero - otherwise
            // opposing normals could turn the difference score positive,
            // yielding spurious results.
            let normal_diff =
                1.0 - sample_surface.normal.dot(screen_surface.normal).max(0.0);

            // Compute the difference between depths; this allows us to reject
            // candidates that are located too far away from our center pixel,
            // which - in turn - reduces bleeding from background into
            // foreground objects.
            let depth_diff =
                (sample_surface.depth - screen_surface.depth).abs();

            // Finally, take into account the screen-space distance; this
            // prevents us from choosing far-away samples when our neighbourhood
            // looks uniform~ish.
            let distance_diff = sample_delta.abs().length_squared() / 10.0;

            let confidence = 1.0 - (normal_diff + depth_diff + distance_diff);

            if confidence > reprojection.confidence {
                reprojection = Reprojection {
                    past_x: sample_screen_pos.x,
                    past_y: sample_screen_pos.y,
                    confidence,
                };
            }
        } else {
            // Reprojected point is located outside of the current camera's
            // viewport (e.g. this will happen to some of the points on the left
            // side of the screen if the camera strafes to right).
        }

        sample_delta.x += 1.0;

        if sample_delta.x >= 2.0 {
            sample_delta.x = -2.0;
            sample_delta.y += 1.0;

            if sample_delta.y >= 2.0 {
                break;
            }
        }
    }

    reprojection_map.set(screen_pos, &reprojection);
}
