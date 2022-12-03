#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]
#![no_std]

mod camera;
mod constants;
mod dynamic_geometry;
mod hit;
mod light;
mod lights;
mod material;
mod materials;
pub mod math;
mod ray;
mod static_geometry;
mod static_geometry_index;
mod triangle;
mod triangle_uv;
mod triangle_uvs;
mod utils;
mod world;

#[cfg(not(target_arch = "spirv"))]
use core::fmt;
use core::mem;

use bytemuck::{Pod, Zeroable};
pub use constants::*;
use spirv_std::glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::real::Real;
use spirv_std::{Image, Sampler};

pub use self::camera::*;
pub use self::dynamic_geometry::*;
pub use self::hit::*;
pub use self::light::*;
pub use self::lights::*;
pub use self::material::*;
pub use self::materials::*;
pub use self::ray::*;
pub use self::static_geometry::*;
pub use self::static_geometry_index::*;
pub use self::triangle::*;
pub use self::triangle_uv::*;
pub use self::triangle_uvs::*;
use self::utils::*;
pub use self::world::*;

// WebGL 2's limit
pub const MAX_BUFFER_BINDING_SIZE: usize = 65536;

pub const MAX_STATIC_TRIANGLES: usize = (MAX_BUFFER_BINDING_SIZE
    - mem::size_of::<PadU32>())
    / mem::size_of::<Triangle>();

pub const MAX_DYNAMIC_TRIANGLES: usize = 256;
pub const MAX_LIGHTS: usize = 64;
pub const MAX_MATERIALS: usize = 64;
pub const STATIC_GEOMETRY_INDEX_SIZE: usize = 4096;
