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
    #[spirv(descriptor_set = 1, binding = 1, storage_buffer)]
    rt_rays: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 2, storage_buffer)]
    rt_hits: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let ray_d0 = unsafe { *rt_rays.index_unchecked(2 * screen_idx) };
    let ray_d1 = unsafe { *rt_rays.index_unchecked(2 * screen_idx + 1) };

    if ray_d0 == Default::default() {
        unsafe {
            rt_hits.index_unchecked_mut(2 * screen_idx).x = f32::from_bits(1);
        }

        return;
    }

    let ray = Ray::new(ray_d0.xyz(), Normal::decode(ray_d1.xy()))
        .with_length(ray_d0.w);

    let is_occluded = ray.intersect(
        local_idx,
        stack,
        triangles,
        bvh,
        materials,
        atlas_tex,
        atlas_sampler,
    );

    unsafe {
        rt_hits.index_unchecked_mut(2 * screen_idx).x =
            f32::from_bits(is_occluded as u32);
    }
}
