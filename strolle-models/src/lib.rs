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

/// Stack of nodes yet-to-be-visited when traversing the BVH.
///
/// For performance reasons, we use a per-workgroup shared memory array where
/// each workgroup-thread simply indexes into a different slice of this memory.
pub type BvhTraversingStack<'a> = &'a mut [u32; BVH_STACK_SIZE * 8 * 8];

/// Maximum stack size per each workgroup-thread.
///
/// The larger this value is, the bigger world (in terms of BVH nodes) can be
/// rendered - at the expense of performance.
pub const BVH_STACK_SIZE: usize = 32;

/// Maximum number of user-provided textures.
///
/// TODO After https://github.com/gfx-rs/wgpu/issues/3334 is implemented, this
///      limit could be (probably) raised
pub const MAX_IMAGES: usize = 16;

/// Maximum number of items in the voxel-map.
///
/// The larger this value is, the less chance of voxel-collision - at the
/// expense of memory (each voxel takes 2 * 4 * 4 = 32 bytes).
///
/// TODO could be configurable
pub const VOXELS_MAP_LENGTH: usize = 16 * 1024 * 1024;

/// Size of each voxel, in world-space terms.
///
/// The lower this value is, the better indirect illumination can be
/// approximated - at the expense of performance and increased likelyhood of
/// voxel-collisions.
///
/// TODO could be configurable
pub const VOXEL_SIZE: f32 = 0.1;
