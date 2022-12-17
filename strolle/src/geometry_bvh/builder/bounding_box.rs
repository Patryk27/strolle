use std::ops::Add;

use spirv_std::glam::Vec3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    min: Option<Vec3>,
    max: Option<Vec3>,
}

impl BoundingBox {
    pub fn grow(&mut self, p: Vec3) {
        if let Some(min) = &mut self.min {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
        } else {
            self.min = Some(p);
        }

        if let Some(max) = &mut self.max {
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        } else {
            self.max = Some(p);
        }
    }

    pub fn with(mut self, p: Vec3) -> Self {
        self.grow(p);
        self
    }

    pub fn min(&self) -> Vec3 {
        self.min.unwrap()
    }

    pub fn max(&self) -> Vec3 {
        self.max.unwrap()
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
