#![no_std]

use spirv_std::glam::{uvec2, vec3, UVec2, UVec3, Vec3, Vec3Swizzles, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::float::Float;
use spirv_std::spirv;
use strolle_models::*;

#[rustfmt::skip]
#[spirv(compute(threads(16, 16)))]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &VoxelPaintingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    voxels: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    pending_voxels: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        VoxelsViewMut::new(voxels),
        PendingVoxelsView::new(pending_voxels),
    )
}

fn main_inner(
    global_id: UVec2,
    params: &VoxelPaintingPassParams,
    camera: &Camera,
    mut voxels: VoxelsViewMut,
    pending_voxels: PendingVoxelsView,
) {
    let viewport_size = camera.viewport_size();
    let pv_width = camera.viewport_size().x / 2;

    // This pass uses 16x16 warps and the pending-voxels texture has 1/4th of
    // the viewport's resolution:
    let chunk_size = viewport_size / 16 / 2;

    let mut dx = 0;
    let mut dy = 0;

    loop {
        let pending_voxel_id = PendingVoxelId::from_xy(
            pv_width,
            global_id * chunk_size + uvec2(dx, dy),
        );

        let pending_voxel = pending_voxels.get(pending_voxel_id);

        if pending_voxel.frame == params.frame {
            let voxel = voxels.get(pending_voxel.voxel_id);

            if voxel.frame == params.frame {
                let strength = if voxel.samples >= 8.0 {
                    let curr_lum = luminance(voxel.color());
                    let sample_lum = luminance(pending_voxel.color);

                    (1.0f32 - (sample_lum - curr_lum).abs()).max(0.1).min(1.0)
                } else {
                    1.0
                };

                voxels.add_sample(
                    pending_voxel.voxel_id,
                    pending_voxel.color,
                    strength,
                );
            } else if voxel.samples == 0.0 || !voxel.is_fresh(params.frame) {
                voxels.set(
                    pending_voxel.voxel_id,
                    Voxel {
                        accum_color: pending_voxel.color,
                        samples: 1.0,
                        point: pending_voxel.point,
                        frame: params.frame,
                    },
                );
            } else {
                let accum_color;
                let samples;

                if voxel.is_nearby(pending_voxel.point) {
                    let coeff = if voxel.samples >= 32.0 { 0.95 } else { 1.0 };

                    let strength = if voxel.samples >= 8.0 {
                        let curr_lum = luminance(voxel.color());
                        let sample_lum = luminance(pending_voxel.color);

                        (1.0f32 - (sample_lum - curr_lum).abs())
                            .max(0.1)
                            .min(1.0)
                    } else {
                        1.0
                    };

                    accum_color = voxel.accum_color * coeff
                        + strength * pending_voxel.color;

                    samples = voxel.samples * coeff + strength;
                } else {
                    accum_color = pending_voxel.color;
                    samples = 1.0;
                }

                voxels.set(
                    pending_voxel.voxel_id,
                    Voxel {
                        accum_color,
                        samples,
                        point: pending_voxel.point,
                        frame: params.frame,
                    },
                );
            }
        }

        // ---

        dx += 1;

        if dx == chunk_size.x {
            dx = 0;
            dy += 1;

            if dy == chunk_size.y {
                break;
            }
        }
    }
}

fn luminance(color: Vec3) -> f32 {
    color.dot(vec3(0.2126, 0.7152, 0.0722))
}
