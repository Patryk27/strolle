use std::ops::Add;

use spirv_std::glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Self {
        points.into_iter().fold(Self::default(), Self::add)
    }

    pub fn grow(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn extent(&self) -> Vec3 {
        self.max() - self.min()
    }

    pub fn area(&self) -> f32 {
        let extent = self.extent();

        extent.x * extent.y + extent.y * extent.z + extent.z * extent.x
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
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
        if rhs.min != Self::default().min {
            self.grow(rhs.min);
        }

        if rhs.max != Self::default().max {
            self.grow(rhs.max);
        }

        self
    }
}
