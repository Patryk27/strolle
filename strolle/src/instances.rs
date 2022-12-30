use strolle_models::InstanceId;

use crate::buffers::StorageBufferable;
use crate::bvh::BoundingBox;
use crate::Instance;

#[derive(Clone, Debug, Default)]
pub struct Instances {
    instances: Vec<Instance>,
    bounding_boxes: Vec<BoundingBox>,
}

impl Instances {
    pub fn clear(&mut self) {
        self.instances.clear();
        self.bounding_boxes.clear();
    }

    pub fn add(&mut self, instance: Instance, bounding_box: BoundingBox) {
        self.instances.push(instance);
        self.bounding_boxes.push(bounding_box);
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (InstanceId, Instance, BoundingBox)> + '_ {
        self.instances
            .iter()
            .zip(self.bounding_boxes.iter())
            .enumerate()
            .map(|(id, (instance, bounding_box))| {
                (InstanceId::new(id as u32), *instance, *bounding_box)
            })
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

impl StorageBufferable for Instances {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.instances)
    }
}
