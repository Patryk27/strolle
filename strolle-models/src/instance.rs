use bytemuck::{Pod, Zeroable};
use glam::{vec4, Mat4, Vec4};

use crate::{BvhPtr, MaterialId, TriangleId};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct Instance {
    transform_x_axis: Vec4,
    transform_y_axis: Vec4,
    transform_z_axis: Vec4,
    transform_w_axis: Vec4,
    inv_transform_x_axis: Vec4,
    inv_transform_y_axis: Vec4,
    inv_transform_z_axis: Vec4,
    inv_transform_w_axis: Vec4,
    meta: Vec4,
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
            transform_x_axis: transform.x_axis,
            transform_y_axis: transform.y_axis,
            transform_z_axis: transform.z_axis,
            transform_w_axis: transform.w_axis,
            inv_transform_x_axis: inv_transform.x_axis,
            inv_transform_y_axis: inv_transform.y_axis,
            inv_transform_z_axis: inv_transform.z_axis,
            inv_transform_w_axis: inv_transform.w_axis,
            meta: vec4(
                f32::from_bits(min_triangle_id.get()),
                f32::from_bits(max_triangle_id.get()),
                f32::from_bits(material_id.get()),
                f32::from_bits(bvh_ptr.get()),
            ),
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4 {
            x_axis: self.transform_x_axis,
            y_axis: self.transform_y_axis,
            z_axis: self.transform_z_axis,
            w_axis: self.transform_w_axis,
        }
    }

    pub fn inv_transform(&self) -> Mat4 {
        Mat4 {
            x_axis: self.inv_transform_x_axis,
            y_axis: self.inv_transform_y_axis,
            z_axis: self.inv_transform_z_axis,
            w_axis: self.inv_transform_w_axis,
        }
    }

    pub fn min_triangle_id(&self) -> TriangleId {
        TriangleId::new(self.meta.x.to_bits())
    }

    pub fn max_triangle_id(&self) -> TriangleId {
        TriangleId::new(self.meta.y.to_bits())
    }

    pub fn material_id(&self) -> MaterialId {
        MaterialId::new(self.meta.z.to_bits())
    }

    pub fn bvh_ptr(&self) -> BvhPtr {
        BvhPtr::new(self.meta.w.to_bits())
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
