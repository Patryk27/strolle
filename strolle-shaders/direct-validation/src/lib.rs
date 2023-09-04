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
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let reprojection = reprojection_map.get(screen_pos);

    let reservoir = if reprojection.is_some() {
        let mut reservoir = DirectReservoir::read(
            direct_prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        if debug::DIRECT_VALIDATION_ENABLED && reservoir.w > 0.0 {
            let light_to_hit = hit.point - reservoir.sample.light_position;

            let ray = Ray::new(
                reservoir.sample.light_position,
                light_to_hit.normalize(),
            );

            if ray.intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
                light_to_hit.length(),
            ) {
                reservoir.w = 0.0;
            }
        }

        reservoir
    } else {
        Default::default()
    };

    reservoir.write(direct_curr_reservoirs, screen_idx);
}
