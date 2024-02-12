use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &GiPassParams,
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
    #[spirv(descriptor_set = 1, binding = 6)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 7)] gi_rays: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 8)] gi_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 9)] gi_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 10, storage_buffer)]
    gi_samples: &mut [Vec4],
) {
    // let global_id = global_id.xy();
    // let screen_pos = resolve_checkerboard(global_id, params.frame);
    // let screen_idx = camera.screen_to_idx(screen_pos);
    // let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    // let triangles = TrianglesView::new(triangles);
    // let bvh = BvhView::new(bvh);
    // let lights = LightsView::new(lights);
    // let materials = MaterialsView::new(materials);
    // let atmosphere = Atmosphere::new(
    //     atmosphere_transmittance_lut_tex,
    //     atmosphere_transmittance_lut_sampler,
    //     atmosphere_sky_lut_tex,
    //     atmosphere_sky_lut_sampler,
    // );

    // if !camera.contains(screen_pos) {
    //     return;
    // }

    // // -------------------------------------------------------------------------

    // let prim_hit = Hit::new(
    //     camera.ray(screen_pos),
    //     GBufferEntry::unpack([
    //         prim_gbuffer_d0.read(screen_pos),
    //         prim_gbuffer_d1.read(screen_pos),
    //     ]),
    // );

    // let gi_ray_direction = gi_rays.read(global_id);

    // // Empty ray direction means that we've either didn't hit anything or that
    // // the surface we've hit is not compatible with our current pass kind (e.g.
    // // we're tracing specular, but the surface is purely diffuse).
    // //
    // // Either way, in this case we've got nothing to do.
    // if gi_ray_direction == Vec4::ZERO {
    //     unsafe {
    //         *gi_samples.index_unchecked_mut(3 * screen_idx) =
    //             Default::default();
    //     }

    //     return;
    // }

    // let gi_hit = Hit::new(
    //     Ray::new(prim_hit.point, gi_ray_direction.xyz()),
    //     GBufferEntry::unpack([
    //         gi_gbuffer_d0.read(global_id),
    //         gi_gbuffer_d1.read(global_id),
    //     ]),
    // );

    // // -------------------------------------------------------------------------

    // let light_id;
    // let light_pdf;
    // let light_radiance;

    // let mut light_dir = Vec3::ZERO;

    // if gi_hit.is_none() {
    //     light_id = LightId::sky();
    //     light_pdf = 1.0;

    //     light_radiance =
    //         atmosphere.sample(world.sun_direction(), gi_hit.direction, 32.0);
    // } else {
    //     let atmosphere_pdf = if world.sun_altitude <= -1.0 {
    //         0.0
    //     } else {
    //         0.25
    //     };

    //     if world.light_count == 0 || wnoise.sample() < atmosphere_pdf {
    //         light_id = LightId::sky();
    //         light_pdf = atmosphere_pdf;
    //         light_dir = wnoise.sample_hemisphere(gi_hit.gbuffer.normal);

    //         light_radiance =
    //             atmosphere.sample(world.sun_direction(), light_dir, 32.0)
    //                 * gi_hit.gbuffer.normal.dot(light_dir);
    //     } else {
    //         let mut res = EphemeralReservoir::default();
    //         let mut res_pdf = 0.0;

    //         let sample_ipdf = world.light_count as f32;
    //         let mut sample_idx = 0;

    //         while sample_idx < 16 {
    //             let light_id =
    //                 LightId::new(wnoise.sample_int() % world.light_count);

    //             let light_radiance = lights
    //                 .get(light_id)
    //                 .radiance(gi_hit.point, gi_hit.gbuffer.normal);

    //             let sample = EphemeralSample {
    //                 light_id,
    //                 light_radiance,
    //             };

    //             let sample_pdf = sample.pdf();

    //             if res.update(&mut wnoise, sample, sample_pdf * sample_ipdf) {
    //                 res_pdf = sample_pdf;
    //             }

    //             sample_idx += 1;
    //         }

    //         res.normalize(res_pdf);

    //         if res.w > 0.0 {
    //             light_id = res.sample.light_id;
    //             light_pdf = (1.0 / res.w) * (1.0 - atmosphere_pdf);
    //             light_radiance = res.sample.light_radiance;
    //         } else {
    //             light_id = LightId::new(0);
    //             light_pdf = 1.0;
    //             light_radiance = Vec3::ZERO;
    //         }
    //     }
    // }

    // // ---

    // let mut radiance = if light_pdf > 0.0 {
    //     let light_visibility = if gi_hit.is_some() {
    //         let ray = if light_id == LightId::sky() {
    //             Ray::new(gi_hit.point, light_dir)
    //         } else {
    //             lights.get(light_id).ray_wnoise(&mut wnoise, gi_hit.point)
    //         };

    //         let is_occluded = ray.intersect(
    //             local_idx,
    //             stack,
    //             triangles,
    //             bvh,
    //             materials,
    //             atlas_tex,
    //             atlas_sampler,
    //         );

    //         if is_occluded {
    //             0.0
    //         } else {
    //             1.0
    //         }
    //     } else {
    //         // If we hit nothing, our indirect-ray must be pointing towards
    //         // the sky - no point retracing it, then
    //         1.0
    //     };

    //     light_radiance * light_visibility / light_pdf
    // } else {
    //     // If the probability of hitting our light is non-positive, there are
    //     // probably no lights present on the scene - in this case zeroing-out
    //     // the radiance is best we can do
    //     Vec3::ZERO
    // };

    // if gi_hit.is_some() {
    //     radiance *= DiffuseBrdf::new(&gi_hit.gbuffer).evaluate().radiance;
    // }

    // radiance += gi_hit.gbuffer.emissive;

    // // -------------------------------------------------------------------------

    // let gi_normal;
    // let gi_point;

    // if gi_hit.is_some() {
    //     gi_normal = Normal::encode(gi_hit.gbuffer.normal);
    //     gi_point = gi_hit.point;
    // } else {
    //     gi_normal = Normal::encode(-gi_hit.direction);
    //     gi_point = gi_hit.direction * World::SUN_DISTANCE;
    // }

    // unsafe {
    //     *gi_samples.index_unchecked_mut(3 * screen_idx) =
    //         prim_hit.point.extend(f32::from_bits(1));

    //     *gi_samples.index_unchecked_mut(3 * screen_idx + 1) =
    //         radiance.extend(gi_normal.x);

    //     *gi_samples.index_unchecked_mut(3 * screen_idx + 2) =
    //         gi_point.extend(gi_normal.y);
    // }
}
