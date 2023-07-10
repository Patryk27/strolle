use glam::{UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{Camera, MaterialId, Normal, Ray, TexRgba32f};

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: MaterialId,
}

impl Hit {
    /// How far to move a hit point away from its surface to avoid
    /// self-intersection when casting shadow rays.
    ///
    /// This constant cannot be zero (because then every object would cast
    /// shadows onto itself), but it cannot be too high either (because then
    /// shadows would feel off, flying on surfaces instead of being attached to
    /// them).
    pub const NUDGE_OFFSET: f32 = 0.001;

    pub fn none() -> Self {
        Self {
            distance: f32::MAX,
            point: Default::default(),
            normal: Default::default(),
            uv: Default::default(),
            material_id: MaterialId::new(0),
        }
    }

    pub fn is_some(&self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn serialize(&self) -> [Vec4; 2] {
        let d0 = self.point.extend(f32::from_bits(self.material_id.get()));

        let d1 = Normal::encode(self.normal)
            .extend(self.uv.x)
            .extend(self.uv.y);

        [d0, d1]
    }

    pub fn deserialize(d0: Vec4, d1: Vec4) -> Self {
        if d0.xyz() == Default::default() {
            Self::none()
        } else {
            let normal = Normal::decode(d1.xy());
            let point = d0.xyz() + normal * Self::NUDGE_OFFSET;

            Self {
                distance: 0.0,
                point,
                normal,
                uv: d1.zw(),
                material_id: MaterialId::new(d0.w.to_bits()),
            }
        }
    }

    pub fn deserialize_point(d0: Vec4) -> Vec3 {
        d0.xyz()
    }

    /// Gets the direct hit from opaque surface at given screen-coordinates.
    ///
    /// That is, this function returns the primary hit if the primary surface
    /// (at given screen-coordinates) is opaque, or the secondary hit otherwise.
    pub fn find_direct(
        camera: &Camera,
        direct_primary_hits_d0: TexRgba32f,
        direct_primary_hits_d1: TexRgba32f,
        direct_secondary_rays: TexRgba32f,
        direct_secondary_hits_d0: TexRgba32f,
        direct_secondary_hits_d1: TexRgba32f,
        screen_pos: UVec2,
    ) -> (Ray, Self) {
        let secondary_ray = direct_secondary_rays.read(screen_pos);

        if secondary_ray == Default::default() {
            let ray = camera.ray(screen_pos);

            let hit = Hit::deserialize(
                direct_primary_hits_d0.read(screen_pos),
                direct_primary_hits_d1.read(screen_pos),
            );

            (ray, hit)
        } else {
            let ray = Ray::new(
                Hit::deserialize_point(direct_primary_hits_d0.read(screen_pos)),
                secondary_ray.xyz(),
            );

            let hit = Hit::deserialize(
                direct_secondary_hits_d0.read(screen_pos),
                direct_secondary_hits_d1.read(screen_pos),
            );

            (ray, hit)
        }
    }
}
