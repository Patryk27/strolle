#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] direct_colors: TexRgba16,
) {
    let screen_pos = global_id.xy();
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let (_, used_memory) = camera.ray(screen_pos).trace(
        local_idx,
        stack,
        triangles,
        bvh,
        materials,
        atlas_tex,
        atlas_sampler,
    );

    let color = gradient(
        [
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 0.0, 0.0),
        ],
        used_memory as f32 / 8192.0,
    );

    unsafe {
        direct_colors.write(screen_pos, color.extend(1.0));
    }
}

fn gradient<const N: usize>(colors: [Vec3; N], progress: f32) -> Vec3 {
    if progress <= 0.0 {
        return colors[0];
    }

    let step = 1.0 / (N as f32 - 1.0);
    let mut i = 0;

    while i < (N - 1) {
        let min = step * (i as f32);
        let max = step * (i as f32 + 1.0);

        if progress >= min && progress <= max {
            let rhs = (progress - min) / step;
            let lhs = 1.0 - rhs;

            return lhs * colors[i] + rhs * colors[i + 1];
        }

        i += 1;
    }

    colors[N - 1]
}
