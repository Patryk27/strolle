/// Relative triangle id, starts as zero for each mesh.
///
/// To obtain absolute triangle id (that can actually be used to load triangles
/// from the buffer), you have to load appropriate mesh-instance and inspect its
/// minimum triangle id.
#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct MeshTriangleId(u32);

impl MeshTriangleId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}
