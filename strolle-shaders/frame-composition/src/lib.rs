#![no_std]

use strolle_gpu::prelude::*;

#[spirv(fragment)]
pub fn main(
    #[spirv(descriptor_set = 0, binding = 0)] sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 1)] indirect_diffuse: Tex,
    uv: Vec2,
    frag_color: &mut Vec4,
) {
    *frag_color = indirect_diffuse.sample(*sampler, uv);
}
