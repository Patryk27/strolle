mod axis;
mod bounding_box;
mod bvh;
mod roped_bvh;
mod serializer;

use std::fmt;
use std::ops::{Index, IndexMut};

use instant::{Duration, Instant};
use spirv_std::glam::{vec4, Vec3};

use self::axis::*;
use self::bounding_box::*;
use self::bvh::*;
use self::roped_bvh::*;
use crate::{StaticGeometry, StaticGeometryIndex, StaticTriangle, Triangle};

type TriangleId = crate::TriangleId<StaticTriangle>;

/// An BVH (SAH-based) geometry indexer.
///
/// Special thanks to:
/// - https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/,
/// - https://github.com/svenstaro/bvh.
#[derive(Default)]
pub struct GeometryIndexer;

impl GeometryIndexer {
    pub fn index(
        geometry: &StaticGeometry,
    ) -> Option<Box<StaticGeometryIndex>> {
        let len = geometry.iter().count();

        log::info!("Indexing geometry; triangles = {}", len);

        let (bvh, tt_bvh) = Self::measure(|| Bvh::build(geometry));
        let (rbvh, tt_rbvh) = Self::measure(|| RopedBvh::build(bvh));

        let ((index, index_len), tt_serialize) =
            Self::measure(|| serializer::serialize(rbvh));

        log::info!(
            "Geometry indexed; tt-bvh = {:?}, tt-rbvh = {:?}, tt-serialize = {:?}, index-size = {}",
            tt_bvh,
            tt_rbvh,
            tt_serialize,
            index_len,
        );

        Some(Box::new(index))
    }

    fn measure<T>(f: impl FnOnce() -> T) -> (T, Duration) {
        let tt = Instant::now();
        let val = f();

        (val, tt.elapsed())
    }
}
