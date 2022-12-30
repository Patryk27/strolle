use bytemuck::{Pod, Zeroable};
use glam::{vec4, Mat4, Vec4};

use crate::{BvhPtr, MaterialId, TriangleId};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct Instance {
    d0: Vec4,
    d1: Vec4,
    d2: Vec4,
    d3: Vec4,
    d4: Vec4,
    d5: Vec4,
    d6: Vec4,
    d7: Vec4,
    d8: Vec4,
}

impl Instance {
    pub fn new(
        transform: Mat4,
        min_triangle_id: TriangleId,
        max_triangle_id: TriangleId,
        material_id: MaterialId,
        bvh_ptr: BvhPtr,
    ) -> Self {
        let inv_transform = transform.inverse();

        Self {
            d0: transform.x_axis,
            d1: transform.y_axis,
            d2: transform.z_axis,
            d3: transform.w_axis,
            d4: inv_transform.x_axis,
            d5: inv_transform.y_axis,
            d6: inv_transform.z_axis,
            d7: inv_transform.w_axis,
            d8: vec4(
                f32::from_bits(min_triangle_id.get()),
                f32::from_bits(max_triangle_id.get()),
                f32::from_bits(material_id.get()),
                f32::from_bits(bvh_ptr.get()),
            ),
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4 {
            x_axis: self.d0,
            y_axis: self.d1,
            z_axis: self.d2,
            w_axis: self.d3,
        }
    }

    pub fn inv_transform(&self) -> Mat4 {
        Mat4 {
            x_axis: self.d4,
            y_axis: self.d5,
            z_axis: self.d6,
            w_axis: self.d7,
        }
    }

    pub fn min_triangle_id(&self) -> TriangleId {
        TriangleId::new(self.d8.x.to_bits())
    }

    pub fn max_triangle_id(&self) -> TriangleId {
        TriangleId::new(self.d8.y.to_bits())
    }

    pub fn material_id(&self) -> MaterialId {
        MaterialId::new(self.d8.z.to_bits())
    }

    pub fn bvh_ptr(&self) -> BvhPtr {
        BvhPtr::new(self.d8.w.to_bits())
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct InstanceId(u32);

impl InstanceId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}
