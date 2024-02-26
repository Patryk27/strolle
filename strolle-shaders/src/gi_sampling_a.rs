use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] gi_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4)] gi_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 5)] gi_d2: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    reservoirs: &[Vec4],
) {
    let global_id = global_id.xy();

    let screen_pos = if params.frame.is_gi_tracing() {
        resolve_checkerboard(global_id, params.frame.get() / 2)
    } else {
        resolve_checkerboard(global_id, params.frame.get())
    };

    let screen_idx = camera.screen_to_idx(screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let gi_ray;
    let gi_ray_pdf;

    if params.frame.is_gi_tracing() {
        let mut wnoise = WhiteNoise::new(params.seed, screen_pos);

        let hit = Hit::new(
            camera.ray(screen_pos),
            GBufferEntry::unpack([
                prim_gbuffer_d0.read(screen_pos),
                prim_gbuffer_d1.read(screen_pos),
            ]),
        );

        if hit.is_none() {
            return;
        } else {
            let sample =
                LayeredBrdf::new(hit.gbuffer).sample(&mut wnoise, -hit.dir);

            gi_ray = Ray::new(hit.point, sample.dir);
            gi_ray_pdf = sample.pdf;
        }
    } else {
        let res = GiReservoir::read(reservoirs, screen_idx);

        if res.is_empty() {
            return;
        }

        gi_ray =
            Ray::new(res.sample.v1_point, res.sample.dir(res.sample.v1_point));

        gi_ray_pdf = 1.0;
    };

    let (gi_hit, _) = gi_ray.trace(
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
            depth: gi_ray.origin().distance(gi_hit.point),
        }
    } else {
        Default::default()
    };

    let d0 = gi_ray.dir().extend(gi_ray_pdf);
    let [d1, d2] = gi_gbuffer.pack();

    unsafe {
        gi_d0.write(global_id, d0);
        gi_d1.write(global_id, d1);
        gi_d2.write(global_id, d2);
    }
}
