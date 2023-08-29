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
    #[spirv(descriptor_set = 0, binding = 0, uniform)] _camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)]
    indirect_diffuse_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 5)]
    indirect_specular_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 6)] reference_colors: TexRgba32f,
    frag_color: &mut Vec4,
) {
    let screen_pos = pos.xy().as_uvec2();

    let (color, apply_color_adjustments) = match params.camera_mode {
        // CameraMode::Image
        0 => {
            let gbuffer = GBufferEntry::unpack([
                direct_gbuffer_d0.read(screen_pos),
                direct_gbuffer_d1.read(screen_pos),
            ]);

            let color = if gbuffer.is_some() {
                let direct = direct_colors.read(screen_pos).xyz();
                let direct = direct * (1.0 - gbuffer.metallic);

                let indirect_diffuse =
                    indirect_diffuse_colors.read(screen_pos).xyz();

                let indirect_specular =
                    indirect_specular_colors.read(screen_pos).xyz();

                gbuffer.emissive
                    + gbuffer.base_color.xyz() * (direct + indirect_diffuse)
                    + indirect_specular
            } else {
                direct_colors.read(screen_pos).xyz()
            };

            (color, true)
        }

        // CameraMode::DirectLightning
        1 => (direct_colors.read(screen_pos).xyz(), true),

        // CameraMode::IndirectDiffuseLightning
        2 => (indirect_diffuse_colors.read(screen_pos).xyz(), true),

        // CameraMode::IndirectSpecularLightning
        3 => (indirect_specular_colors.read(screen_pos).xyz(), true),

        // CameraMode::BvhHeatmap
        4 => (direct_colors.read(screen_pos).xyz(), false),

        // CameraMode::Reference
        5 => {
            let color = reference_colors.read(screen_pos);
            let color = color.xyz() / color.w;

            (color, true)
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
    color.with_luminance(color.luminance() / (1.0 + color.luminance()))
}
