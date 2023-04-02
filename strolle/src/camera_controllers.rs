use std::collections::HashMap;

use crate::{CameraController, CameraHandle, Params};

#[derive(Debug)]
pub struct CameraControllers<P>
where
    P: Params,
{
    cameras: HashMap<CameraHandle, CameraController<P>>,
    next_id: usize,
}

impl<P> CameraControllers<P>
where
    P: Params,
{
    pub fn add(&mut self, camera: CameraController<P>) -> CameraHandle {
        let handle = CameraHandle::new(self.next_id);

        self.cameras.insert(handle, camera);
        self.next_id += 1;

        handle
    }

    pub fn get(&self, camera_handle: CameraHandle) -> &CameraController<P> {
        self.cameras.get(&camera_handle).unwrap_or_else(|| {
            panic!("Camera does not exist: {:?}", camera_handle)
        })
    }

    pub fn get_mut(
        &mut self,
        camera_handle: CameraHandle,
    ) -> &mut CameraController<P> {
        self.cameras.get_mut(&camera_handle).unwrap_or_else(|| {
            panic!("Camera does not exist: {:?}", camera_handle)
        })
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut CameraController<P>> + '_ {
        self.cameras.values_mut()
    }

    pub fn remove(&mut self, camera_handle: CameraHandle) {
        self.cameras.remove(&camera_handle);
    }
}

impl<P> Default for CameraControllers<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            cameras: Default::default(),
            next_id: Default::default(),
        }
    }
}
