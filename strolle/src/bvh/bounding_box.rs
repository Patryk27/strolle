use std::ops::Add;

use spirv_std::glam::{vec3, Mat4, Vec3};

use super::bvh_object::BvhObject;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    min: Option<Vec3>,
    max: Option<Vec3>,
}

impl BoundingBox {
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Self {
        points.into_iter().fold(Self::default(), Self::add)
    }

    pub fn from_objects<T>(objects: impl IntoIterator<Item = T>) -> Self
    where
        T: BvhObject,
    {
        objects
            .into_iter()
            .map(|object| object.bounding_box())
            .fold(Self::default(), Self::add)
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

    pub fn transform(&self, mat: Mat4) -> Self {
        let mut out = Self::default();
        let min = self.min();
        let max = self.max();

        for i in 0..8 {
            let point = {
                let x = if i & 1 == 0 { min.x } else { max.x };
                let y = if i & 2 == 0 { min.y } else { max.y };
                let z = if i & 4 == 0 { min.z } else { max.z };

                vec3(x, y, z)
            };

            out.grow(mat.transform_point3(point));
        }

        out
    }

    pub fn min(&self) -> Vec3 {
        self.min.expect("Bounding box is empty")
    }

    pub fn max(&self) -> Vec3 {
        self.max.expect("Bounding box is empty")
    }

    pub fn center(&self) -> Vec3 {
        (self.min() + self.max()) / 2.0
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
            if n.is_nan() {
                // This can happen if our extent is a 2D (e.g. a plane) - in
                // that case it doesn't matter what value gets assigned here,
                // since all of the vectors will get the same value
                *n = 0.0;
            }

            if *n < 0.0 && *n > -0.001 {
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
