//! Common structs, algorithms etc. used by Strolle's shaders and renderer.

#![no_std]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

mod atmosphere;
mod bvh_ptr;
mod bvh_view;
mod camera;
mod geometry;
mod hit;
mod light;
mod lights;
mod material;
mod materials;
mod noise;
mod normal;
mod passes;
mod ray;
mod reprojection;
mod reservoir;
mod triangle;
mod triangles;
mod utils;
mod world;

pub use self::atmosphere::*;
pub use self::bvh_ptr::*;
pub use self::bvh_view::*;
pub use self::camera::*;
pub use self::geometry::*;
pub use self::hit::*;
pub use self::light::*;
pub use self::lights::*;
pub use self::material::*;
pub use self::materials::*;
pub use self::noise::*;
pub use self::normal::*;
pub use self::passes::*;
pub use self::ray::*;
pub use self::reprojection::*;
pub use self::reservoir::*;
pub use self::triangle::*;
pub use self::triangles::*;
pub use self::utils::*;
pub use self::world::*;

/// Stack of nodes yet-to-be-visited when traversing the BVH.
///
/// For performance reasons, we use a per-workgroup shared memory array where
/// each workgroup-thread simply indexes into a different slice of this memory.
pub type BvhTraversingStack<'a> = &'a mut [u32; BVH_STACK_SIZE * 8 * 8];

/// Maximum stack size per each workgroup-thread.
///
/// The larger this value is, the bigger world can be rendered - at the expense
/// of performance.
pub const BVH_STACK_SIZE: usize = 24;
