use std::ops::Add;

use spirv_std::glam::Vec3;

use crate::GeometryTris;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    min: Option<Vec3>,
    max: Option<Vec3>,
}

impl BoundingBox {
    pub fn for_scene(scene: &GeometryTris) -> Self {
        scene
            .iter()
            .flat_map(|(_, tri)| tri.vertices())
            .fold(Self::default(), Self::with)
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

    pub fn with(mut self, p: Vec3) -> Self {
        self.grow(p);
        self
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

        p
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
