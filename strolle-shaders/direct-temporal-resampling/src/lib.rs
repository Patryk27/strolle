#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
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
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_candidates: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    direct_curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let lights = LightsView::new(lights);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.is_none() {
        return;
    }

    // ---

    let candidate = unsafe { *direct_candidates.get_unchecked(screen_idx) };

    let mut reservoir = DirectReservoir::default();
    let mut reservoir_p_hat = 0.0;

    let mut subject_mode = SubjectMode::None;
    let mut subject_reservoir = DirectReservoir::default();
    let mut subject_p_hat = 0.0;
    let mut subject_ray = Ray::default();
    let mut subject_ray_distance = 0.0;

    // ---

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        subject_reservoir = DirectReservoir::read(
            direct_prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        subject_reservoir.clamp_m(20.0 * candidate.x);

        subject_p_hat = subject_reservoir.sample.p_hat(lights, hit);

        if subject_p_hat > 0.0 {
            if debug::DIRECT_VALIDATION_ENABLED && params.frame % 3 == 0 {
                subject_mode = SubjectMode::Reprojection;

                (subject_ray, subject_ray_distance) =
                    subject_reservoir.sample.ray(hit);
            } else if reservoir.merge(
                &mut wnoise,
                &subject_reservoir,
                subject_p_hat,
            ) {
                reservoir_p_hat = subject_p_hat;
            }
        }
    }

    if subject_mode == SubjectMode::None {
        let light_id = LightId::new(candidate.z.to_bits());

        subject_mode = SubjectMode::Candidate;

        (subject_ray, subject_ray_distance) =
            lights.get(light_id).ray(&mut wnoise, hit.point);

        subject_reservoir = DirectReservoir {
            reservoir: Reservoir {
                sample: DirectReservoirSample {
                    light_id,
                    light_position: subject_ray.origin(),
                    surface_point: hit.point,
                },
                w_sum: 0.0,
                m_sum: candidate.x,
                w: candidate.y,
            },
            frame: params.frame,
        };

        subject_p_hat = candidate.w;
    }

    // ---

    if subject_mode != SubjectMode::None {
        if subject_ray.intersect(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            subject_ray_distance,
        ) {
            subject_reservoir.w = 0.0;
        }

        if reservoir.merge(&mut wnoise, &subject_reservoir, subject_p_hat) {
            reservoir_p_hat = subject_p_hat;
        }
    }

    // ---

    reservoir.normalize(reservoir_p_hat);
    reservoir.clamp_w(10.0);
    reservoir.write(direct_curr_reservoirs, screen_idx);
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum SubjectMode {
    None,
    Candidate,
    Reprojection,
}
