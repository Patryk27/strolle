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
    #[spirv(push_constant)] params: &FrameCompositionPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_colors: Tex,
    #[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] direct_primary_hits_d0: Tex,
    #[spirv(descriptor_set = 0, binding = 5)] direct_primary_hits_d2: Tex,
    #[spirv(descriptor_set = 0, binding = 7)] direct_primary_hits_d3: Tex,
    #[spirv(descriptor_set = 0, binding = 9)] direct_secondary_hits_d0: Tex,
    #[spirv(descriptor_set = 0, binding = 11)] direct_secondary_hits_d2: Tex,
    #[spirv(descriptor_set = 0, binding = 13)] indirect_colors: Tex,
    #[spirv(descriptor_set = 0, binding = 15)] surface_map: Tex,
    #[spirv(descriptor_set = 0, binding = 17)] velocity_map: Tex,
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
            let primary_point = Hit::deserialize_point(
                direct_primary_hits_d0.sample(*sampler, texel_xy),
            );

            let primary_base_color =
                direct_primary_hits_d2.sample(*sampler, texel_xy);

            let d3 = direct_primary_hits_d3.sample(*sampler, texel_xy);
            let primary_emissive = d3.xyz();
            let primary_metallic = d3.w;

            let secondary_base_color =
                direct_secondary_hits_d2.sample(*sampler, texel_xy);

            let secondary_point = Hit::deserialize_point(
                direct_secondary_hits_d0.sample(*sampler, texel_xy),
            );

            let direct = direct_colors.sample(*sampler, texel_xy).xyz();
            let indirect = indirect_colors.sample(*sampler, texel_xy).xyz();

            let color = if primary_point == Default::default() {
                // Case 1: We've hit the sky.
                //
                // Arguably, this is the easiest case to compose - the direct
                // resolving pass already handles generating the sky color and
                // puts it into `direct_colors`, which we have right here.
                direct
            } else if primary_metallic > 0.0 {
                // Case 2: We've hit a conductive surface.
                //
                // This requires us to blend primary and secondary surfaces
                // depending on the primary surface's metallicness.
                //
                // Intuitively, if the metallic factor is 1.0, then the primary
                // surface behaves like a mirror.
                //
                // Note that ReSTIR reservoirs here are allocated on the
                // *secondary* surface, so we can't know whether the primary one
                // is shaded or not, so its base color functions kinda as an
                // emissive as well.
                let primary_color = primary_emissive
                    + ((primary_metallic - 1.0) * primary_base_color.xyz());

                // Case 2a/2b: If the secondary hit is sky, don't multiply by
                //             base color (which is black then).
                let secondary_color = if secondary_point == Default::default() {
                    primary_metallic * direct
                } else {
                    primary_metallic
                        * secondary_base_color.xyz()
                        * (direct + indirect)
                };

                primary_color + secondary_color
            } else if primary_base_color.w < 1.0 {
                // Case 3: We've hit a transparent surface.
                //
                // Similarly as with metalics, this requires us to blend primary
                // and secondary surfaces, although this time according to the
                // primary surface's base color.
                //
                // ReSTIR reservoirs here are also allocated on the secondary
                // surface.

                let alpha = primary_base_color.w;

                let primary_color =
                    primary_emissive + (alpha * primary_base_color.xyz());

                // Case 3a/3b: If the secondary hit is sky, don't multiply by
                //             base color (which is black then).
                let secondary_color = if secondary_point == Default::default() {
                    (1.0 - alpha) * direct
                } else {
                    (1.0 - alpha)
                        * secondary_base_color.xyz()
                        * (direct + indirect)
                };

                primary_color + secondary_color
            } else {
                // Case 4: We've hit an opaque surface.

                primary_emissive
                    + primary_base_color.xyz() * (direct + indirect)
            };

            (color, true)
        }

        // CameraMode::DirectLightning
        1 => {
            let base_color =
                direct_primary_hits_d2.sample(*sampler, texel_xy).xyz();

            let direct = direct_colors.sample(*sampler, texel_xy).xyz();

            (base_color * direct, true)
        }

        // CameraMode::DemodulatedDirectLightning
        2 => {
            let direct = direct_colors.sample(*sampler, texel_xy).xyz();

            (direct, true)
        }

        // CameraMode::IndirectLightning
        3 => {
            let base_color =
                direct_primary_hits_d2.sample(*sampler, texel_xy).xyz();

            let indirect = indirect_colors.sample(*sampler, texel_xy).xyz();

            (base_color * indirect, true)
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
