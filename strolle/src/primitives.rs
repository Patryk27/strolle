use std::collections::HashMap;
use std::mem;
use std::ops::Range;

use super::{Primitive, PrimitiveId, PrimitivesRef};
use crate::utils::Allocator;
use crate::Params;

#[derive(Debug)]
pub struct Primitives<P>
where
    P: Params,
{
    tlas: TlasPrimitives<P>,
    blas: HashMap<P::InstanceHandle, BlasPrimitives>,
}

impl<P> Primitives<P>
where
    P: Params,
{
    pub fn tlas(&self) -> &TlasPrimitives<P> {
        &self.tlas
    }

    pub fn tlas_mut(&mut self) -> &mut TlasPrimitives<P> {
        &mut self.tlas
    }

    pub fn create_blas(
        &mut self,
        handle: P::InstanceHandle,
    ) -> &mut BlasPrimitives {
        self.blas.entry(handle).or_default()
    }

    pub fn blas(&self, handle: P::InstanceHandle) -> &BlasPrimitives {
        self.blas.get(&handle).unwrap()
    }

    pub fn delete_blas(&mut self, handle: P::InstanceHandle) {
        self.blas.remove(&handle);
    }
}

impl<P> Default for Primitives<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            tlas: TlasPrimitives {
                allocator: Default::default(),
                items: Default::default(),
                current: Default::default(),
                previous: Default::default(),
                index: Default::default(),
            },
            blas: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct TlasPrimitives<P>
where
    P: Params,
{
    allocator: Allocator,
    items: Vec<Primitive>,
    current: Vec<Primitive>,
    previous: Vec<Primitive>,
    index: HashMap<P::InstanceHandle, IndexedPrimitive>,
}

impl<P> TlasPrimitives<P>
where
    P: Params,
{
    pub fn add(
        &mut self,
        handle: P::InstanceHandle,
        mut primitives: impl Iterator<Item = Primitive> + ExactSizeIterator,
    ) {
        if let Some(entry) = self.index.get(&handle) {
            // TODO ensure length is the same

            self.items[entry.primitive_ids.clone()]
                .fill_with(|| primitives.next().unwrap());
        } else {
            let primitive_ids =
                if let Some(ids) = self.allocator.take(primitives.len()) {
                    self.items[ids.clone()]
                        .fill_with(|| primitives.next().unwrap());

                    ids
                } else {
                    let a = self.items.len();
                    self.items.extend(primitives);
                    let b = self.items.len();

                    a..b
                };

            self.index
                .insert(handle, IndexedPrimitive { primitive_ids });
        }
    }

    pub fn remove(&mut self, parent: &P::InstanceHandle) {
        if let Some(entry) = self.index.remove(parent) {
            self.allocator.give(entry.primitive_ids.clone());
            self.items[entry.primitive_ids].fill(Primitive::Killed);
        }
    }

    pub fn all(&self) -> PrimitivesRef {
        PrimitivesRef::new(
            PrimitiveId::new(0),
            PrimitiveId::new(self.current.len() as u32),
        )
    }

    pub fn index(&self, range: PrimitivesRef) -> &[Primitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &self.current[start..end]
    }

    pub fn index_mut(&mut self, range: PrimitivesRef) -> &mut [Primitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &mut self.current[start..end]
    }

    pub fn copy(&mut self, previous: PrimitivesRef, current: PrimitivesRef) {
        self.current[current.as_range()]
            .copy_from_slice(&self.previous[previous.as_range()]);
    }

    pub fn begin(&mut self) {
        self.current = self.items.clone();
    }

    pub fn commit(&mut self) {
        self.previous = mem::take(&mut self.current);
    }
}

#[derive(Debug, Default)]
pub struct BlasPrimitives {
    items: Vec<Primitive>,
}

impl BlasPrimitives {
    pub fn add(
        &mut self,
        mut primitives: impl Iterator<Item = Primitive> + ExactSizeIterator,
    ) {
        if self.items.is_empty() {
            self.items.extend(primitives);
        } else {
            // TODO
            assert_eq!(primitives.len(), self.items.len());

            self.items.fill_with(|| primitives.next().unwrap());
        }
    }

    pub fn all(&self) -> PrimitivesRef {
        PrimitivesRef::new(
            PrimitiveId::new(0),
            PrimitiveId::new(self.items.len() as u32),
        )
    }

    pub fn index(&self, range: PrimitivesRef) -> &[Primitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &self.items[start..end]
    }

    pub fn index_mut(&mut self, range: PrimitivesRef) -> &mut [Primitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &mut self.items[start..end]
    }
}

#[derive(Debug)]
struct IndexedPrimitive {
    primitive_ids: Range<usize>,
}

pub trait PrimitivesAccessor
where
    Self: Sync + Send,
{
    fn index(&self, range: PrimitivesRef) -> &[Primitive];
    fn index_mut(&mut self, range: PrimitivesRef) -> &mut [Primitive];
}

impl<P> PrimitivesAccessor for TlasPrimitives<P>
where
    P: Params,
{
    fn index(&self, range: PrimitivesRef) -> &[Primitive] {
        Self::index(self, range)
    }

    fn index_mut(&mut self, range: PrimitivesRef) -> &mut [Primitive] {
        Self::index_mut(self, range)
    }
}

impl PrimitivesAccessor for BlasPrimitives {
    fn index(&self, range: PrimitivesRef) -> &[Primitive] {
        Self::index(self, range)
    }

    fn index_mut(&mut self, range: PrimitivesRef) -> &mut [Primitive] {
        Self::index_mut(self, range)
    }
}
