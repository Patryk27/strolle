#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 5)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 6)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 7, uniform)] world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    atmosphere_transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 2)]
    atmosphere_transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)] atmosphere_sky_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 4)]
    atmosphere_sky_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)] direct_hits: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 7)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 8, storage_buffer)]
    direct_initial_samples: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
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

    let mut hit = Hit::from_direct(
        camera.ray(screen_pos),
        direct_hits.read(screen_pos).xyz(),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // TODO describe
    hit.gbuffer.base_color = Vec4::splat(1.0);

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Select the best light-candidate, judging lights by their *unshadowed*
    // contribution (i.e. during this phase we don't cast shadow rays).

    let mut reservoir = DirectReservoir::default();

    if hit.is_some() {
        let mut light_idx = 0;

        while light_idx < world.light_count {
            let light_id = LightId::new(light_idx);

            let light_contribution =
                lights.get(light_id).contribution(hit).diffuse;

            let sample = DirectReservoirSample {
                light_id,
                light_contribution,
                light_pdf: 1.0,
                hit_point: hit.point,
            };

            reservoir.add(&mut wnoise, sample, sample.p_hat());
            light_idx += 1;
        }
    } else {
        // If we hit nothing, the reservoir will remain empty and we'll just
        // sample the sky in a moment.
    }

    // If the reservoir is empty (i.e. it has seen zero samples or all of the
    // samples are unusable¹), sample the sky.
    //
    // If the reservoir has seen some samples, sample the sky with 25%
    // probability (following the ReSTIR paper).
    //
    // ¹ e.g. are lights very very far away from here
    let sky_weight = if reservoir.w_sum == 0.0 {
        1.0
    } else {
        0.25 * reservoir.w_sum
    };

    if sky_weight > 0.0 {
        // If we hit nothing, sample the sky - otherwise, sample the sun, so:
        //
        // - if the user is looking at the sky, we get no-hit and sample the
        //   sky,
        //
        // - if the user is looking at the world, we get hit and sample the sun
        //   so that the sun is able to cast shadows.
        let sky = if hit.is_none() {
            atmosphere.eval(world.sun_direction(), hit.direction)
        } else {
            atmosphere.sun(world.sun_direction())
        };

        let sample = DirectReservoirSample::sky(sky);

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
        hit_point,
        ..
    } = reservoir.sample;

    let light_visibility = if hit.is_some() {
        let light = if reservoir.sample.is_sky() {
            Light::sun(world.sun_position())
        } else {
            lights.get(reservoir.sample.light_id)
        };

        light.visibility_bnoise(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            bnoise,
            hit,
        )
    } else {
        1.0
    };

    let light = light_contribution * light_visibility;

    unsafe {
        *direct_initial_samples.get_unchecked_mut(2 * screen_idx + 0) =
            light.extend(f32::from_bits(light_id.get()));

        *direct_initial_samples.get_unchecked_mut(2 * screen_idx + 1) =
            hit_point.extend(light_pdf);
    }
}
