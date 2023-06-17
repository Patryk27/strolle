#![no_std]

use spirv_std::glam::{UVec2, UVec3, Vec3, Vec3Swizzles, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_gpu::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(local_invocation_index)]
    local_idx: u32,
    #[spirv(push_constant)]
    params: &DirectShadingPassParams,
    #[spirv(workgroup)]
    stack: BvhTraversingStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[BvhNode],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 4, uniform)]
    world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    atmosphere_transmittance_lut_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 1, binding = 2)]
    atmosphere_transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)]
    atmosphere_sky_lut_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 1, binding = 4)]
    atmosphere_sky_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)]
    direct_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)]
    direct_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 7)]
    direct_hits_d2: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 8)]
    direct_colors: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        local_idx,
        params,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        LightsView::new(lights),
        MaterialsView::new(materials),
        Atmosphere::new(
            atmosphere_transmittance_lut_tex,
            atmosphere_transmittance_lut_sampler,
            atmosphere_sky_lut_tex,
            atmosphere_sky_lut_sampler,
        ),
        world,
        camera,
        direct_hits_d0,
        direct_hits_d1,
        direct_hits_d2,
        direct_colors,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    local_idx: u32,
    params: &DirectShadingPassParams,
    stack: BvhTraversingStack,
    triangles: TrianglesView,
    bvh: BvhView,
    lights: LightsView,
    materials: MaterialsView,
    atmosphere: Atmosphere,
    world: &World,
    camera: &Camera,
    direct_hits_d0: TexRgba32f,
    direct_hits_d1: TexRgba32f,
    direct_hits_d2: TexRgba32f,
    direct_colors: TexRgba16f,
) {
    let mut noise = Noise::new(params.seed, screen_pos);
    let ray = camera.ray(screen_pos);

    let hit = Hit::deserialize(
        direct_hits_d0.read(screen_pos),
        direct_hits_d1.read(screen_pos),
    );

    if hit.is_none() {
        let sky = atmosphere
            .eval(world.sun_direction(), camera.ray(screen_pos).direction());

        unsafe {
            direct_colors.write(screen_pos, sky.extend(1.0));
        }

        return;
    }

    let mut color = Vec3::ZERO;
    let material = materials.get(MaterialId::new(hit.material_id));
    let albedo = direct_hits_d2.read(screen_pos).xyz();

    let mut light_id = 0;

    while light_id < world.light_count {
        let light = lights.get(LightId::new(light_id));

        color += light.eval(
            local_idx, triangles, bvh, stack, &mut noise, material, hit, ray,
            albedo,
        );

        light_id += 1;
    }

    unsafe {
        direct_colors.write(screen_pos, color.extend(1.0));
    }
}
