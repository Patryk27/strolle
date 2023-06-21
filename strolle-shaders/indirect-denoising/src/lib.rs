//! This pass applies denoising on the indirect lightning; currently we perform
//! a basic temporal anti-aliasing with color clamping.
//!
//! Thanks to:
//!
//! - https://www.shadertoy.com/view/4tcXD2
//!   (Path tracer + Temporal AA by yvtjp)
//!
//! - https://de45xmedrsdbp.cloudfront.net/Resources/files/TemporalAA_small-59732822.pdf
//!   (High Quality Temporal Supersampling by Brian Karis)

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
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    raw_indirect_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 4)]
    indirect_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 5)]
    past_indirect_colors: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        camera,
        ReprojectionMap::new(reprojection_map),
        SurfaceMap::new(surface_map),
        raw_indirect_colors,
        indirect_colors,
        past_indirect_colors,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    surface_map: SurfaceMap,
    raw_indirect_colors: TexRgba16f,
    indirect_colors: TexRgba16f,
    past_indirect_colors: TexRgba16f,
) {
    let past_color = {
        let reprojection = reprojection_map.get(screen_pos);

        if reprojection.is_valid() {
            past_indirect_colors.read(reprojection.past_screen_pos())
        } else {
            Default::default()
        }
    };

    let mut color = past_color.xyz();
    let mix_rate = past_color.w.min(0.5);
    let in0 = raw_indirect_colors.read(screen_pos).xyz();

    color = (color * color).lerp(in0 * in0, mix_rate);
    color.x = color.x.sqrt();
    color.y = color.y.sqrt();
    color.z = color.z.sqrt();

    // -------------------------------------------------------------------------

    let screen_surface = surface_map.get(screen_pos);

    let neighbour = move |dx: i32, dy: i32| {
        let pos = screen_pos.as_ivec2() + ivec2(dx, dy);

        if !camera.contains(pos) {
            // If our neighbour is outside of the viewport, reject the sample.
            //
            // Note that instead of returning `Vec3::ZERO`, we return the center
            // sample since otherwise we could unnecessarily darken corners or
            // pixels nearby complex surface.
            return in0;
        }

        let pos = pos.as_uvec2();

        if surface_map.get(pos).evaluate_similarity_to(screen_surface) < 0.25 {
            // If our neighbour is too different from us geometrically (e.g. has
            // normal pointing towards a very different direction), reject the
            // sample.
            return in0;
        }

        raw_indirect_colors.read(pos).xyz()
    };

    // TODO optimization opportunity: preload neighbours into shared memory
    let in1 = neighbour(1, 0);
    let in2 = neighbour(-1, 0);
    let in3 = neighbour(0, 1);
    let in4 = neighbour(0, -1);
    let in5 = neighbour(1, 1);
    let in6 = neighbour(-1, 1);
    let in7 = neighbour(1, -1);
    let in8 = neighbour(-1, -1);

    let color = encode_pal_yuv(color);
    let in0 = encode_pal_yuv(in0);
    let in1 = encode_pal_yuv(in1);
    let in2 = encode_pal_yuv(in2);
    let in3 = encode_pal_yuv(in3);
    let in4 = encode_pal_yuv(in4);
    let in5 = encode_pal_yuv(in5);
    let in6 = encode_pal_yuv(in6);
    let in7 = encode_pal_yuv(in7);
    let in8 = encode_pal_yuv(in8);

    // -------------------------------------------------------------------------

    let min = |x: Vec3, y: Vec3| x.min(y);
    let max = |x: Vec3, y: Vec3| x.max(y);
    let mix = |x: Vec3, y: Vec3, v| x.lerp(y, v);

    let mut min_color = min(min(min(in0, in1), min(in2, in3)), in4);
    let mut max_color = max(max(max(in0, in1), max(in2, in3)), in4);

    min_color = mix(
        min_color,
        min(min(min(in5, in6), min(in7, in8)), min_color),
        0.5,
    );

    max_color = mix(
        max_color,
        max(max(max(in5, in6), max(in7, in8)), max_color),
        0.5,
    );

    // -------------------------------------------------------------------------

    let color_before_clamping = color;
    let color = color.clamp(min_color, max_color);

    let clamping = (color - color_before_clamping).length_squared();

    let mix_rate = 1.0 / (1.0 / mix_rate + 1.0);
    let mix_rate = mix_rate + clamping * 4.0;
    let mix_rate = mix_rate.clamp(0.05, 0.5);

    let out = decode_pal_yuv(color).extend(mix_rate);

    unsafe {
        indirect_colors.write(screen_pos, out);
    }
}

fn encode_pal_yuv(rgb: Vec3) -> Vec3 {
    let rgb = rgb.powf(2.0);

    vec3(
        rgb.dot(vec3(0.299, 0.587, 0.114)),
        rgb.dot(vec3(-0.14713, -0.28886, 0.436)),
        rgb.dot(vec3(0.615, -0.51499, -0.10001)),
    )
}

fn decode_pal_yuv(yuv: Vec3) -> Vec3 {
    let rgb = vec3(
        yuv.dot(vec3(1.0, 0., 1.13983)),
        yuv.dot(vec3(1.0, -0.39465, -0.58060)),
        yuv.dot(vec3(1.0, 2.03211, 0.)),
    );

    rgb.powf(1.0 / 2.0)
}
