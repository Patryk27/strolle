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
    pub data: Vec4,
}

impl Camera {
    /// Given a point in world-coordinates, returns it in clip-coordinates.
    pub fn world_to_clip(&self, pos: Vec3) -> Vec4 {
        self.projection_view * pos.extend(1.0)
    }

    /// Given a point in world-coordinates, returns it in screen-coordinates.
    pub fn world_to_screen(&self, pos: Vec3) -> Vec2 {
        self.clip_to_screen(self.world_to_clip(pos))
    }

    /// Given a point in clip-coordinates, returns it in screen-coordinates.
    pub fn clip_to_screen(&self, pos: Vec4) -> Vec2 {
        let ndc = pos.xy() / pos.w;
        let ndc = vec2(ndc.x, -ndc.y);

        (0.5 * ndc + 0.5) * self.screen.xy()
    }

    /// Given a point in screen-coordinates, returns a unique index for it; used
    /// to index screen-space structures.
    pub fn screen_to_idx(&self, pos: UVec2) -> usize {
        (pos.y * (self.screen.x as u32) + pos.x) as usize
    }

    /// Returns size of the camera's viewport in pixels.
    ///
    /// Note that camera's viewport's size might be different from the total
    /// window's size (e.g. user is free to create two separate cameras, each
    /// occupying half a screen - in that case this function will return that
    /// half-size).
    pub fn screen_size(&self) -> UVec2 {
        self.screen.xy().as_uvec2()
    }

    /// Checks if given coordinates match camera's screen size and, if not,
    /// wraps them.
    ///
    /// See also: [`Self::contains()`].
    pub fn contain(&self, mut pos: IVec2) -> UVec2 {
        let screen_size = self.screen.xy().as_ivec2();

        if pos.x < 0 {
            pos.x = -pos.x;
        }

        if pos.y < 0 {
            pos.y = -pos.y;
        }

        if pos.x >= screen_size.x {
            pos.x = screen_size.x - pos.x + screen_size.x;
        }

        if pos.y >= screen_size.y {
            pos.y = screen_size.y - pos.y + screen_size.y;
        }

        pos.as_uvec2()
    }

    /// Returns whether given point lays inside the screen.
    ///
    /// See also: [`Self::contain()`].
    pub fn contains(&self, pos: IVec2) -> bool {
        let screen_size = self.screen.xy().as_ivec2();

        pos.x >= 0
            && pos.y >= 0
            && pos.x < screen_size.x
            && pos.y < screen_size.y
    }

    /// Casts a ray from camera's center to given screen-coordinates.
    pub fn ray(&self, screen_pos: UVec2) -> Ray {
        let screen_size = self.screen.xy();
        let screen_pos = screen_pos.as_vec2() + vec2(0.5, 0.5);

        let ndc = screen_pos * 2.0 / screen_size - Vec2::ONE;
        let ndc = vec2(ndc.x, -ndc.y);

        let far_plane =
            self.ndc_to_world.project_point3(ndc.extend(f32::EPSILON));

        let near_plane = self.ndc_to_world.project_point3(ndc.extend(1.0));

        Ray::new(near_plane, (far_plane - near_plane).normalize())
    }

    /// Returns camera's approximate origin, without taking into account the
    /// near-plane.
    ///
    /// Faster than `self.ray(...).origin()`, but somewhat less accurate.
    pub fn approx_origin(&self) -> Vec3 {
        self.origin.xyz()
    }

    pub fn mode(&self) -> u32 {
        self.data.x.to_bits()
    }

    pub fn is_eq(&self, rhs: &Self) -> bool {
        if !self
            .projection_view
            .abs_diff_eq(rhs.projection_view, 0.0025)
        {
            return false;
        }

        if self.mode() != rhs.mode() {
            return false;
        }

        true
    }
}
