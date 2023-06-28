use glam::Affine3A;

use crate::Params;

#[derive(Debug)]
pub struct Instance<P>
where
    P: Params,
{
    pub(crate) mesh_handle: P::MeshHandle,
    pub(crate) material_handle: P::MaterialHandle,
    pub(crate) transform: Affine3A,
    pub(crate) transform_inverse: Affine3A,
}

impl<P> Instance<P>
where
    P: Params,
{
    pub fn new(
        mesh_handle: P::MeshHandle,
        material_handle: P::MaterialHandle,
        transform: Affine3A,
    ) -> Self {
        Self {
            mesh_handle,
            material_handle,
            transform,
            transform_inverse: transform.inverse(),
        }
    }
}
