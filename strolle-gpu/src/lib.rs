//! Common structs, algorithms etc. used by Strolle's shaders and renderer.

#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::manual_range_contains)]

mod atmosphere;
mod brdf;
mod bvh_view;
mod camera;
mod gbuffer;
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
mod surface;
mod triangle;
mod triangles;
mod utils;
mod world;

pub use self::atmosphere::*;
pub use self::brdf::*;
pub use self::bvh_view::*;
pub use self::camera::*;
pub use self::gbuffer::*;
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
pub use self::surface::*;
pub use self::triangle::*;
pub use self::triangles::*;
pub use self::utils::*;
pub use self::world::*;

pub mod prelude {
    pub use core::f32::consts::PI;

    pub use spirv_std::arch::IndexUnchecked;
    pub use spirv_std::glam::*;
    #[cfg(target_arch = "spirv")]
    pub use spirv_std::num_traits::Float;
    pub use spirv_std::{spirv, Image, Sampler};

    pub use crate::*;
}

/// Stack for nodes yet-to-be-visited when traversing the BVH.
///
/// For performance reasons, we use a per-workgroup shared-memory array where
/// each workgroup-thread simply indexes into a different slice of this memory.
pub type BvhStack<'a> = &'a mut [u32; BVH_STACK_SIZE * 8 * 8];

/// Maximum stack size per each workgroup-thread when traversing the BVH.
///
/// Affects the maximum size of BVH tree (it must not grow larger than
/// `2 ^ BVH_STACK_SIZE`).
pub const BVH_STACK_SIZE: usize = 24;

/// Golden angle, used for spatial filters.
pub const GOLDEN_ANGLE: f32 = 2.39996;
