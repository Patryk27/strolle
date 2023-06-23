use std::collections::HashMap;

use crate::{CameraController, CameraHandle};

#[derive(Debug, Default)]
pub struct CameraControllers {
    cameras: HashMap<CameraHandle, CameraController>,
    next_id: usize,
}

impl CameraControllers {
    pub fn add(&mut self, camera: CameraController) -> CameraHandle {
        let handle = CameraHandle::new(self.next_id);

        self.cameras.insert(handle, camera);
        self.next_id += 1;

        handle
    }

    pub fn get(&self, camera_handle: CameraHandle) -> &CameraController {
        self.cameras.get(&camera_handle).unwrap_or_else(|| {
            panic!("Camera does not exist: {:?}", camera_handle)
        })
    }

    pub fn get_mut(
        &mut self,
        camera_handle: CameraHandle,
    ) -> &mut CameraController {
        self.cameras.get_mut(&camera_handle).unwrap_or_else(|| {
            panic!("Camera does not exist: {:?}", camera_handle)
        })
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut CameraController> + '_ {
        self.cameras.values_mut()
    }

    pub fn remove(&mut self, camera_handle: CameraHandle) {
        self.cameras.remove(&camera_handle);
    }
}
