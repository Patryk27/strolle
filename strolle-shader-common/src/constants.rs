use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Default)]
pub struct ShaderConstants {
    pub width: f32,
    pub height: f32,
    pub scaled_width: f32,
    pub scaled_height: f32,
    pub time: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Default)]
pub struct Projection {
    pub view_proj_1: Vec4,
    pub view_proj_2: Vec4,
    pub view_proj_3: Vec4,
    pub view_proj_4: Vec4,
}

impl Projection {
    pub fn new(view_proj: Mat4) -> Self {
        Self {
            view_proj_1: view_proj.x_axis,
            view_proj_2: view_proj.y_axis,
            view_proj_3: view_proj.z_axis,
            view_proj_4: view_proj.w_axis,
        }
    }

    pub fn view_proj(&self) -> Mat4 {
        Mat4::from_cols(
            self.view_proj_1,
            self.view_proj_2,
            self.view_proj_3,
            self.view_proj_4,
        )
    }
}
