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
    prev_camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    velocity_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)]
    reprojection_map: TexRgba32f,
) {
    main_inner(
        global_id.xy(),
        prev_camera,
        SurfaceMap::new(surface_map),
        SurfaceMap::new(prev_surface_map),
        velocity_map,
        ReprojectionMap::new(reprojection_map),
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    prev_camera: &Camera,
    surface_map: SurfaceMap,
    prev_surface_map: SurfaceMap,
    velocity_map: TexRgba32f,
    reprojection_map: ReprojectionMap,
) {
    let mut reprojection = Reprojection::default();
    let screen_surface = surface_map.get(screen_pos);

    let prev_screen_pos =
        screen_pos.as_vec2() - velocity_map.read(screen_pos).xy();

    let check_sample =
        move |dx: i32, dy: i32, reprojection: &mut Reprojection| {
            let prev_screen_pos = prev_screen_pos + vec2(dx as f32, dy as f32);

            if !prev_camera.contains(prev_screen_pos.round().as_ivec2()) {
                return false;
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

            sample_confidence >= 0.5
        };

    if check_sample(0, 0, &mut reprojection) {
        //
    } else {
        check_sample(-1, 0, &mut reprojection);
        check_sample(1, 0, &mut reprojection);
        check_sample(0, -1, &mut reprojection);
        check_sample(0, 1, &mut reprojection);
    }

    reprojection_map.set(screen_pos, &reprojection);
}
