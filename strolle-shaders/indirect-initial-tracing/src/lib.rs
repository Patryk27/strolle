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
    bvh: &[BvhNode],
    #[spirv(descriptor_set = 1, binding = 0)]
    direct_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 1)]
    direct_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)]
    indirect_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)]
    indirect_hits_d1: TexRgba32f,
) {
    main_inner(
        global_id.xy(),
        local_idx,
        params,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        direct_hits_d0,
        direct_hits_d1,
        indirect_hits_d0,
        indirect_hits_d1,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    local_idx: u32,
    params: &IndirectInitialTracingPassParams,
    stack: BvhStack,
    triangles: TrianglesView,
    bvh: BvhView,
    direct_hits_d0: TexRgba32f,
    direct_hits_d1: TexRgba32f,
    indirect_hits_d0: TexRgba32f,
    indirect_hits_d1: TexRgba32f,
) {
    let mut noise = Noise::new(params.seed, global_id);
    let screen_pos = upsample(global_id, params.frame);

    let direct_hit = Hit::deserialize(
        direct_hits_d0.read(screen_pos),
        direct_hits_d1.read(screen_pos),
    );

    let indirect_hit = if direct_hit.is_none() {
        Hit::none()
    } else {
        let ray = Ray::new(
            direct_hit.point,
            noise.sample_hemisphere(direct_hit.normal),
        );

        ray.trace_nearest(local_idx, triangles, bvh, stack).0
    };

    let [d0, d1] = indirect_hit.serialize();

    unsafe {
        indirect_hits_d0.write(global_id, d0);
        indirect_hits_d1.write(global_id, d1);
    }
}
