use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec4, Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::float::Float;

use crate::Ray;

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Camera {
    projection_view: Mat4,
    origin: Vec4,
    viewport: Vec4,
    onb_u: Vec4,
    onb_v: Vec4,
    onb_w: Vec4,
    clear_color: Vec4,
}

impl Camera {
    pub fn new(
        projection_view: Mat4,
        origin: Vec3,
        look_at: Vec3,
        up: Vec3,
        fov: f32,
        viewport_position: UVec2,
        viewport_size: UVec2,
        clear_color: Vec3,
    ) -> Self {
        let (onb_u, onb_v, onb_w) =
            OrthonormalBasis::build(origin, look_at, up);

        Self {
            projection_view,
            origin: origin.extend(fov),
            viewport: viewport_position
                .as_vec2()
                .extend(viewport_size.x as f32)
                .extend(viewport_size.y as f32),
            onb_u,
            onb_v,
            onb_w,
            clear_color: clear_color.extend(0.0),
        }
    }

    pub fn project(&self, position: Vec3) -> Vec4 {
        self.projection_view * position.extend(1.0)
    }

    pub fn ray(&self, image_xy: UVec2) -> Ray {
        let origin = self.origin.xyz();

        let direction = {
            // Map from viewport's size to 0..1
            let pos = image_xy.as_vec2() / self.viewport_size().as_vec2();

            // Map to -1..1
            let pos = 2.0 * pos - 1.0;

            // Map to 1..-1
            let pos = vec2(pos.x, -pos.y);

            // Adjust for aspect ratio
            let pos = vec2(pos.x / self.viewport_aspect_ratio(), pos.y);

            // Adjust for the field of view
            let pos = pos * (self.fov() / 2.0).tan();

            OrthonormalBasis::trace(
                self.onb_u,
                self.onb_v,
                self.onb_w,
                vec4(pos.x, pos.y, -1.0, 0.0),
            )
            .xyz()
        };

        Ray::new(origin, direction)
    }

    pub fn origin(&self) -> Vec3 {
        self.origin.xyz()
    }

    pub fn clear_color(&self) -> Vec3 {
        self.clear_color.xyz()
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
struct OrthonormalBasis;

impl OrthonormalBasis {
    fn build(origin: Vec3, look_at: Vec3, up: Vec3) -> (Vec4, Vec4, Vec4) {
        let w = (origin - look_at).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);

        (u.extend(0.0), v.extend(0.0), w.extend(0.0))
    }

    fn trace(u: Vec4, v: Vec4, w: Vec4, vec: Vec4) -> Vec4 {
        u * vec.x + v * vec.y + w * vec.z
    }
}
