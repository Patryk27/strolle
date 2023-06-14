#![no_std]

use core::f32::consts::PI;

use spirv_std::glam::{vec2, UVec2, UVec3, Vec3Swizzles, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use strolle_gpu::{
    upsample, Camera, GeometryMap, IndirectReservoir,
    IndirectResolvingPassParams, Noise, TexRgba16f, TexRgba32f,
};

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &IndirectResolvingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    geometry_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    raw_indirect_colors: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    indirect_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        GeometryMap::new(geometry_map),
        raw_indirect_colors,
        indirect_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    params: &IndirectResolvingPassParams,
    camera: &Camera,
    geometry_map: GeometryMap,
    raw_indirect_colors: TexRgba16f,
    indirect_spatial_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, screen_pos);
    let mut out = Vec4::ZERO;
    let mut sample_idx = 0;

    let screen_geo = geometry_map.get(screen_pos);

    while sample_idx < 8 {
        let reservoir_pos_offset = {
            let angle = noise.sample() * PI * 2.0;
            let distance = 2.0 * sample_idx as f32;

            vec2(angle.sin(), angle.cos()) * distance
        };

        let reservoir_pos = (screen_pos.as_vec2() / 2.0) + reservoir_pos_offset;
        let reservoir_pos = reservoir_pos.as_ivec2();

        if reservoir_pos.x < 0 || reservoir_pos.y < 0 {
            sample_idx += 1;
            continue;
        }

        let reservoir_pos = reservoir_pos.as_uvec2();
        let reservoir_screen_pos = upsample(reservoir_pos, params.frame);

        if !camera.contains(reservoir_screen_pos.as_ivec2()) {
            sample_idx += 1;
            continue;
        }

        let reservoir = IndirectReservoir::read(
            indirect_spatial_reservoirs,
            camera.half_screen_to_idx(reservoir_pos),
        );

        let reservoir_screen_geo = geometry_map.get(reservoir_screen_pos);

        let reservoir_color = reservoir.sample().radiance * reservoir.w;

        let mut reservoir_weight = 1.0;

        reservoir_weight *=
            screen_geo.evaluate_similarity_to(&reservoir_screen_geo);

        reservoir_weight *= reservoir.m_sum.sqrt().max(1.0).min(5.0);

        out += (reservoir_color * reservoir_weight).extend(reservoir_weight);
        sample_idx += 1;
    }

    let out = out.xyz() / out.w.max(1.0);

    unsafe {
        raw_indirect_colors.write(screen_pos, out.extend(1.0));
    }
}
