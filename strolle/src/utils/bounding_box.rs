use std::ops::{Add, AddAssign};

use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
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

    pub fn half_area(&self) -> f32 {
        let extent = self.extent();

        extent.x * extent.y + extent.y * extent.z + extent.z * extent.x
    }

    pub fn is_set(&self) -> bool {
        self.min.x != Self::default().min.x
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new(Vec3::MAX, Vec3::MIN)
    }
}

impl Add<Vec3> for BoundingBox {
    type Output = Self;

    fn add(mut self, rhs: Vec3) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Vec3> for BoundingBox {
    fn add_assign(&mut self, rhs: Vec3) {
        self.min = self.min.min(rhs);
        self.max = self.max.max(rhs);
    }
}

impl FromIterator<Vec3> for BoundingBox {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Vec3>,
    {
        let mut this = Self::default();

        for item in iter {
            this += item;
        }

        this
    }
}

impl Add<Self> for BoundingBox {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Self> for BoundingBox {
    fn add_assign(&mut self, rhs: Self) {
        *self += rhs.min;
        *self += rhs.max;
    }
}

impl FromIterator<Self> for BoundingBox {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        let mut this = Self::default();

        for item in iter {
            this += item;
        }

        this
    }
}
