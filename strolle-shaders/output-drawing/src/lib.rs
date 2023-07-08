#![no_std]

use strolle_gpu::prelude::*;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_idx: i32,
    #[spirv(position)] output: &mut Vec4,
) {
    fn full_screen_triangle(vert_idx: i32) -> Vec4 {
        let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
        let pos = 2.0 * uv - Vec2::ONE;

        pos.extend(0.0).extend(1.0)
    }

    *output = full_screen_triangle(vert_idx);
}

#[spirv(fragment)]
#[allow(clippy::too_many_arguments)]
pub fn main_fs(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(push_constant)] params: &OutputDrawingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_colors: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] direct_hits_d0: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 5)] direct_hits_d2: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 7)] direct_hits_d3: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 9)] indirect_colors: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 11)] surface_map: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 13)] velocity_map: &Image!(2D, type=f32, sampled),
    frag_color: &mut Vec4,
) {
    let texel_xy = {
        let viewport_pos = camera.viewport_position();
        let viewport_size = camera.viewport_size().as_vec2();

        let x = (pos.x - viewport_pos.x) / (viewport_size.x);
        let y = (pos.y - viewport_pos.y) / (viewport_size.y);

        vec2(x, y)
    };

    let (color, apply_color_adjustments) = match params.camera_mode {
        // CameraMode::Image
        0 => {
            let point = Hit::deserialize_point(
                direct_hits_d0.sample(*sampler, texel_xy),
            );

            let albedo = direct_hits_d2.sample(*sampler, texel_xy).xyz();
            let direct = direct_colors.sample(*sampler, texel_xy).xyz();
            let indirect = indirect_colors.sample(*sampler, texel_xy).xyz();
            let emissive = direct_hits_d3.sample(*sampler, texel_xy).xyz();

            let color = if point == Default::default() {
                // If we hit nothing, the `direct` color will contain sky but
                // `albedo` is going to be all black, so we need to handle it
                // separately and not multply by albedo then:
                direct
            } else {
                // TODO multiplying direct by albedo is not correct here
                //      (same case as in `LightContribution::with_albedo()`)
                albedo * (direct + indirect) + emissive
            };

            (color, true)
        }

        // CameraMode::DirectLightning
        1 => {
            let albedo = direct_hits_d2.sample(*sampler, texel_xy).xyz();
            let direct = direct_colors.sample(*sampler, texel_xy).xyz();

            (albedo * direct, true)
        }

        // CameraMode::DemodulatedDirectLightning
        2 => {
            let direct = direct_colors.sample(*sampler, texel_xy).xyz();

            (direct, true)
        }

        // CameraMode::IndirectLightning
        3 => {
            let albedo = direct_hits_d2.sample(*sampler, texel_xy).xyz();
            let indirect = indirect_colors.sample(*sampler, texel_xy).xyz();

            (albedo * indirect, true)
        }

        // CameraMode::DemodulatedIndirectLightning
        4 => {
            let indirect = indirect_colors.sample(*sampler, texel_xy).xyz();

            (indirect, true)
        }

        // CameraMode::NormalMap
        5 => {
            let surface = surface_map.sample(*sampler, texel_xy).xyz();

            let normal = if surface.z == 0.0 {
                Default::default()
            } else {
                Vec3::splat(0.5) + Normal::decode(surface.xy()) * 0.5
            };

            (normal, false)
        }

        // CameraMode::BvhHeatmap
        6 => {
            let heatmap = direct_colors.sample(*sampler, texel_xy).xyz();

            (heatmap, false)
        }

        // CameraMode::VelocityMap
        7 => {
            let velocity = velocity_map
                .sample(*sampler, texel_xy)
                .xy()
                .abs()
                .extend(0.0);

            (velocity, false)
        }

        _ => Default::default(),
    };

    *frag_color = if apply_color_adjustments {
        let color = apply_debanding(pos.xy(), color);
        let color = apply_tone_mapping(color);

        color.extend(1.0)
    } else {
        color.extend(1.0)
    };
}

/// Applies screen-space debanding using a simple dither.
///
/// Thanks to:
/// https://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf (slide 49)
fn apply_debanding(pos: Vec2, color: Vec3) -> Vec3 {
    fn screen_space_dither(pos: Vec2) -> Vec3 {
        let dither = Vec3::splat(vec2(171.0, 231.0).dot(pos));
        let dither = (dither / vec3(103.0, 71.0, 97.0)).fract();

        (dither - 0.5) / 255.0
    }

    let color = color.powf(1.0 / 2.2);
    let color = color + screen_space_dither(pos);

    color.powf(2.2)
}

/// Applies Reinhard tone mapping.
fn apply_tone_mapping(color: Vec3) -> Vec3 {
    fn luminance(color: Vec3) -> f32 {
        color.dot(vec3(0.2126, 0.7152, 0.0722))
    }

    fn change_luminance(color: Vec3, l_out: f32) -> Vec3 {
        let l_in = luminance(color);

        color * (l_out / l_in)
    }

    let l_old = luminance(color);
    let l_new = l_old / (1.0 + l_old);

    change_luminance(color, l_new)
}
