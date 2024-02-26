use strolle_gpu::prelude::*;

#[spirv(vertex)]
pub fn vs(
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
pub fn fs(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(push_constant)] params: &FrameCompositionPassParams,
    #[spirv(descriptor_set = 0, binding = 0)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] di_diff_colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] di_spec_colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4)] gi_diff_colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 5)] gi_spec_colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 6)] ref_colors: TexRgba32,
    frag_color: &mut Vec4,
) {
    let screen_pos = pos.xy().as_uvec2();

    let gbuffer = GBufferEntry::unpack([
        prim_gbuffer_d0.read(screen_pos),
        prim_gbuffer_d1.read(screen_pos),
    ]);

    let color = match params.camera_mode {
        // CameraMode::Image
        0 => {
            let di_diff = di_diff_colors.read(screen_pos).xyz();
            let di_spec = di_spec_colors.read(screen_pos).xyz();
            let gi_diff = gi_diff_colors.read(screen_pos).xyz();
            let gi_spec = gi_spec_colors.read(screen_pos).xyz();

            if gbuffer.is_some() {
                gbuffer.emissive
                    + (di_diff + gi_diff) * gbuffer.base_color.xyz()
                    + di_spec
                    + gi_spec
            } else {
                di_diff
            }
        }

        // CameraMode::DiDiffuse
        1 => di_diff_colors.read(screen_pos).xyz(),

        // CameraMode::DiSpecular
        2 => di_spec_colors.read(screen_pos).xyz(),

        // CameraMode::GiDiffuse
        3 => gi_diff_colors.read(screen_pos).xyz(),

        // CameraMode::GiSpecular
        4 => gi_spec_colors.read(screen_pos).xyz(),

        // CameraMode::BvhHeatmap
        5 => ref_colors.read(screen_pos).xyz(),

        // CameraMode::Reference
        6 => {
            let color = ref_colors.read(screen_pos);

            color.xyz() / color.w
        }

        _ => Default::default(),
    };

    *frag_color = color.extend(1.0);
}
