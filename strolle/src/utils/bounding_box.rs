use std::ops::{Add, AddAssign};

use glam::{vec3, Affine3A};
use spirv_std::glam::Vec3;

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
        if !self.is_set() {
            return f32::MAX;
        }

        let extent = self.extent();

        extent.x * extent.y + extent.y * extent.z + extent.z * extent.x
    }

    pub fn with_transform(&self, transform: Affine3A) -> Self {
        (0..8)
            .map(|i| {
                let point = vec3(
                    if i & 1 > 0 { self.max.x } else { self.min.x },
                    if i & 2 > 0 { self.max.y } else { self.min.y },
                    if i & 4 > 0 { self.max.z } else { self.min.z },
                );

                transform.transform_point3(point)
            })
            .collect()
    }

    pub fn is_set(&self) -> bool {
        self.min.x != Self::default().min.x
    }

    /// Maps `p` from `self.min() ..= self.max()` to `0.0 ..= 1.0`.
    pub fn map(&self, mut p: Vec3) -> Vec3 {
        p = (p - self.min()) / self.extent();

        // This can happen if our extent is a 2D (e.g. a plane) - in that case
        // it doesn't matter which particular x/y/z gets assigned here, since
        // all of the vectors will get the same value:

        if p.x.is_nan() {
            p.x = 0.0;
        }

        if p.y.is_nan() {
            p.y = 0.0;
        }

        if p.z.is_nan() {
            p.z = 0.0;
        }

        p.clamp(Vec3::ZERO, Vec3::ONE)
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
