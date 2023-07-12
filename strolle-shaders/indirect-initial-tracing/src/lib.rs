#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(local_invocation_index)]
    local_idx: u32,
    #[spirv(push_constant)]
    params: &IndirectInitialTracingPassParams,
    #[spirv(workgroup)]
    stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3)]
    atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)]
    atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0)]
    direct_primary_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 1)]
    direct_primary_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)]
    indirect_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)]
    indirect_hits_d1: TexRgba32f,
) {
    main_inner(
        global_id.xy(),
        local_idx,
        stack,
        WhiteNoise::new(params.seed, global_id.xy()),
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        MaterialsView::new(materials),
        atlas_tex,
        atlas_sampler,
        direct_primary_hits_d0,
        direct_primary_hits_d1,
        indirect_hits_d0,
        indirect_hits_d1,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    local_idx: u32,
    stack: BvhStack,
    mut wnoise: WhiteNoise,
    triangles: TrianglesView,
    bvh: BvhView,
    materials: MaterialsView,
    atlas_tex: Tex,
    atlas_sampler: &Sampler,
    direct_primary_hits_d0: TexRgba32f,
    direct_primary_hits_d1: TexRgba32f,
    indirect_hits_d0: TexRgba32f,
    indirect_hits_d1: TexRgba32f,
) {
    let direct_hit = Hit::deserialize(
        direct_primary_hits_d0.read(screen_pos),
        direct_primary_hits_d1.read(screen_pos),
    );

    let indirect_hit = if direct_hit.is_none() {
        Hit::none()
    } else {
        let ray = Ray::new(
            direct_hit.point,
            wnoise.sample_hemisphere(direct_hit.normal),
        );

        ray.trace(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
        )
        .0
    };

    let [d0, d1] = indirect_hit.serialize();

    unsafe {
        indirect_hits_d0.write(screen_pos, d0);
        indirect_hits_d1.write(screen_pos, d1);
    }
}
