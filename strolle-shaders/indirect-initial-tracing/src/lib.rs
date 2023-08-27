#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
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
    #[spirv(descriptor_set = 1, binding = 1)] direct_hits: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4)] indirect_rays: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 5)] indirect_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)] indirect_gbuffer_d1: TexRgba32f,
) {
    let screen_pos = global_id.xy();
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    let direct_hit = Hit::from_direct(
        camera.ray(screen_pos),
        direct_hits.read(screen_pos).xyz(),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let indirect_ray_direction;
    let indirect_hit;

    if direct_hit.is_none() {
        indirect_ray_direction = Vec3::ZERO;
        indirect_hit = TriangleHit::none();
    } else {
        let expects_diffuse_sample =
            IndirectReservoir::expects_diffuse_sample(screen_pos, params.frame);

        indirect_ray_direction = if expects_diffuse_sample {
            wnoise.sample_hemisphere(direct_hit.gbuffer.normal)
        } else {
            let sample = SpecularBrdf::new(&direct_hit.gbuffer)
                .sample(&mut wnoise, direct_hit);

            if sample.is_invalid() {
                wnoise.sample_hemisphere(direct_hit.gbuffer.normal)
            } else {
                sample.direction
            }
        };

        let ray = Ray::new(direct_hit.point, indirect_ray_direction);

        (indirect_hit, _) = ray.trace(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
        );
    };

    let indirect_gbuffer = if indirect_hit.is_some() {
        // TODO reloading material here shouldn't be necessary because we
        //      already load materials during ray-traversal
        let mut indirect_material = materials.get(indirect_hit.material_id);

        indirect_material.adjust_for_indirect();

        GBufferEntry {
            base_color: indirect_material.base_color(
                atlas_tex,
                atlas_sampler,
                indirect_hit.uv,
            ),
            normal: indirect_hit.normal,
            metallic: indirect_material.metallic,
            emissive: indirect_material.emissive(
                atlas_tex,
                atlas_sampler,
                indirect_hit.uv,
            ),
            roughness: indirect_material.roughness,
            reflectance: indirect_material.reflectance,
            depth: direct_hit.point.distance(indirect_hit.point),
        }
    } else {
        Default::default()
    };

    let [d0, d1] = indirect_gbuffer.pack();

    unsafe {
        indirect_rays.write(
            screen_pos,
            indirect_ray_direction.extend(Default::default()),
        );

        indirect_gbuffer_d0.write(screen_pos, d0);
        indirect_gbuffer_d1.write(screen_pos, d1);
    }
}
