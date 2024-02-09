use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &GiPassParams,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)] blue_noise_sobol: &[u32],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    blue_noise_scrambling_tile: &[u32],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    blue_noise_ranking_tile: &[u32],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 6)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 7)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] gi_rays: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4)] gi_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 5)] gi_gbuffer_d1: TexRgba32,
) {
    let global_id = global_id.xy();
    let screen_pos = resolve_checkerboard(global_id, params.frame);
    let mut bnoise = LdsBlueNoise::new(
        blue_noise_sobol,
        blue_noise_scrambling_tile,
        blue_noise_ranking_tile,
        screen_pos,
        params.frame / 4,
        0,
    );
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let prim_hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    let needs_shading = if params.is_diff() {
        prim_hit.gbuffer.needs_diff()
    } else {
        prim_hit.gbuffer.needs_spec()
    };

    if prim_hit.is_none() || !needs_shading {
        unsafe {
            gi_rays.write(global_id, Vec4::ZERO);
        }

        return;
    }

    // ---

    let gi_ray_direction = if params.is_diff() {
        bnoise.sample_hemisphere(prim_hit.gbuffer.normal)
    } else {
        let sample =
            SpecularBrdf::new(&prim_hit.gbuffer).sample(&mut wnoise, prim_hit);

        if sample.is_invalid() {
            wnoise.sample_hemisphere(prim_hit.gbuffer.normal)
        } else {
            sample.direction
        }
    };

    let ray = Ray::new(
        prim_hit.point + prim_hit.gbuffer.normal * 0.001,
        gi_ray_direction,
    );

    let (gi_hit, _) = ray.trace(
        local_idx,
        stack,
        triangles,
        bvh,
        materials,
        atlas_tex,
        atlas_sampler,
    );

    // ---

    let gi_gbuffer = if gi_hit.is_some() {
        let mut gi_material = materials.get(gi_hit.material_id);

        gi_material.regularize();

        GBufferEntry {
            base_color: gi_material.base_color(
                atlas_tex,
                atlas_sampler,
                gi_hit.uv,
            ),
            normal: gi_hit.normal,
            metallic: gi_material.metallic,
            emissive: gi_material.emissive(atlas_tex, atlas_sampler, gi_hit.uv),
            roughness: gi_material.roughness,
            reflectance: gi_material.reflectance,
            depth: prim_hit.point.distance(gi_hit.point),
        }
    } else {
        Default::default()
    };

    let [d0, d1] = gi_gbuffer.pack();

    unsafe {
        gi_rays.write(global_id, gi_ray_direction.extend(Default::default()));
        gi_gbuffer_d0.write(global_id, d0);
        gi_gbuffer_d1.write(global_id, d1);
    }
}
