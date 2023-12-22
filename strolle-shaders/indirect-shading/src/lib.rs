#![no_std]

use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &IndirectPassParams,
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
    #[spirv(descriptor_set = 1, binding = 5)] direct_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)] direct_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 7)] indirect_rays: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 8)] indirect_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 9)] indirect_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 10, storage_buffer)]
    indirect_samples: &mut [Vec4],
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

    let direct_hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let indirect_ray_direction = indirect_rays.read(screen_pos);

    // Empty ray direction means that we've either didn't hit anything or that
    // the surface we've hit is not compatible with our current pass kind (e.g.
    // we're tracing specular, but the surface is purely diffuse).
    //
    // Either way, in this case we've got nothing to do.
    if indirect_ray_direction == Vec4::ZERO {
        unsafe {
            *indirect_samples.index_unchecked_mut(3 * screen_idx) =
                Default::default();
        }

        return;
    }

    let indirect_hit = Hit::new(
        Ray::new(direct_hit.point, indirect_ray_direction.xyz()),
        GBufferEntry::unpack([
            indirect_gbuffer_d0.read(screen_pos),
            indirect_gbuffer_d1.read(screen_pos),
        ]),
    );

    // -------------------------------------------------------------------------

    let light_id;
    let light_pdf;
    let light_radiance;

    let mut light_dir = Vec3::ZERO;

    if indirect_hit.is_none() {
        light_id = LightId::sky();
        light_pdf = 1.0;

        light_radiance =
            atmosphere.sky(world.sun_direction(), indirect_hit.direction);
    } else {
        let atmosphere_pdf = if world.sun_altitude <= -1.0 {
            0.0
        } else {
            0.25
        };

        if wnoise.sample() < atmosphere_pdf {
            light_id = LightId::sky();
            light_pdf = atmosphere_pdf;
            light_dir = wnoise.sample_hemisphere(indirect_hit.gbuffer.normal);

            light_radiance = atmosphere.sky(world.sun_direction(), light_dir)
                * indirect_hit.gbuffer.normal.dot(light_dir);
        } else {
            let mut res = EphemeralReservoir::default();
            let mut light_idx = 0;

            while light_idx < world.light_count {
                let light_id = LightId::new(light_idx);

                let light_radiance =
                    lights.get(light_id).contribution(indirect_hit);

                let sample = EphemeralReservoirSample {
                    light_id,
                    light_radiance,
                };

                res.update(&mut wnoise, sample, sample.pdf());
                light_idx += 1;
            }

            if res.w > 0.0 {
                light_id = res.sample.light_id;
                light_pdf = res.sample.pdf() / res.w / (1.0 - atmosphere_pdf);
                light_radiance = res.sample.light_radiance;
            } else {
                light_id = LightId::new(0);
                light_pdf = 0.0;
                light_radiance = Vec3::ZERO;
            }
        }
    }

    // ---

    let mut radiance = if light_pdf > 0.0 {
        let light_visibility = if indirect_hit.is_some() {
            let (ray, ray_dist) = if light_id == LightId::sky() {
                (Ray::new(indirect_hit.point, light_dir), f32::MAX)
            } else {
                lights.get(light_id).ray(&mut wnoise, indirect_hit.point)
            };

            let is_occluded = ray.intersect(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
                ray_dist,
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

        light_radiance * light_visibility / light_pdf
    } else {
        // If the probability of hitting our light is non-positive, there are
        // probably no lights present on the scene - in this case zeroing-out
        // the radiance is best we can do
        Vec3::ZERO
    };

    radiance += indirect_hit.gbuffer.emissive;

    // -------------------------------------------------------------------------

    let indirect_normal;
    let indirect_point;

    if indirect_hit.is_some() {
        indirect_normal = Normal::encode(indirect_hit.gbuffer.normal);
        indirect_point = indirect_hit.point;
    } else {
        indirect_normal = Normal::encode(-indirect_hit.direction);
        indirect_point = indirect_hit.direction * World::SUN_DISTANCE;
    }

    unsafe {
        *indirect_samples.index_unchecked_mut(3 * screen_idx) =
            direct_hit.point.extend(f32::from_bits(1));

        *indirect_samples.index_unchecked_mut(3 * screen_idx + 1) =
            radiance.extend(indirect_normal.x);

        *indirect_samples.index_unchecked_mut(3 * screen_idx + 2) =
            indirect_point.extend(indirect_normal.y);
    }
}
