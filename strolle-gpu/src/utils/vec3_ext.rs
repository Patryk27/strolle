use glam::Vec3;

pub trait Vec3Ext
where
    Self: Sized,
{
    fn reflect(self, other: Self) -> Self;
}

impl Vec3Ext for Vec3 {
    fn reflect(self, other: Self) -> Self {
        self - 2.0 * other.dot(self) * other
    }
}
