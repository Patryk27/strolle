#![no_std]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

mod bvh_ptr;
mod bvh_view;
mod camera;
mod hit;
mod info;
mod instance;
mod instances_view;
mod light;
mod lights_view;
mod material;
mod materials_view;
mod mesh;
mod ray;
mod triangle;
mod triangles_view;
mod world;

pub use self::bvh_ptr::*;
pub use self::bvh_view::*;
pub use self::camera::*;
pub use self::hit::*;
pub use self::info::*;
pub use self::instance::*;
pub use self::instances_view::*;
pub use self::light::*;
pub use self::lights_view::*;
pub use self::material::*;
pub use self::materials_view::*;
pub use self::mesh::*;
pub use self::ray::*;
pub use self::triangle::*;
pub use self::triangles_view::*;
pub use self::world::*;

/// Contains ids of nodes yet to be visited, for the entire workgroup at once.
///
/// Usually this would be modeled as `let mut stack = [0; ...];`, but using
/// workgroup memory makes the code run slightly faster.
pub type BvhTraversingStack<'a> = &'a mut [u32; 32 * 8 * 8];

pub mod debug {
    /// Instead of rendering triangles, shows BVH's bounding boxes representing
    /// scene hierarchy; the brighter the color, the more nested given node is.
    pub const ENABLE_AABB: bool = false;
}
