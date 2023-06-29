//! This pass applies denoising on the direct lightning; currently we perform
//! just basic color clamping a'la INSIDE.
//!
//! Thanks to:
//!
//! - https://s3.amazonaws.com/arena-attachments/655504/c5c71c5507f0f8bf344252958254fb7d.pdf?1468341463
//!   (Temporal Reprojection Anti-Aliasing in INSIDE)

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
    raw_direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 3)]
    direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 4)]
    prev_direct_colors: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        camera,
        ReprojectionMap::new(reprojection_map),
        raw_direct_colors,
        direct_colors,
        prev_direct_colors,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    raw_direct_colors: TexRgba16f,
    direct_colors: TexRgba16f,
    prev_direct_colors: TexRgba16f,
) {
    let curr_color = raw_direct_colors.read(screen_pos).xyz();
    let reprojection = reprojection_map.get(screen_pos);

    let color = if reprojection.is_some() {
        let mut min_color = Vec3::MAX;
        let mut max_color = Vec3::MIN;
        let mut delta_pos = ivec2(-1, -1);

        loop {
            let sample_pos = screen_pos.as_ivec2() + delta_pos;

            if camera.contains(sample_pos) {
                let color = raw_direct_colors.read(sample_pos).xyz();

                min_color = min_color.min(color);
                max_color = max_color.max(color);
            }

            // ---

            delta_pos.x += 1;

            if delta_pos.x >= 2 {
                delta_pos.x = -1;
                delta_pos.y += 1;

                if delta_pos.y >= 2 {
                    break;
                }
            }
        }

        let prev_color = prev_direct_colors
            .read(reprojection.prev_screen_pos())
            .xyz()
            .clip(min_color, max_color);

        prev_color * 0.75 + curr_color * 0.25
    } else {
        curr_color
    };

    unsafe {
        direct_colors.write(screen_pos, color.extend(Default::default()));
    }
}
