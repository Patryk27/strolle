use std::ops::Add;

use spirv_std::glam::Vec3;
use strolle_models as gpu;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    min: Option<Vec3>,
    max: Option<Vec3>,
}

impl BoundingBox {
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Self {
        points.into_iter().fold(Self::default(), Self::add).fixup()
    }

    pub fn from_triangles(
        triangles: impl IntoIterator<Item = gpu::Triangle>,
    ) -> Self {
        Self::from_points(
            triangles
                .into_iter()
                .flat_map(|triangle| triangle.vertices()),
        )
    }

    pub fn from_triangle(triangle: gpu::Triangle) -> Self {
        Self::from_points(triangle.vertices())
    }

    /// Adjusts the bounding box so that all of its axes have non-zero length;
    /// this helps in raytracing, because our ray-box tests can assume that all
    /// bounding boxes are "well-formed".
    ///
    /// A bounding box can end up degraded this way when it represents e.g. a
    /// floor (its length in the `Y` axis can be then `0.0`).
    fn fixup(mut self) -> Self {
        if let (Some(min), Some(max)) = (&mut self.min, &mut self.max) {
            if max.x - min.x <= 0.001 {
                max.x = min.x + 0.001;
            }

            if max.y - min.y <= 0.001 {
                max.y = min.y + 0.001;
            }

            if max.z - min.z <= 0.001 {
                max.z = min.z + 0.001;
            }
        }

        self
    }

    pub fn grow(&mut self, p: Vec3) {
        if let Some(min) = &mut self.min {
            *min = min.min(p);
        } else {
            self.min = Some(p);
        }

        if let Some(max) = &mut self.max {
            *max = max.max(p);
        } else {
            self.max = Some(p);
        }
    }

    pub fn min(&self) -> Vec3 {
        self.min.expect("Bounding box is empty")
    }

    pub fn max(&self) -> Vec3 {
        self.max.expect("Bounding box is empty")
    }

    pub fn extent(&self) -> Vec3 {
        self.max() - self.min()
    }

    pub fn area(&self) -> f32 {
        if let (Some(min), Some(max)) = (self.min, self.max) {
            let extent = max - min;

            assert!(extent.length() > 0.0);

            extent.x * extent.y + extent.y * extent.z + extent.z * extent.x
        } else {
            0.0
        }
    }

    /// Maps `p` from `self.min() ..= self.max()` to `0.0 ..= 1.0`.
    pub fn map(&self, mut p: Vec3) -> Vec3 {
        let clip = |n: &mut f32| {
            assert!(
                !n.is_nan() && !n.is_infinite(),
                "Couldn't map point onto {self:?}",
            );

            if *n > -0.001 && *n < 0.0 {
                *n = 0.0;
            }

            if *n > 1.0 && *n < 1.001 {
                *n = 1.0;
            }
        };

        p = (p - self.min()) / self.extent();

        clip(&mut p.x);
        clip(&mut p.y);
        clip(&mut p.z);

        p
    }
}

impl Add<Vec3> for BoundingBox {
    type Output = Self;

    fn add(mut self, rhs: Vec3) -> Self::Output {
        self.grow(rhs);
        self
    }
}

impl Add<Self> for BoundingBox {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        if let Some(min) = rhs.min {
            self.grow(min);
        }

        if let Some(max) = rhs.max {
            self.grow(max);
        }

        self
    }
}
