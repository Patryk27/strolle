use glam::{ivec2, vec3, UVec2, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{BilinearFilter, Camera, ReprojectionMap, SurfaceMap, TexRgba16f};

pub struct TemporalDenoiser<'a> {
    pub camera: &'a Camera,
    pub reprojection_map: ReprojectionMap<'a>,
    pub surface_map: SurfaceMap<'a>,
    pub samples: TexRgba16f<'a>,
    pub image: TexRgba16f<'a>,
    pub prev_image: TexRgba16f<'a>,
}

impl<'a> TemporalDenoiser<'a> {
    /// Applies temporal denoising on given textures.
    ///
    /// Thanks to:
    ///
    /// - https://www.shadertoy.com/view/4tcXD2
    ///   (Path tracer + Temporal AA by yvtjp)
    ///
    /// - https://de45xmedrsdbp.cloudfront.net/Resources/files/TemporalAA_small-59732822.pdf
    ///   (High Quality Temporal Supersampling by Brian Karis)
    pub fn run(self, screen_pos: UVec2) {
        let reprojection = self.reprojection_map.get(screen_pos);

        let prev_color = if reprojection.is_some() {
            BilinearFilter::reproject(reprojection, move |pos| {
                (self.prev_image.read(pos), 1.0)
            })
        } else {
            Default::default()
        };

        let mut color = prev_color.xyz();
        let mix_rate = prev_color.w.min(0.5);
        let in0 = self.samples.read(screen_pos).xyz();

        color = (color * color).lerp(in0 * in0, mix_rate);
        color.x = color.x.sqrt();
        color.y = color.y.sqrt();
        color.z = color.z.sqrt();

        // -------------------------------------------------------------------------

        let surface = self.surface_map.get(screen_pos);

        let neighbour = move |dx: i32, dy: i32| {
            let pos = screen_pos.as_ivec2() + ivec2(dx, dy);

            if !self.camera.contains(pos) {
                // If our neighbour is outside of the viewport, reject the
                // sample.
                //
                // Note that instead of returning `Vec3::ZERO`, we return the
                // center sample since otherwise we could unnecessarily darken
                // corners or pixels nearby complex surfaces.
                return in0;
            }

            let pos = pos.as_uvec2();

            if self.surface_map.get(pos).evaluate_similarity_to(&surface) < 0.33
            {
                // If our neighbour is too different from us geometrically (e.g.
                // has normal pointing towards a very different direction),
                // reject the sample.
                return in0;
            }

            self.samples.read(pos).xyz()
        };

        // TODO optimization opportunity: preload neighbours into shared memory
        let in1 = neighbour(1, 0);
        let in2 = neighbour(-1, 0);
        let in3 = neighbour(0, 1);
        let in4 = neighbour(0, -1);
        let in5 = neighbour(1, 1);
        let in6 = neighbour(-1, 1);
        let in7 = neighbour(1, -1);
        let in8 = neighbour(-1, -1);

        let color = encode_pal_yuv(color);
        let in0 = encode_pal_yuv(in0);
        let in1 = encode_pal_yuv(in1);
        let in2 = encode_pal_yuv(in2);
        let in3 = encode_pal_yuv(in3);
        let in4 = encode_pal_yuv(in4);
        let in5 = encode_pal_yuv(in5);
        let in6 = encode_pal_yuv(in6);
        let in7 = encode_pal_yuv(in7);
        let in8 = encode_pal_yuv(in8);

        // -------------------------------------------------------------------------

        let min = |x: Vec3, y: Vec3| x.min(y);
        let max = |x: Vec3, y: Vec3| x.max(y);
        let mix = |x: Vec3, y: Vec3, v| x.lerp(y, v);

        let mut min_color = min(min(min(in0, in1), min(in2, in3)), in4);
        let mut max_color = max(max(max(in0, in1), max(in2, in3)), in4);

        min_color = mix(
            min_color,
            min(min(min(in5, in6), min(in7, in8)), min_color),
            0.5,
        );

        max_color = mix(
            max_color,
            max(max(max(in5, in6), max(in7, in8)), max_color),
            0.5,
        );

        // -------------------------------------------------------------------------

        let color_before_clamping = color;
        let color = color.clamp(min_color, max_color);

        let clamping = (color - color_before_clamping).length_squared();

        let mix_rate = 1.0 / (1.0 / mix_rate + 1.0);
        let mix_rate = mix_rate + clamping * 4.0;
        let mix_rate = mix_rate.clamp(0.05, 0.5);

        let out = decode_pal_yuv(color).extend(mix_rate);

        unsafe {
            self.image.write(screen_pos, out);
        }
    }
}

fn encode_pal_yuv(mut rgb: Vec3) -> Vec3 {
    rgb = rgb.powf(2.2);

    vec3(
        rgb.dot(vec3(0.299, 0.587, 0.114)),
        rgb.dot(vec3(-0.14713, -0.28886, 0.436)),
        rgb.dot(vec3(0.615, -0.51499, -0.10001)),
    )
}

fn decode_pal_yuv(yuv: Vec3) -> Vec3 {
    let rgb = vec3(
        yuv.dot(vec3(1.0, 0., 1.13983)),
        yuv.dot(vec3(1.0, -0.39465, -0.58060)),
        yuv.dot(vec3(1.0, 2.03211, 0.0)),
    );

    rgb.powf(1.0 / 2.2)
}
