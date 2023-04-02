#![no_std]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

mod bvh_ptr;
mod bvh_view;
mod camera;
mod hit;
mod light;
mod lights;
mod material;
mod materials;
mod mesh;
mod noise;
mod passes;
mod pending_voxel;
mod pending_voxel_hit;
mod pending_voxel_hits;
mod pending_voxels;
mod ray;
mod triangle;
mod triangles;
mod voxel;
mod voxels;
mod world;

pub use self::bvh_ptr::*;
pub use self::bvh_view::*;
pub use self::camera::*;
pub use self::hit::*;
pub use self::light::*;
pub use self::lights::*;
pub use self::material::*;
pub use self::materials::*;
pub use self::mesh::*;
pub use self::noise::*;
pub use self::passes::*;
pub use self::pending_voxel::*;
pub use self::pending_voxel_hit::*;
pub use self::pending_voxel_hits::*;
pub use self::pending_voxels::*;
pub use self::ray::*;
pub use self::triangle::*;
pub use self::triangles::*;
pub use self::voxel::*;
pub use self::voxels::*;
pub use self::world::*;

/// Contains ids of nodes yet to be visited, for the entire workgroup at once.
///
/// Usually this would be modeled as `let mut stack = [0; ...];`, but using
/// workgroup memory makes the code run slightly faster.
pub type BvhTraversingStack<'a> = &'a mut [u32; 32 * 8 * 8];

/// Maximum number of images (aka textures).
///
/// TODO
pub const MAX_IMAGES: usize = 10;

// TODO
pub const VOXELS_MAP_LENGTH: usize = 32 * 1024 * 1024;

// TODO
pub const VOXEL_SIZE: f32 = 0.1;
