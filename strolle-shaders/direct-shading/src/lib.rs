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
    #[spirv(descriptor_set = 1, binding = 5)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    direct_initial_samples: &mut [Vec4],
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

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    let (light_id, light_pdf, light_radiance) =
        EphemeralReservoir::sample::<false>(
            &mut wnoise,
            &atmosphere,
            world,
            &lights,
            hit,
        );

    let (light_position, light_radiance) = if light_pdf > 0.0 {
        let (light_position, light_visibility) = if hit.is_some() {
            let light = if light_id.is_sun() {
                Light::sky(world.sun_position())
            } else {
                lights.get(light_id)
            };

            light.visibility(
                local_idx,
                stack,
                triangles,
                bvh,
                materials,
                atlas_tex,
                atlas_sampler,
                &mut wnoise,
                hit.point,
            )
        } else {
            // If we hit nothing, our ray must be pointing towards the sky - no
            // point in tracing it, then
            (world.sun_position(), 1.0)
        };

        (
            light_position,
            light_radiance * light_visibility / light_pdf,
        )
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    };

    unsafe {
        *direct_initial_samples.get_unchecked_mut(2 * screen_idx + 0) =
            light_radiance.extend(f32::from_bits(hit.is_some() as u32));

        *direct_initial_samples.get_unchecked_mut(2 * screen_idx + 1) =
            light_position.extend(f32::from_bits(light_id.get()));
    }
}
