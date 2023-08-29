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
    #[spirv(descriptor_set = 1, binding = 7)] indirect_rays: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 8)] indirect_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 9)] indirect_gbuffer_d1: TexRgba32f,
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

    // -------------------------------------------------------------------------

    let direct_hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    if direct_hit.is_none() {
        unsafe {
            *indirect_samples.get_unchecked_mut(3 * screen_idx + 0) =
                Default::default();
        }

        return;
    }

    let indirect_hit = Hit::new(
        Ray::new(direct_hit.point, indirect_rays.read(screen_pos).xyz()),
        GBufferEntry::unpack([
            indirect_gbuffer_d0.read(screen_pos),
            indirect_gbuffer_d1.read(screen_pos),
        ]),
    );

    // -------------------------------------------------------------------------

    let (light_id, light_pdf, light_radiance) =
        EphemeralReservoir::sample::<true>(
            &mut wnoise,
            &atmosphere,
            world,
            &lights,
            indirect_hit,
        );

    let mut radiance = if light_pdf > 0.0 {
        let light_visibility = if indirect_hit.is_some() {
            let light = if light_id.is_sky() {
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
                indirect_hit.point,
            )
        } else {
            // If we hit nothing, our indirect-ray must be pointing towards
            // the sky - no point re-tracing it, then
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
        *indirect_samples.get_unchecked_mut(3 * screen_idx + 0) =
            direct_hit.point.extend(f32::from_bits(1));

        *indirect_samples.get_unchecked_mut(3 * screen_idx + 1) =
            radiance.extend(indirect_normal.x);

        *indirect_samples.get_unchecked_mut(3 * screen_idx + 2) =
            indirect_point.extend(indirect_normal.y);
    }
}
