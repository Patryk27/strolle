use bytemuck::{Pod, Zeroable};
use glam::{vec2, IVec2, Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::Ray;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Camera {
    pub projection_view: Mat4,
    pub ndc_to_world: Mat4,
    pub origin: Vec4,
    pub screen: Vec4,
}

impl Camera {
    /// Given a point in world-coordinates, returns it in clip-coordinates.
    pub fn world_to_clip(self, pos: Vec3) -> Vec4 {
        self.projection_view * pos.extend(1.0)
    }

    /// Given a point in world-coordinates, returns it in screen-coordinates.
    pub fn world_to_screen(self, pos: Vec3) -> Vec2 {
        self.clip_to_screen(self.world_to_clip(pos))
    }

    /// Given a point in clip-coordinates, returns it in screen-coordinates.
    pub fn clip_to_screen(self, pos: Vec4) -> Vec2 {
        let ndc = pos.xy() / pos.w;
        let ndc = vec2(ndc.x, -ndc.y);

        (0.5 * ndc + 0.5) * self.screen.xy()
    }

    /// Given a point in screen-coordinates, returns a unique index for it; used
    /// to index screen-space structures.
    pub fn screen_to_idx(self, pos: UVec2) -> usize {
        (pos.y * (self.screen.x as u32) + pos.x) as usize
    }

    /// Returns size of the camera's viewport in pixels.
    ///
    /// Note that camera's viewport's size might be different from the total
    /// window's size (e.g. user is free to create two separate cameras, each
    /// occupying half a screen - in that case this function will return that
    /// half-size).
    pub fn screen_size(self) -> UVec2 {
        self.screen.xy().as_uvec2()
    }

    /// Checks if given coordinates match camera's screen size and, if not,
    /// wraps them.
    ///
    /// See also: [`Self::contains()`].
    pub fn contain(self, mut pos: IVec2) -> UVec2 {
        let screen_size = self.screen.xy().as_ivec2();

        if pos.x < 0 {
            pos.x = -pos.x;
        }

        if pos.y < 0 {
            pos.y = -pos.y;
        }

        if pos.x >= screen_size.x {
            pos.x = screen_size.x - pos.x + screen_size.x - 1;
        }

        if pos.y >= screen_size.y {
            pos.y = screen_size.y - pos.y + screen_size.y - 1;
        }

        pos.as_uvec2()
    }

    /// Casts a ray from camera's center to given screen-coordinates.
    pub fn ray(self, screen_pos: UVec2) -> Ray {
        let screen_size = self.screen.xy();
        let screen_pos = screen_pos.as_vec2() + vec2(0.5, 0.5);

        let ndc = screen_pos * 2.0 / Vec2::new((screen_size.x).max(crate::STROLLE_EPSILON), (screen_size.y).max(crate::STROLLE_EPSILON)) - Vec2::ONE;
        let ndc = vec2(ndc.x, -ndc.y);

        let far_plane = self.ndc_to_world.project_point3(ndc.extend(crate::STROLLE_EPSILON));

        let near_plane = self.ndc_to_world.project_point3(ndc.extend(1.0));

        Ray::new(near_plane, crate::safe_normalize(far_plane - near_plane))
    }

    /// Returns camera's approximate origin, without taking into account the
    /// near-plane.
    ///
    /// Faster than `self.ray(...).origin()`, but somewhat less accurate.
    pub fn approx_origin(self) -> Vec3 {
        self.origin.xyz()
    }

    pub fn is_eq(self, rhs: Self) -> bool {
        self.projection_view
            .abs_diff_eq(rhs.projection_view, 0.0025)
    }
}

pub trait CameraContains<Rhs> {
    /// Returns whether given point lays inside the screen.
    ///
    /// See also: [`Camera::contain()`].
    fn contains(self, rhs: Rhs) -> bool;
}

impl CameraContains<UVec2> for Camera {
    fn contains(self, rhs: UVec2) -> bool {
        let screen_size = self.screen.xy().as_uvec2();

        rhs.x < screen_size.x && rhs.y < screen_size.y
    }
}

impl CameraContains<IVec2> for Camera {
    fn contains(self, rhs: IVec2) -> bool {
        let screen_size = self.screen.xy().as_ivec2();

        rhs.x >= 0
            && rhs.y >= 0
            && rhs.x < screen_size.x
            && rhs.y < screen_size.y
    }
}

impl CameraContains<Vec2> for Camera {
    fn contains(self, rhs: Vec2) -> bool {
        let screen_size = self.screen.xy();

        rhs.x >= 0.0
            && rhs.y >= 0.0
            && rhs.x < screen_size.x
            && rhs.y < screen_size.y
    }
}

#[cfg(test)]
mod tests {
    use glam::{ivec2, uvec2, vec4};

    use super::*;

    #[test]
    fn contain() {
        let target = Camera {
            projection_view: Default::default(),
            ndc_to_world: Default::default(),
            origin: Default::default(),
            screen: vec4(1024.0, 768.0, 0.0, 0.0),
        };

        // Case: minimum point inside the screen
        assert_eq!(target.contain(ivec2(0, 0)), uvec2(0, 0));

        // Case: point inside the screen
        assert_eq!(target.contain(ivec2(123, 456)), uvec2(123, 456));

        // Case: maximum point inside the screen
        assert_eq!(target.contain(ivec2(1023, 767)), uvec2(1023, 767));

        // Case: point outside the screen
        assert_eq!(target.contain(ivec2(1024, 768)), uvec2(1023, 767));
        assert_eq!(target.contain(ivec2(1025, 768)), uvec2(1022, 767));
        assert_eq!(target.contain(ivec2(1030, 768)), uvec2(1017, 767));
        assert_eq!(target.contain(ivec2(1030, 783)), uvec2(1017, 752));
    }
}
