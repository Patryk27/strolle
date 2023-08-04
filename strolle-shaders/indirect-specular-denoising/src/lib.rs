#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)]
    indirect_specular_samples: TexRgba16f,
    #[spirv(descriptor_set = 0, binding = 1)]
    indirect_specular_colors: TexRgba16f,
) {
    let screen_pos = global_id.xy();

    unsafe {
        indirect_specular_colors.write(
            screen_pos,
            indirect_specular_samples.read(screen_pos).xyz().extend(1.0),
        );
    }
}
