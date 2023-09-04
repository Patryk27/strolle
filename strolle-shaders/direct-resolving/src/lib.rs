#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    direct_initial_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    direct_next_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    direct_prev_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 4)] direct_samples: TexRgba16f,
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);

    // -------------------------------------------------------------------------

    let initial_sample =
        unsafe { *direct_initial_samples.get_unchecked(2 * screen_idx) };

    if initial_sample.w.to_bits() == 0 {
        unsafe {
            direct_samples.write(screen_pos, initial_sample);
        }
    } else {
        let reservoir = DirectReservoir::read(
            direct_next_reservoirs,
            camera.screen_to_idx(screen_pos),
        );

        let out = reservoir.sample.light_radiance * reservoir.w;

        unsafe {
            direct_samples.write(screen_pos, out.extend(1.0));

            // TODO swap buffers instead of copying them
            *direct_prev_reservoirs.get_unchecked_mut(3 * screen_idx + 0) =
                *direct_next_reservoirs.get_unchecked(3 * screen_idx + 0);

            *direct_prev_reservoirs.get_unchecked_mut(3 * screen_idx + 1) =
                *direct_next_reservoirs.get_unchecked(3 * screen_idx + 1);

            *direct_prev_reservoirs.get_unchecked_mut(3 * screen_idx + 2) =
                *direct_next_reservoirs.get_unchecked(3 * screen_idx + 2);
        }
    }
}
