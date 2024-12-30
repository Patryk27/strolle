#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

pub trait F32Ext
where
    Self: Sized,
{
    fn sqr(self) -> Self;
    fn saturate(self) -> Self;
    fn inverse_sqrt(self) -> Self;
    fn acos_approx(self) -> Self;
}

impl F32Ext for f32 {
    fn sqr(self) -> Self {
        self * self
    }

    fn saturate(self) -> Self {
        self.clamp(0.0, 1.0)
    }

    fn inverse_sqrt(self) -> Self {
        1.0 / self.max(crate::STROLLE_EPSILON).sqrt()
    }

    fn acos_approx(self) -> Self {
        2.0f32.sqrt() * (1.0 - self).saturate().sqrt()
    }
}
