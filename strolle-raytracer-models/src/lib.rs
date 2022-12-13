#![no_std]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

mod camera;
mod geometry_bvh_view;
mod geometry_tris_view;
mod geometry_uvs_view;
mod hit;
mod light;
mod lights;
mod material;
mod materials;
mod ray;
mod triangle;
mod triangle_uv;
mod utils;
mod world;

#[cfg(not(target_arch = "spirv"))]
use core::fmt;

use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{
    vec2, vec3, vec4, Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles,
};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;

pub use self::camera::*;
pub use self::geometry_bvh_view::*;
pub use self::geometry_tris_view::*;
pub use self::geometry_uvs_view::*;
pub use self::hit::*;
pub use self::light::*;
pub use self::lights::*;
pub use self::material::*;
pub use self::materials::*;
pub use self::ray::*;
pub use self::triangle::*;
pub use self::triangle_uv::*;
use self::utils::*;
pub use self::world::*;

pub const MAX_LIGHTS: usize = 256;
pub const MAX_MATERIALS: usize = 256;
