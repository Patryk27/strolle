//! This pass generates lookup textures used to render sky.
//!
//! Thanks to:
//!
//! - https://www.shadertoy.com/view/slSXRW
//!   (Production Sky Rendering by AndrewHelmer)
//!
//! - https://github.com/sebh/UnrealEngineSkyAtmosphere
//!
//! Original license:
//!
//! ```text
//! MIT License
//!
//! Copyright (c) 2020 Epic Games, Inc.
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```

pub mod generate_scattering_lut;
pub mod generate_sky_lut;
pub mod generate_transmittance_lut;
mod utils;

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn generate_scattering_lut(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 1)]
    transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] out: TexRgba16,
) {
    generate_scattering_lut::main(
        global_id,
        transmittance_lut_tex,
        transmittance_lut_sampler,
        out,
    );
}

#[spirv(compute(threads(8, 8)))]
pub fn generate_sky_lut(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] world: &World,
    #[spirv(descriptor_set = 0, binding = 1)] transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 2)]
    transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] scattering_lut_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)] scattering_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 5)] out: TexRgba16,
) {
    generate_sky_lut::main(
        global_id,
        world,
        transmittance_lut_tex,
        transmittance_lut_sampler,
        scattering_lut_tex,
        scattering_lut_sampler,
        out,
    );
}

#[spirv(compute(threads(8, 8)))]
pub fn generate_transmittance_lut(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] out: TexRgba16,
) {
    generate_transmittance_lut::main(global_id, out);
}
