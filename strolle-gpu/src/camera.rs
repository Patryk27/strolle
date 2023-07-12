use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec3, IVec2, Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::Ray;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Camera {
    pub projection_view: Mat4,
    pub origin: Vec4,
    pub viewport: Vec4,
    pub onb_u: Vec4,
    pub onb_v: Vec4,
    pub onb_w: Vec4,
}

impl Camera {
    /// Transforms point from world-coordinates into clip-coordinates.
    pub fn world_to_clip(&self, pos: Vec3) -> Vec4 {
        self.projection_view * pos.extend(1.0)
    }

    /// Transforms point from world-coordinates into screen-coordinates.
    pub fn world_to_screen(&self, pos: Vec3) -> Vec2 {
        self.clip_to_screen(self.world_to_clip(pos))
    }

    /// Transforms point from clip-coordinates into screen-coordinates.
    pub fn clip_to_screen(&self, pos: Vec4) -> Vec2 {
        let ndc = pos.xy() / pos.w;
        let ndc = vec2(ndc.x, -ndc.y);

        (0.5 * ndc + 0.5) * self.viewport_size().as_vec2()
    }

    /// Transforms point from screen-coordinates into a unique index (as long as
    /// the given point is within the viewport size).
    ///
    /// Used to index screen-space structures.
    pub fn screen_to_idx(&self, pos: UVec2) -> usize {
        (pos.y * self.viewport_size().x + pos.x) as usize
    }

    pub fn contains(&self, pos: IVec2) -> bool {
        let viewport_size = self.viewport_size().as_ivec2();

        pos.x >= 0
            && pos.y >= 0
            && pos.x < viewport_size.x
            && pos.y < viewport_size.y
    }

    pub fn ray(&self, screen_pos: UVec2) -> Ray {
        let origin = self.origin.xyz();

        let direction = {
            // Map from viewport's size to 0..1
            let pos = screen_pos.as_vec2() / self.viewport_size().as_vec2();

            // Map to -1..1
            let pos = 2.0 * pos - 1.0;
            let pos = vec2(pos.x, -pos.y);

            // Adjust for aspect ratio
            let pos = vec2(pos.x / self.viewport_aspect_ratio(), pos.y);

            // Adjust for the field of view
            let pos = pos * (self.fov() / 2.0).tan();

            OrthonormalBasis::trace(
                self.onb_u,
                self.onb_v,
                self.onb_w,
                vec3(pos.x, pos.y, -1.0),
            )
            .xyz()
        };

        Ray::new(origin, direction)
    }

    pub fn origin(&self) -> Vec3 {
        self.origin.xyz()
    }

    pub fn fov(&self) -> f32 {
        self.origin.w
    }

    pub fn viewport_position(&self) -> Vec2 {
        self.viewport.xy()
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.viewport.zw().as_uvec2()
    }

    pub fn viewport_aspect_ratio(&self) -> f32 {
        let size = self.viewport_size().as_vec2();

        size.y / size.x
    }
}

// Thanks to https://4programmers.net/Z_pogranicza/Raytracing
pub struct OrthonormalBasis;

impl OrthonormalBasis {
    pub fn build(origin: Vec3, look_at: Vec3, up: Vec3) -> (Vec4, Vec4, Vec4) {
        let w = (origin - look_at).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);

        (u.extend(0.0), v.extend(0.0), w.extend(0.0))
    }

    pub fn trace(u: Vec4, v: Vec4, w: Vec4, vec: Vec3) -> Vec4 {
        u * vec.x + v * vec.y + w * vec.z
    }
}
