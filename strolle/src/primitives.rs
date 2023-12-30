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
    scopes: HashMap<PrimitiveScope<P>, ScopedPrimitives<P>>,
}

impl<P> Primitives<P>
where
    P: Params,
{
    pub fn create_scope(
        &mut self,
        scope: PrimitiveScope<P>,
    ) -> &mut ScopedPrimitives<P> {
        self.scopes.entry(scope).or_default()
    }

    pub fn delete_scope(&mut self, scope: PrimitiveScope<P>) {
        self.scopes.remove(&scope);
    }

    pub fn scope(&self, scope: PrimitiveScope<P>) -> &ScopedPrimitives<P> {
        self.scopes
            .get(&scope)
            .unwrap_or_else(|| panic!("scope does not exist: {scope:?}"))
    }

    pub fn scope_mut(
        &mut self,
        scope: PrimitiveScope<P>,
    ) -> &mut ScopedPrimitives<P> {
        self.scopes
            .get_mut(&scope)
            .unwrap_or_else(|| panic!("scope does not exist: {scope:?}"))
    }
}

impl<P> Default for Primitives<P>
where
    P: Params,
{
    fn default() -> Self {
        let mut this = Self {
            scopes: Default::default(),
        };

        this.create_scope(PrimitiveScope::Tlas);
        this
    }
}

#[derive(Debug, Default)]
pub struct ScopedPrimitives<P>
where
    P: Params,
{
    allocator: Allocator,
    items: Vec<Primitive>,
    current: Vec<Primitive>,
    previous: Vec<Primitive>,
    index: HashMap<PrimitiveOwner<P>, IndexedPrimitive>,
}

impl<P> ScopedPrimitives<P>
where
    P: Params,
{
    pub fn add(
        &mut self,
        handle: PrimitiveOwner<P>,
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

    pub fn remove(&mut self, parent: &PrimitiveOwner<P>) {
        if let Some(entry) = self.index.remove(parent) {
            self.allocator.give(entry.primitive_ids.clone());
            self.items[entry.primitive_ids].fill(Primitive::Killed);
        }
    }

    pub fn get_ref(&self) -> PrimitivesRef {
        PrimitivesRef::new(
            PrimitiveId::new(0),
            PrimitiveId::new(self.current.len() as u32),
        )
    }

    pub fn current(&self, range: PrimitivesRef) -> &[Primitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &self.current[start..end]
    }

    pub fn current_mut(&mut self, range: PrimitivesRef) -> &mut [Primitive] {
        let start = range.start().get() as usize;
        let end = range.end().get() as usize;

        &mut self.current[start..end]
    }

    pub fn copy_previous_to_current(
        &mut self,
        previous: PrimitivesRef,
        current: PrimitivesRef,
    ) {
        self.current[current.as_range()]
            .copy_from_slice(&self.previous[previous.as_range()]);
    }

    pub fn begin_bvh_refresh(&mut self) {
        self.current = self.items.clone();
    }

    pub fn end_bvh_refresh(&mut self) {
        self.previous = mem::take(&mut self.current);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveScope<P>
where
    P: Params,
{
    Tlas,
    Blas(P::MeshHandle),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveOwner<P>
where
    P: Params,
{
    Mesh(P::MeshHandle),
    Instance(P::InstanceHandle),
}

#[derive(Debug)]
struct IndexedPrimitive {
    primitive_ids: Range<usize>,
}
