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
#[cfg(not(target_arch = "spirv"))]
use spirv_std::glam::Mat4;
use spirv_std::glam::{
    vec2, vec3, vec4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles,
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
pub use self::utils::*;
pub use self::world::*;

pub const MAX_LIGHTS: usize = 256;
pub const MAX_MATERIALS: usize = 256;

/// Contains ids of nodes yet to be visited, for the entire workgroup at once.
///
/// Usually this would be modelled as `let mut stack = [0; 16];`, but using
/// workgroup memory makes the code slightly faster.
pub type RayTraversingStack<'a> = &'a mut [usize; 16 * 8 * 8];

pub mod debug {
    /// Instead of rendering triangles, shows the BVH's bounding boxes and
    /// hierarchy.
    ///
    /// The brighter the color, the more nested given node is.
    pub const ENABLE_AABB: bool = false;
}
