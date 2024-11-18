// First, let's define helper traits for converting between CPU and GPU types
use glam as cpu;
use spirv_std::glam as gpu;

// Helper trait for converting CPU types to GPU types
pub trait ToGpu<T> {
    fn to_gpu(&self) -> T;
}


impl ToGpu<gpu::Affine3A> for cpu::Affine3A {
    fn to_gpu(&self) -> gpu::Affine3A {
        gpu::Affine3A::from_cols_array(&self.to_cols_array())
    }
}

impl ToGpu<gpu::Mat4> for cpu::Mat4 {
    fn to_gpu(&self) -> gpu::Mat4 {
        gpu::Mat4::from_cols_array(&self.to_cols_array())
    }
}

impl ToGpu<gpu::Vec4> for cpu::Vec4 {
    fn to_gpu(&self) -> gpu::Vec4 {
        gpu::Vec4::new(self.x, self.y, self.z, self.w)
    }
}

impl ToGpu<gpu::Vec3> for cpu::Vec3 {
    fn to_gpu(&self) -> gpu::Vec3 {
        gpu::Vec3::new(self.x, self.y, self.z)
    }
}

impl ToGpu<gpu::Vec2> for cpu::Vec2 {
    fn to_gpu(&self) -> gpu::Vec2 {
        gpu::Vec2::new(self.x, self.y)
    }
}


impl ToGpu<gpu::UVec2> for cpu::UVec2 {
    fn to_gpu(&self) -> gpu::UVec2 {
        gpu::UVec2::new(self.x, self.y)
    }
}

impl ToGpu<gpu::UVec3> for cpu::UVec3 {
    fn to_gpu(&self) -> gpu::UVec3 {
        gpu::UVec3::new(self.x, self.y, self.z)
    }
}

impl ToGpu<gpu::UVec4> for cpu::UVec4 {
    fn to_gpu(&self) -> gpu::UVec4 {
        gpu::UVec4::new(self.x, self.y, self.z, self.w)
    }
}