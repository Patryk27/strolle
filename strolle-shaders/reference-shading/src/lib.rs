#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &ReferencePassParams,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 4)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 5)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 6, uniform)] world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 2)]
    atmosphere_transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 3)]
    atmosphere_transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 4)] atmosphere_sky_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 5)]
    atmosphere_sky_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    reference_rays: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    reference_hits: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 8)] reference_colors: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let lights = LightsView::new(lights);
    let materials = MaterialsView::new(materials);
    let atmosphere = Atmosphere::new(
        atmosphere_transmittance_lut_tex,
        atmosphere_transmittance_lut_sampler,
        atmosphere_sky_lut_tex,
        atmosphere_sky_lut_sampler,
    );

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    if params.depth == u8::MAX as u32 {
        let previous_color = if camera.is_eq(prev_camera) {
            reference_colors.read(screen_pos)
        } else {
            Default::default()
        };

        let current_color = reference_rays[3 * screen_idx + 2].xyz();

        unsafe {
            reference_colors
                .write(screen_pos, previous_color + current_color.extend(1.0));
        }

        return;
    }

    // -------------------------------------------------------------------------

    let ray;
    let mut color;
    let mut throughput;

    if params.depth == 0 {
        ray = camera.ray(screen_pos);
        color = Vec3::ZERO;
        throughput = Vec3::ONE;
    } else {
        let d0 = reference_rays[3 * screen_idx];
        let d1 = reference_rays[3 * screen_idx + 1];
        let d2 = reference_rays[3 * screen_idx + 2];

        ray = Ray::new(d0.xyz(), d1.xyz());
        color = d2.xyz();
        throughput = vec3(d0.w, d1.w, d2.w);
    }

    let hit = {
        let t_hit = TriangleHit::unpack([
            reference_hits[2 * screen_idx],
            reference_hits[2 * screen_idx + 1],
        ]);

        if t_hit.is_none() {
            reference_rays[3 * screen_idx] = Default::default();
            reference_rays[3 * screen_idx + 1] = Default::default();

            color += throughput
                * atmosphere.sky(world.sun_direction(), ray.direction());

            reference_rays[3 * screen_idx + 2] =
                color.extend(Default::default());

            return;
        }

        let mut material = materials.get(t_hit.material_id);

        if params.depth > 0 {
            material.adjust_for_indirect();
        }

        Hit {
            point: t_hit.point + t_hit.normal * Hit::NUDGE_OFFSET,
            origin: ray.origin(),
            direction: ray.direction(),
            gbuffer: GBufferEntry {
                base_color: material.base_color(
                    atlas_tex,
                    atlas_sampler,
                    t_hit.uv,
                ),
                normal: t_hit.normal,
                metallic: material.metallic,
                emissive: material.emissive(atlas_tex, atlas_sampler, t_hit.uv),
                roughness: material.roughness,
                reflectance: material.reflectance,
                depth: 0.0,
            },
        }
    };

    // -------------------------------------------------------------------------

    color += throughput * hit.gbuffer.emissive;

    if world.light_count > 0 {
        let light_id = wnoise.sample_int() % world.light_count;
        let light_pdf = 1.0 / (world.light_count as f32);

        let light = lights.get(LightId::new(light_id));
        let light_radiance = light.contribution(hit);

        let light_visibility = light.visibility(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            &mut wnoise,
            hit.point,
        );

        color += throughput * light_visibility * light_radiance / light_pdf;
    }

    // -------------------------------------------------------------------------

    let reflected_sample = LayeredBrdf::sample(&mut wnoise, hit);

    if reflected_sample.is_invalid() {
        reference_rays[3 * screen_idx] = Default::default();
        reference_rays[3 * screen_idx + 1] = Default::default();
        return;
    }

    let reflected_ray = Ray::new(hit.point, reflected_sample.direction);

    throughput *= reflected_sample.direction.dot(hit.gbuffer.normal);
    throughput *= reflected_sample.throughput;

    // -------------------------------------------------------------------------

    reference_rays[3 * screen_idx] =
        reflected_ray.origin().extend(throughput.x);

    reference_rays[3 * screen_idx + 1] =
        reflected_ray.direction().extend(throughput.y);

    reference_rays[3 * screen_idx + 2] = color.extend(throughput.z);
}
