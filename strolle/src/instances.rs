use strolle_models as gpu;

use crate::buffers::StorageBufferable;
use crate::bvh::BoundingBox;

#[derive(Clone, Debug, Default)]
pub struct Instances {
    instances: Vec<gpu::Instance>,
    bounding_boxes: Vec<BoundingBox>,
}

impl Instances {
    pub fn clear(&mut self) {
        self.instances.clear();
        self.bounding_boxes.clear();
    }

    pub fn add(&mut self, instance: gpu::Instance, bounding_box: BoundingBox) {
        self.instances.push(instance);
        self.bounding_boxes.push(bounding_box);
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (gpu::InstanceId, gpu::Instance, BoundingBox)> + '_
    {
        self.instances
            .iter()
            .zip(self.bounding_boxes.iter())
            .enumerate()
            .map(|(id, (instance, bounding_box))| {
                (gpu::InstanceId::new(id as u32), *instance, *bounding_box)
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
