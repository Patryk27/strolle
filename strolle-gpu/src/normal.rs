use glam::{vec3, Vec2, Vec3, Vec3Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

pub struct Normal;

impl Normal {
    /// Compresses normal from Vec3 into Vec2 using octahedron-normal mapping.
    pub fn encode(n: Vec3) -> Vec2 {
        let n = n / (n.x.abs() + n.y.abs() + n.z.abs());

        let n = if n.z >= 0.0 {
            n.xy()
        } else {
            (1.0 - n.yx().abs()) * n.xy().signum()
        };

        n * 0.5 + 0.5
    }

    /// See: [`Self::encode()`].
    pub fn decode(n: Vec2) -> Vec3 {
        let n = n * 2.0 - 1.0;
        let mut n = vec3(n.x, n.y, 1.0 - n.x.abs() - n.y.abs());
        let t = (-n.z).max(0.0);

        n.x += if n.x > 0.0 { -t } else { t };
        n.y += if n.y > 0.0 { -t } else { t };
        n.normalize()
    }
}
