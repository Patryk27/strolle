use spirv_std::glam::Vec3;

use crate::bvh::{BoundingBox, BvhNodePayload};

pub trait BvhObject {
    fn payload(&self) -> BvhNodePayload;
    fn bounding_box(&self) -> BoundingBox;
    fn center(&self) -> Vec3;
}

impl<T> BvhObject for &T
where
    T: BvhObject,
{
    fn payload(&self) -> BvhNodePayload {
        T::payload(self)
    }

    fn bounding_box(&self) -> BoundingBox {
        T::bounding_box(self)
    }

    fn center(&self) -> Vec3 {
        T::center(self)
    }
}
