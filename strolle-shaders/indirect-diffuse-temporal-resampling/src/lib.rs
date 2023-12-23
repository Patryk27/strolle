#![no_std]

use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prev_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_diffuse_curr_temporal_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    indirect_diffuse_prev_temporal_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, global_id.xy());
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);
    let reprojection = reprojection_map.get(screen_pos);

    let mut main = IndirectReservoir::default();
    let mut main_pdf = 0.0;

    // ---

    let d0 = unsafe { *indirect_samples.index_unchecked(3 * screen_idx) };
    let d1 = unsafe { *indirect_samples.index_unchecked(3 * screen_idx + 1) };
    let d2 = unsafe { *indirect_samples.index_unchecked(3 * screen_idx + 2) };

    let hit_point = d0.xyz();
    let hit_normal = surface.normal;

    if d0.w.to_bits() == 1 {
        let sample = IndirectReservoirSample {
            radiance: d1.xyz(),
            direct_point: d0.xyz(),
            indirect_point: d2.xyz(),
            indirect_normal: Normal::decode(vec2(d1.w, d2.w)),
            frame: params.frame,
        };

        main_pdf = sample.diffuse_pdf(hit_point, hit_normal);
        main.update(&mut wnoise, sample, main_pdf);
    }

    // ---

    if reprojection.is_some() {
        let mut sample = IndirectReservoir::read(
            indirect_diffuse_prev_temporal_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        sample.clamp_m(50.0);

        let sample_pdf = sample.sample.diffuse_pdf(hit_point, hit_normal);

        if main.merge(&mut wnoise, &sample, sample_pdf) {
            main_pdf = sample_pdf;
        }
    }

    // ---

    let mut sample_idx = 0;

    while main.m < 16.0 && sample_idx < 4 {
        let mut sample_pos = if reprojection.is_some() {
            reprojection.prev_pos_round().as_ivec2()
        } else {
            screen_pos.as_ivec2()
        };

        sample_pos += (wnoise.sample_disk() * 32.0).as_ivec2();
        sample_idx += 1;

        let sample_pos = camera.contain(sample_pos);
        let sample_surface = prev_surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            continue;
        }

        if (sample_surface.depth - surface.depth).abs() > 0.2 * surface.depth {
            continue;
        }

        if sample_surface.normal.dot(surface.normal) < 0.8 {
            continue;
        }

        let mut sample = IndirectReservoir::read(
            indirect_diffuse_prev_temporal_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        sample.clamp_m(1.0);

        let sample_pdf = sample.sample.diffuse_pdf(hit_point, hit_normal);

        if main.merge(&mut wnoise, &sample, sample_pdf) {
            main_pdf = sample_pdf;
        }
    }

    // -------------------------------------------------------------------------

    main.normalize(main_pdf);
    main.write(indirect_diffuse_curr_temporal_reservoirs, screen_idx);
}
