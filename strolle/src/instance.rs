use spirv_std::glam::{Mat4, Vec4};

use crate::Params;

#[derive(Debug)]
pub struct Instance<P>
where
    P: Params,
{
    mesh_handle: P::MeshHandle,
    material_handle: P::MaterialHandle,
    transform: Mat4,
}

impl<P> Instance<P>
where
    P: Params,
{
    pub fn new(
        mesh_handle: P::MeshHandle,
        material_handle: P::MaterialHandle,
        transform: Mat4,
    ) -> Self {
        assert!(
            transform.row(3).abs_diff_eq(Vec4::W, 1e-6),
            "Instances cannot have perspective projections"
        );

        Self {
            mesh_handle,
            material_handle,
            transform,
        }
    }

    pub fn mesh_handle(&self) -> &P::MeshHandle {
        &self.mesh_handle
    }

    pub fn material_handle(&self) -> &P::MaterialHandle {
        &self.material_handle
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }
}
