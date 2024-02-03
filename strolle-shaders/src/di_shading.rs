use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 5)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 6)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 7, uniform)] world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
    let lights = LightsView::new(lights);

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

    if hit.is_none() {
        return;
    }

    // ---

    let mut res = EphemeralReservoir::default();
    let mut res_pdf = 0.0;

    if world.light_count > 0 {
        let sample_ipdf = world.light_count as f32;
        let mut sample_idx = 0;

        while sample_idx < 16 {
            let light_id =
                LightId::new(wnoise.sample_int() % world.light_count);

            let light_radiance =
                lights.get(light_id).radiance(hit.point, hit.gbuffer.normal);

            let sample = EphemeralSample {
                light_id,
                light_radiance,
            };

            let sample_pdf = sample.pdf();

            if sample_pdf > 0.0 {
                if res.update(&mut wnoise, sample, sample_pdf * sample_ipdf) {
                    res_pdf = sample_pdf;
                }
            }

            sample_idx += 1;
        }
    }

    res.normalize(res_pdf);

    // ---

    let res = if res.m > 0.0 {
        let ray = lights
            .get(res.sample.light_id)
            .ray_bnoise(bnoise.first_sample(), hit.point);

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
            res.w = 0.0;
        }

        DiReservoir {
            reservoir: Reservoir {
                sample: DiSample {
                    light_id: res.sample.light_id,
                    light_point: ray.origin(),
                    exists: true,
                    is_occluded,
                },
                m: res.m,
                w: res.w,
            },
        }
    } else {
        Default::default()
    };

    res.write(curr_reservoirs, screen_idx);
}
