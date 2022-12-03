use crate::*;

/// Maps triangle vertices into UVs.
///
/// # Memory layout
///
/// One triangle's UVs take `3 [vertices] * 2 [f32 per vertice]` = `6 [f32]`,
/// which means that we can store two triangles worth of UVs in three `Vec4`,
/// giving us:
///
/// ```text
/// mapping #0              mapping #1
/// =====================   =====================
/// uvs[0]          uvs[1]          uvs[2]
/// -------------   -------------   -------------
/// x   y   z   w   x   y   z   w   x   y   z   w
/// .....   .....   .....   .....   .....   .....
/// uv0     uv1     uv2     uv0     uv1     uv2
/// ```
///
/// (`uvs` here standing for either `static_uvs` or `dynamic_uvs`.)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TriangleUvs {
    uvs: [Vec4; 3 * (MAX_STATIC_TRIANGLES + MAX_DYNAMIC_TRIANGLES) / 2],
}

impl TriangleUvs {
    pub fn get(&self, id: TriangleId<AnyTriangle>) -> TriangleUv {
        let (_, id) = id.unpack();

        if id % 2 == 0 {
            let ptr = 3 * (id / 2);

            TriangleUv {
                uv0: unsafe { self.uvs.get_unchecked(ptr).xy() },
                uv1: unsafe { self.uvs.get_unchecked(ptr).zw() },
                uv2: unsafe { self.uvs.get_unchecked(ptr + 1).xy() },
            }
        } else {
            let ptr = 3 * ((id - 1) / 2) + 1;

            TriangleUv {
                uv0: unsafe { self.uvs.get_unchecked(ptr).zw() },
                uv1: unsafe { self.uvs.get_unchecked(ptr + 1).xy() },
                uv2: unsafe { self.uvs.get_unchecked(ptr + 1).zw() },
            }
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl TriangleUvs {
    pub fn set(
        &mut self,
        id: TriangleId<AnyTriangle>,
        TriangleUv { uv0, uv1, uv2 }: TriangleUv,
    ) {
        let (_, id) = id.unpack();

        if id % 2 == 0 {
            let ptr = &mut self.uvs[3 * (id / 2)..][..2];

            ptr[0].x = uv0.x;
            ptr[0].y = uv0.y;
            ptr[0].z = uv1.x;
            ptr[0].w = uv1.y;
            ptr[1].x = uv2.x;
            ptr[1].y = uv2.y;
        } else {
            let ptr = &mut self.uvs[3 * ((id - 1) / 2) + 1..][..2];

            ptr[0].z = uv0.x;
            ptr[0].w = uv0.y;
            ptr[1].x = uv1.x;
            ptr[1].y = uv1.y;
            ptr[1].z = uv2.x;
            ptr[1].w = uv2.y;
        }
    }

    pub fn remove(&mut self, id: TriangleId<DynamicTriangle>) {
        for id in id.get()..(MAX_DYNAMIC_TRIANGLES - 1) {
            let curr_id = TriangleId::new_dynamic(id).into_any();
            let next_id = TriangleId::new_dynamic(id + 1).into_any();

            self.set(curr_id, self.get(next_id));
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for TriangleUvs {
    fn default() -> Self {
        Self::zeroed()
    }
}
