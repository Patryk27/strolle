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
    // Step 1:
    //
    // Similarly as for direct lightning, let's start by selecting the best
    // light-candidate, judging lights by their *unshadowed* contribution.
    //
    // This algorithm follows a similar logic as direct initial shading, so
    // comments were skipped for brevity.

    let mut reservoir = DirectReservoir::default();

    if indirect_hit.is_some() {
        let mut light_idx = 0;

        while light_idx < world.light_count {
            let light_id = LightId::new(light_idx);

            let light_contribution =
                lights.get(light_id).contribution(indirect_hit).sum();

            let sample = DirectReservoirSample {
                light_id,
                light_contribution,
                light_pdf: 1.0,
                hit_point: Default::default(),
            };

            reservoir.add(&mut wnoise, sample, sample.p_hat());
            light_idx += 1;
        }
    }

    let sky_weight = if reservoir.w_sum == 0.0 {
        1.0
    } else {
        0.25 * reservoir.w_sum
    };

    let mut sky_normal = Vec3::ZERO;

    if sky_weight > 0.0 {
        let mut sky_exposure = 8.0;

        // If we indirectly-hit nothing, we know that our indirect-ray must be
        // pointing towards the sky - great, let's use it!
        //
        // If we indirectly-hit something, we don't know in which way we can
        // sample the sky, so just take a random guess on the hemisphere on our
        // surface.
        if indirect_hit.is_none() {
            sky_normal = indirect_hit.direction;
        } else {
            sky_normal = wnoise.sample_hemisphere(indirect_hit.gbuffer.normal);
            sky_exposure *= indirect_hit.gbuffer.normal.dot(sky_normal);
        };

        let sample = DirectReservoirSample::sky(
            sky_exposure * atmosphere.eval(world.sun_direction(), sky_normal),
        );

        reservoir.add(&mut wnoise, sample, sky_weight * sample.p_hat());
    }

    // -------------------------------------------------------------------------
    // Step 2:
    //
    // Select the best light-candidate and cast a shadow ray to check if that
    // light (which might be sun) is actually visible to us.

    let light_pdf = reservoir.sample.p_hat() / reservoir.w_sum;

    let DirectReservoirSample {
        light_id,
        light_contribution,
        ..
    } = reservoir.sample;

    let light_visibility = if indirect_hit.is_some() {
        let light = if reservoir.sample.is_sky() {
            Light::sun(sky_normal * World::SUN_DISTANCE)
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
        // If we indirectly-hit nothing, we know that our indirect-ray must be
        // pointing towards the sky - great, no need to actually trace the ray!
        1.0
    };

    let radiance = indirect_hit.gbuffer.emissive
        + light_contribution * light_visibility / light_pdf;

    let indirect_normal;
    let indirect_point;

    if indirect_hit.is_some() {
        indirect_normal = Normal::encode(indirect_hit.gbuffer.normal);
        indirect_point = indirect_hit.point;
    } else {
        indirect_normal = Normal::encode(-sky_normal);
        indirect_point = sky_normal * World::SUN_DISTANCE;
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
