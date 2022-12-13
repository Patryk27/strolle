use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct Camera {
    origin: Vec4,
    viewport: Vec4,
    onb_u: Vec4,
    onb_v: Vec4,
    onb_w: Vec4,
    clear_color: Vec4,
}

impl Camera {
    pub fn ray(&self, pos: Vec2) -> Ray {
        let origin = self.origin.xyz();

        let direction = {
            let viewport_fov = self.viewport.z;
            let viewport_aspect_ratio = self.viewport.y / self.viewport.x;

            // Map from viewport's size to 0..1
            let pos = pos / self.viewport.xy();

            // Map to -1..1
            let pos = 2.0 * pos - 1.0;

            // Map to 1..-1
            let pos = vec2(pos.x, -pos.y);

            // Adjust for aspect ratio
            let pos = vec2(pos.x / viewport_aspect_ratio, pos.y);

            // Adjust for the field of view
            let pos = pos * (viewport_fov / 2.0).tan();

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

    pub fn clear_color(&self) -> Vec3 {
        self.clear_color.xyz()
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.viewport.xy().as_uvec2()
    }
}

impl Camera {
    pub fn new(
        origin: Vec3,
        look_at: Vec3,
        up: Vec3,
        viewport_size: UVec2,
        viewport_fov: f32,
        clear_color: Vec3,
    ) -> Self {
        let (onb_u, onb_v, onb_w) =
            OrthonormalBasis::build(origin, look_at, up);

        Self {
            origin: origin.extend(0.0),
            viewport: viewport_size.as_vec2().extend(viewport_fov).extend(0.0),
            onb_u,
            onb_v,
            onb_w,
            clear_color: clear_color.extend(0.0),
        }
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
