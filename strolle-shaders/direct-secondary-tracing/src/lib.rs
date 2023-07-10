#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(local_invocation_index)]
    local_idx: u32,
    #[spirv(workgroup)]
    stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3)]
    atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)]
    atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    direct_primary_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)]
    direct_primary_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)]
    direct_primary_hits_d2: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 4)]
    direct_secondary_rays: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 5)]
    direct_secondary_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)]
    direct_secondary_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 7)]
    direct_secondary_hits_d2: TexRgba16f,
) {
    main_inner(
        global_id.xy(),
        local_idx,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        MaterialsView::new(materials),
        atlas_tex,
        atlas_sampler,
        camera,
        direct_primary_hits_d0,
        direct_primary_hits_d1,
        direct_primary_hits_d2,
        direct_secondary_rays,
        direct_secondary_hits_d0,
        direct_secondary_hits_d1,
        direct_secondary_hits_d2,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    local_idx: u32,
    stack: BvhStack,
    triangles: TrianglesView,
    bvh: BvhView,
    materials: MaterialsView,
    atlas_tex: Tex,
    atlas_sampler: &Sampler,
    camera: &Camera,
    direct_primary_hits_d0: TexRgba32f,
    direct_primary_hits_d1: TexRgba32f,
    direct_primary_hits_d2: TexRgba16f,
    direct_secondary_rays: TexRgba32f,
    direct_secondary_hits_d0: TexRgba32f,
    direct_secondary_hits_d1: TexRgba32f,
    direct_secondary_hits_d2: TexRgba16f,
) {
    let primary_albedo = direct_primary_hits_d2.read(screen_pos);

    // If our fragment is opaque, there's nothing more to do here
    if primary_albedo.w >= 0.99 {
        unsafe {
            direct_secondary_rays.write(screen_pos, Vec4::ZERO);
            direct_secondary_hits_d2.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    let primary_hit = Hit::deserialize(
        direct_primary_hits_d0.read(screen_pos),
        direct_primary_hits_d1.read(screen_pos),
    );

    if primary_hit.is_none() {
        unsafe {
            direct_secondary_rays.write(screen_pos, Vec4::ZERO);
            direct_secondary_hits_d2.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    let primary_ray = camera.ray(screen_pos);
    let primary_material = materials.get(primary_hit.material_id);

    // -------------------------------------------------------------------------

    let secondary_ray = {
        let origin = primary_hit.point - primary_hit.normal * 0.02;

        let direction = if primary_material.reflectivity > 0.0 {
            primary_ray.direction().reflect(primary_hit.normal)
        } else {
            let mut cos_incident_angle =
                primary_hit.normal.dot(-primary_ray.direction());

            let eta = if cos_incident_angle > 0.0 {
                primary_material.refraction
            } else {
                1.0 / primary_material.refraction
            };

            let refraction_coeff =
                1.0 - (1.0 - cos_incident_angle.powi(2)) / eta.powi(2);

            // if refraction_coeff < 0.0 {
            //     TODO
            //     return;
            // }

            let mut normal = primary_hit.normal;
            let cos_transmitted_angle = refraction_coeff.sqrt();

            if cos_incident_angle < 0.0 {
                normal = -normal;
                cos_incident_angle = -cos_incident_angle;
            }

            primary_ray.direction() / eta
                - normal * (cos_transmitted_angle - cos_incident_angle / eta)
        };

        Ray::new(origin, direction)
    };

    // -------------------------------------------------------------------------

    // TODO trace up to the nearest *opaque* surface
    let (secondary_hit, _) =
        secondary_ray.trace_nearest(local_idx, triangles, bvh, stack);

    let secondary_albedo = if secondary_hit.is_some() {
        materials.get(secondary_hit.material_id).albedo(
            atlas_tex,
            atlas_sampler,
            secondary_hit.uv,
        )
    } else {
        Vec4::ZERO
    };

    let [secondary_hit_d0, secondary_hit_d1] = secondary_hit.serialize();

    unsafe {
        direct_secondary_rays.write(
            screen_pos,
            secondary_ray.direction().extend(Default::default()),
        );

        direct_secondary_hits_d0.write(screen_pos, secondary_hit_d0);
        direct_secondary_hits_d1.write(screen_pos, secondary_hit_d1);
        direct_secondary_hits_d2.write(screen_pos, secondary_albedo);
    }
}
