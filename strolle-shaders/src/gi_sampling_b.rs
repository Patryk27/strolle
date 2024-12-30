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
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
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
    #[spirv(descriptor_set = 1, binding = 6)] prim_gbuffer_d1: TexRgba16,
    #[spirv(descriptor_set = 1, binding = 7)] gi_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 8)] gi_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 9)] gi_d2: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 10, storage_buffer)]
    prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 11, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
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

    let prim_hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    if prim_hit.is_none() {
        return;
    }

    let d0 = gi_d0.read(global_id);
    let d1 = gi_d1.read(global_id);
    let d2 = gi_d2.read(global_id);

    let mut wnoise;
    let gi_hit;
    let gi_ray_pdf;

    if params.frame.is_gi_tracing() {
        wnoise = WhiteNoise::new(params.seed, screen_pos);

        gi_hit = Hit::new(
            Ray::new(prim_hit.point, d0.xyz()),
            GBufferEntry::unpack([d1, d2]),
        );

        gi_ray_pdf = d0.w;
    } else {
        let res = GiReservoir::read(prev_reservoirs, screen_idx);

        if res.is_empty() {
            return;
        } else {
            wnoise = WhiteNoise::from_state(res.sample.rng);

            gi_hit = Hit::new(
                Ray::new(res.sample.v1_point, d0.xyz()),
                GBufferEntry::unpack([d1, d2]),
            );

            gi_ray_pdf = 1.0;
        }
    }

    // -------------------------------------------------------------------------

    let rng = wnoise.state();

    let light_id;
    let light_pdf;
    let light_rad;
    let mut light_dir = Vec3::ZERO;

    if gi_hit.is_none() {
        light_id = LightId::sky();
        light_pdf = 1.0;
        light_rad = atmosphere.sample(world.sun_dir(), gi_hit.dir);
    } else {
        let atmosphere_pdf = if world.sun_altitude <= -1.0 {
            0.0
        } else {
            0.25
        };

        if world.light_count == 0 || wnoise.sample() < atmosphere_pdf {
            light_id = LightId::sky();
            light_pdf = atmosphere_pdf;
            light_dir = wnoise.sample_hemisphere(gi_hit.gbuffer.normal);

            light_rad = atmosphere.sample(world.sun_dir(), light_dir)
                * gi_hit.gbuffer.normal.dot(light_dir);
        } else {
            let res =
                EphemeralReservoir::build(&mut wnoise, lights, *world, gi_hit);

            if res.w > 0.0 {
                // For simplicity, we assume an unmodulated diffuse BRDF here
                // and modulate it later, a few lines below
                let light_diff_brdf = Vec3::ONE;
                let light_spec_brdf = res.sample.light_rad.spec_brdf;

                light_id = res.sample.light_id;
                light_pdf = (1.0 / res.w) * (1.0 - atmosphere_pdf);

                light_rad = res.sample.light_rad.radiance
                    * (light_diff_brdf + light_spec_brdf);
            } else {
                light_id = LightId::new(0);
                light_pdf = 1.0;
                light_rad = Vec3::ZERO;
            }
        }
    }

    // -------------------------------------------------------------------------

    let mut radiance = if light_pdf > 0.0 {
        let light_vis = if gi_hit.is_some() {
            let ray = if light_id == LightId::sky() {
                Ray::new(gi_hit.point, light_dir)
            } else {
                lights.get(light_id).ray_wnoise(&mut wnoise, gi_hit.point)
            };

            let is_occluded = ray.intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
            );

            if is_occluded {
                0.0
            } else {
                1.0
            }
        } else {
            // If we hit nothing, our indirect-ray must be pointing towards
            // the sky - no point retracing it, then
            1.0
        };

        light_rad * light_vis / light_pdf
    } else {
        // If the probability of hitting our light is non-positive, there are
        // probably no lights present on the scene - in this case zeroing-out
        // the radiance is best we can do
        Vec3::ZERO
    };

    if gi_hit.is_some() {
        radiance *= gi_hit.gbuffer.base_color.xyz() / PI;
        radiance += gi_hit.gbuffer.emissive * 60.0; //Emissive strength should be parameterised somehow.
    }

    // -------------------------------------------------------------------------

    let mut res = GiReservoir::default();

    if gi_ray_pdf > 0.0 {
        let v1_point = prim_hit.point;
        let v2_point;
        let v2_normal;

        if gi_hit.is_some() {
            v2_point = gi_hit.point;
            v2_normal = gi_hit.gbuffer.normal;
        } else {
            v2_point = v1_point + gi_hit.dir * World::SUN_DISTANCE;
            v2_normal = -gi_hit.dir;
        }

        res.sample = GiSample {
            pdf: 0.0,
            rng,
            radiance,
            v1_point,
            v2_point,
            v2_normal,
        };

        res.m = 1.0;
        res.w = 1.0 / gi_ray_pdf;
        res.sample.pdf = res.sample.pdf(prim_hit);
    }

    res.write(curr_reservoirs, screen_idx);
}
