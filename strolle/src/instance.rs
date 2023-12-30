use glam::{Affine3A, Mat4};

use crate::Params;

#[derive(Debug)]
pub struct Instance<P>
where
    P: Params,
{
    pub(crate) mesh_handle: P::MeshHandle,
    pub(crate) material_handle: P::MaterialHandle,
    pub(crate) xform: Affine3A,
    pub(crate) xform_inv: Affine3A,
    pub(crate) xform_inv_trans: Mat4,
    pub(crate) inline: bool,
}

impl<P> Instance<P>
where
    P: Params,
{
    pub fn new(
        mesh_handle: P::MeshHandle,
        material_handle: P::MaterialHandle,
        xform: Affine3A,
    ) -> Self {
        let xform_inv = xform.inverse();
        let xform_inv_trans = Mat4::from(xform_inv).transpose();

        Self {
            mesh_handle,
            material_handle,
            xform,
            xform_inv,
            xform_inv_trans,
            inline: true,
        }
    }
}
