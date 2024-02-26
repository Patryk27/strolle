use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 4)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 5)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 6, uniform)] world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    atmosphere_transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 2)]
    atmosphere_transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)] atmosphere_sky_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 4)]
    atmosphere_sky_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    next_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 8, storage_buffer)]
    prev_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 9)] diff_output: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 10)] spec_output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let lights = LightsView::new(lights);
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

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    let mut res =
        DiReservoir::read(next_reservoirs, camera.screen_to_idx(screen_pos));

    let confidence;
    let radiance;

    if hit.is_some() {
        let is_occluded = res.sample.ray(hit.point).intersect(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
        );

        confidence = if res.sample.is_occluded == is_occluded {
            res.sample.confidence
        } else {
            0.0
        };

        res.sample.confidence = 1.0;
        res.sample.is_occluded = is_occluded;

        radiance = if res.sample.is_occluded {
            LightRadiance::default()
        } else {
            lights.get(res.sample.light_id).radiance(hit) * res.w
        };
    } else {
        confidence = 1.0;

        radiance = LightRadiance {
            radiance: atmosphere.sample(world.sun_dir(), hit.dir),
            diff_brdf: Vec3::ONE,
            spec_brdf: Vec3::ZERO,
        };
    };

    unsafe {
        let diff_brdf = (1.0 - hit.gbuffer.metallic) / PI;
        let spec_brdf = radiance.spec_brdf;

        diff_output.write(
            screen_pos,
            (radiance.radiance * diff_brdf).extend(confidence),
        );

        spec_output.write(
            screen_pos,
            (radiance.radiance * spec_brdf).extend(confidence),
        );
    }

    res.write(prev_reservoirs, screen_idx);
}
