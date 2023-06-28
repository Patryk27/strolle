//! This pass performs camera reprojection, i.e. it finds out where each pixel
//! was located in the previous frame.

#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    past_camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    past_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    velocity_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)]
    reprojection_map: TexRgba32f,
) {
    main_inner(
        global_id.xy(),
        past_camera,
        SurfaceMap::new(surface_map),
        SurfaceMap::new(past_surface_map),
        velocity_map,
        ReprojectionMap::new(reprojection_map),
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    past_camera: &Camera,
    surface_map: SurfaceMap,
    past_surface_map: SurfaceMap,
    velocity_map: TexRgba32f,
    reprojection_map: ReprojectionMap,
) {
    let mut reprojection = Reprojection::default();
    let screen_surface = surface_map.get(screen_pos);

    let velocity = velocity_map.read(screen_pos).xy();
    let past_screen_pos = screen_pos.as_ivec2() - velocity.round().as_ivec2();

    let mut sample_delta = ivec2(-1, -1);

    loop {
        let sample_screen_pos = past_screen_pos + sample_delta;

        if past_camera.contains(sample_screen_pos) {
            let sample_screen_pos = sample_screen_pos.as_uvec2();

            // TODO optimization opportunity: preload neighbours into shared
            //      memory
            let sample_surface = past_surface_map.get(sample_screen_pos);

            // Check if the pixel we're looking at shades the same object; if
            // it's not, there's no point in reusing its sample later.
            let instance_diff = if sample_surface.instance_uuid
                == screen_surface.instance_uuid
            {
                0.0
            } else {
                1.0
            };

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
                if sample_surface.depth == 0.0 || screen_surface.depth == 0.0 {
                    // Edge case: don't reproject sky; this can cause indirect
                    // lightning to bleed
                    1.0
                } else {
                    (sample_surface.depth - screen_surface.depth).abs()
                };

            // Finally, take into account the screen-space distance; this
            // prevents us from choosing far-away samples when our neighbourhood
            // looks uniform~ish.
            let distance_diff =
                sample_delta.abs().as_vec2().length_squared() * 0.1;

            let confidence = 1.0
                - (instance_diff + normal_diff + depth_diff + distance_diff);

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

        sample_delta.x += 1;

        if sample_delta.x >= 2 {
            sample_delta.x = -2;
            sample_delta.y += 1;

            if sample_delta.y >= 2 {
                break;
            }
        }
    }

    reprojection_map.set(screen_pos, &reprojection);
}
