use bytemuck::{Pod, Zeroable};
use glam::{vec3a, vec4, Affine3A, Mat3A, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DirectRasterPassParams {
    pub payload: Vec4,
    pub curr_xform_inv_d0: Vec4,
    pub curr_xform_inv_d1: Vec4,
    pub curr_xform_inv_d2: Vec4,
    pub prev_xform_d0: Vec4,
    pub prev_xform_d1: Vec4,
    pub prev_xform_d2: Vec4,
}

impl DirectRasterPassParams {
    pub fn instance_uuid(&self) -> u32 {
        self.payload.x.to_bits()
    }

    pub fn material_id(&self) -> u32 {
        self.payload.y.to_bits()
    }

    pub fn curr_xform_inv(&self) -> Affine3A {
        Self::decode_affine([
            self.curr_xform_inv_d0,
            self.curr_xform_inv_d1,
            self.curr_xform_inv_d2,
        ])
    }

    pub fn prev_xform(&self) -> Affine3A {
        Self::decode_affine([
            self.prev_xform_d0,
            self.prev_xform_d1,
            self.prev_xform_d2,
        ])
    }

    /// Encodes a 3D affine transformation as three Vec4s; we use this to
    /// overcome padding issues when copying data from CPU into GPU.
    pub fn encode_affine(xform: Affine3A) -> [Vec4; 3] {
        let d0 = vec4(
            xform.matrix3.x_axis.x,
            xform.matrix3.x_axis.y,
            xform.matrix3.x_axis.z,
            xform.translation.x,
        );

        let d1 = vec4(
            xform.matrix3.y_axis.x,
            xform.matrix3.y_axis.y,
            xform.matrix3.y_axis.z,
            xform.translation.y,
        );

        let d2 = vec4(
            xform.matrix3.z_axis.x,
            xform.matrix3.z_axis.y,
            xform.matrix3.z_axis.z,
            xform.translation.z,
        );

        [d0, d1, d2]
    }

    /// See: [`Self::encode_affine()`].
    pub fn decode_affine([d0, d1, d2]: [Vec4; 3]) -> Affine3A {
        Affine3A {
            matrix3: Mat3A {
                x_axis: vec3a(d0.x, d0.y, d0.z),
                y_axis: vec3a(d1.x, d1.y, d1.z),
                z_axis: vec3a(d2.x, d2.y, d2.z),
            },
            translation: vec3a(d0.w, d1.w, d2.w),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DirectInitialShadingPassParams {
    pub seed: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DirectTemporalResamplingPassParams {
    pub seed: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DirectSpatialResamplingPassParams {
    pub seed: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct IndirectInitialShadingPassParams {
    pub seed: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct IndirectInitialTracingPassParams {
    pub seed: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct IndirectTemporalResamplingPassParams {
    pub seed: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct IndirectSpatialResamplingPassParams {
    pub seed: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct IndirectResolvingPassParams {
    pub seed: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct OutputDrawingPassParams {
    pub camera_mode: u32,
}
