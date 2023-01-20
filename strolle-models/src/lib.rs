#![no_std]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

mod bvh_ptr;
mod bvh_view;
mod camera;
mod drawing_pass_params;
mod hit;
mod light;
mod lights_view;
mod material;
mod materials_view;
mod mesh;
mod noise;
mod ray;
mod ray_pass_params;
mod triangle;
mod triangles_view;
mod world;

pub use self::bvh_ptr::*;
pub use self::bvh_view::*;
pub use self::camera::*;
pub use self::drawing_pass_params::*;
pub use self::hit::*;
pub use self::light::*;
pub use self::lights_view::*;
pub use self::material::*;
pub use self::materials_view::*;
pub use self::mesh::*;
pub use self::noise::*;
pub use self::ray::*;
pub use self::ray_pass_params::*;
pub use self::triangle::*;
pub use self::triangles_view::*;
pub use self::world::*;

/// Contains ids of nodes yet to be visited, for the entire workgroup at once.
///
/// Usually this would be modeled as `let mut stack = [0; ...];`, but using
/// workgroup memory makes the code run slightly faster.
pub type BvhTraversingStack<'a> = &'a mut [u32; 32 * 8 * 8];

/// Maximum number of images (aka textures).
///
/// The lowest common denominator here is Metal with its limit of 16.
pub const MAX_IMAGES: usize = 16;
