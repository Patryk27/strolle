use std::mem;
use std::ops::Range;

use super::{BvhPrimitive, BvhPrimitiveId, BvhPrimitivesRef};

#[derive(Debug, Default)]
pub struct BvhPrimitives {
    all: Vec<BvhPrimitive>,
    current: Vec<BvhPrimitive>,
    previous: Vec<BvhPrimitive>,
}

impl BvhPrimitives {
    pub fn add(&mut self, prim: BvhPrimitive) {
        self.all.push(prim);
    }

    pub fn update(
        &mut self,
        ids: Range<usize>,
    ) -> impl Iterator<Item = &mut BvhPrimitive> {
        self.all[ids].iter_mut()
    }

    pub fn current_ref(&self) -> BvhPrimitivesRef {
        BvhPrimitivesRef::new(
            BvhPrimitiveId::new(0),
            BvhPrimitiveId::new(self.current.len() as u32),
        )
    }

    pub fn current(&self, range: BvhPrimitivesRef) -> &[BvhPrimitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &self.current[start..end]
    }

    pub fn current_mut(
        &mut self,
        range: BvhPrimitivesRef,
    ) -> &mut [BvhPrimitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &mut self.current[start..end]
    }

    pub fn copy_previous_to_current(
        &mut self,
        previous: BvhPrimitivesRef,
        current: BvhPrimitivesRef,
    ) {
        self.current[current.as_range()]
            .copy_from_slice(&self.previous[previous.as_range()]);
    }

    pub fn begin_refresh(&mut self) {
        self.current =
            self.all.iter().filter(|p| p.is_alive()).copied().collect();
    }

    pub fn end_refresh(&mut self) {
        self.previous = mem::take(&mut self.current);
    }
}
