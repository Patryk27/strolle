use crate::{Instance, InstanceId};

pub struct InstancesView<'a> {
    items: &'a [Instance],
}

impl<'a> InstancesView<'a> {
    pub fn new(items: &'a [Instance]) -> Self {
        Self { items }
    }

    pub fn get(&self, id: InstanceId) -> Instance {
        unsafe { *self.items.get_unchecked(id.get() as usize) }
    }
}
