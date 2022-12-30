use crate::{Instance, InstanceId};

pub struct InstancesView<'a> {
    data: &'a [Instance],
}

impl<'a> InstancesView<'a> {
    pub fn new(data: &'a [Instance]) -> Self {
        Self { data }
    }

    pub fn get(&self, id: InstanceId) -> Instance {
        // TODO safety
        unsafe { *self.data.get_unchecked(id.get() as usize) }
    }
}
