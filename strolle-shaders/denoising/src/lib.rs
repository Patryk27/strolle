#![no_std]

use spirv_std::glam::{
    ivec2, UVec2, UVec3, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};
use spirv_std::{spirv, Image};
use strolle_models::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &DenoisingPassParams,
    #[spirv(descriptor_set = 0, binding = 0)]
    directs: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 0, binding = 1)]
    pending_directs: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 0, binding = 2)]
    indirects: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 0, binding = 3)]
    pending_indirects: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 0, binding = 4)]
    normals: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 0, binding = 5)]
    pending_normals: &Image!(2D, format = rgba16f, sampled = false),
) {
    main_inner(
        global_id.xy(),
        params,
        directs,
        pending_directs,
        indirects,
        pending_indirects,
        normals,
        pending_normals,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    params: &DenoisingPassParams,
    directs: &Image!(2D, format = rgba16f, sampled = false),
    pending_directs: &Image!(2D, format = rgba16f, sampled = false),
    indirects: &Image!(2D, format = rgba16f, sampled = false),
    pending_indirects: &Image!(2D, format = rgba16f, sampled = false),
    normals: &Image!(2D, format = rgba16f, sampled = false),
    pending_normals: &Image!(2D, format = rgba16f, sampled = false),
) {
    let mut noise = Noise::new(params.seed, global_id.x, global_id.y);

    let pending_direct: Vec4 = pending_directs.read(global_id);
    let pending_indirect: Vec4 = pending_indirects.read(global_id);
    let pending_normal: Vec4 = pending_normals.read(global_id);

    let direct = pending_direct.xyz();
    let indirect = pending_indirect.xyz();
    let normal = pending_normal;

    let indirect = denoise_indirect_lightning(
        global_id, indirects, normals, &mut noise, indirect, normal,
    );

    unsafe {
        directs.write(global_id, direct.extend(1.0));
        indirects.write(global_id, indirect.extend(1.0));
        normals.write(global_id, normal);
    }
}

// TODO use camera reprojection
fn denoise_indirect_lightning(
    global_id: UVec2,
    indirects: &Image!(2D, format = rgba16f, sampled = false),
    normals: &Image!(2D, format = rgba16f, sampled = false),
    noise: &mut Noise,
    indirect: Vec3,
    normal: Vec4,
) -> Vec3 {
    // let temporal_indirect = {
    //     let prev_normal: Vec4 = normals.read(global_id);

    //     if are_normals_close(prev_normal, normal) {
    //         indirects.read(global_id).xyz()
    //     } else {
    //         Vec3::ZERO
    //     }
    // };

    let _spatial_indirect = {
        let mut neighbourhood = Vec4::ZERO;
        let mut n = 0;

        while n < 3 {
            let neighbour_dx = (3.0 * (noise.sample() - 0.5)) as i32;
            let neighbour_dy = (3.0 * (noise.sample() - 0.5)) as i32;

            let neighbour_xy =
                global_id.as_ivec2() + ivec2(neighbour_dx, neighbour_dy);

            if neighbour_xy.x >= 0 && neighbour_xy.y >= 0 {
                let neighbour_xy = neighbour_xy.as_uvec2();
                let neighbour_normal: Vec4 = normals.read(neighbour_xy);

                if are_normals_close(neighbour_normal, normal) {
                    neighbourhood +=
                        indirects.read(neighbour_xy).xyz().extend(1.0);
                }
            }

            n += 1;
        }

        neighbourhood.xyz() / neighbourhood.w.max(1.0)
    };

    // if temporal_indirect != Vec3::ZERO {
    //     indirect = (temporal_indirect + indirect) / 2.0;
    // }

    // if spatial_indirect != Vec3::ZERO {
    //     indirect = (1.5 * spatial_indirect + indirect) / 2.5;
    // }

    indirect
}

fn are_normals_close(prev: Vec4, curr: Vec4) -> bool {
    prev.xyz() != Vec3::ZERO
        && prev.xyz().distance_squared(curr.xyz()) < 0.01
        && prev.w == curr.w
}
