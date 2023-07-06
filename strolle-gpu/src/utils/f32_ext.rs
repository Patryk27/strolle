#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

pub trait F32Ext
where
    Self: Sized,
{
    fn saturate(self) -> Self;
    fn inverse_sqrt(self) -> Self;
}

impl F32Ext for f32 {
    fn saturate(self) -> Self {
        self.clamp(0.0, 1.0)
    }

    fn inverse_sqrt(self) -> Self {
        1.0 / self.sqrt()
    }
}
