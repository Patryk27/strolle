use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    rt_rays: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    rt_hits: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.is_none() {
        return;
    }

    // ---

    let mut main = DiReservoir::read(reservoirs, screen_idx);

    let d0 = unsafe { *rt_rays.index_unchecked(2 * screen_idx + 1) };
    let d1 = unsafe { *rt_hits.index_unchecked(2 * screen_idx + 1) };
    let main_pdf = d1.x;
    let main_nth = d1.y as u32;
    let lhs_m = d1.z;
    let rhs_m = d1.w;

    let mut pi = main_pdf;
    let mut pi_sum = main_pdf * lhs_m;

    if rhs_m > 0.0 {
        let mut ps = d0.z;

        let is_occluded =
            unsafe { rt_hits.index_unchecked(2 * screen_idx).x.to_bits() == 1 };

        if is_occluded {
            ps = 0.0;
        }

        pi = if main_nth == 2 { ps } else { pi };
        pi_sum += ps * rhs_m;
    }

    main.normalize_ex(main_pdf, pi, pi_sum);
    main.write(reservoirs, screen_idx);

    // ---

    let ray = if main.is_empty() {
        Default::default()
    } else {
        main.sample.ray(hit.point)
    };

    unsafe {
        *rt_rays.index_unchecked_mut(2 * screen_idx) =
            ray.origin().extend(ray.length());

        *rt_rays.index_unchecked_mut(2 * screen_idx + 1) =
            Normal::encode(ray.direction())
                .extend(Default::default())
                .extend(Default::default());
    }
}
