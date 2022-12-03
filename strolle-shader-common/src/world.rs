use crate::*;

pub struct World<'a> {
    pub static_geo: &'a StaticGeometry,
    pub static_geo_index: &'a StaticGeometryIndex,
    pub dynamic_geo: &'a DynamicGeometry,
    pub uvs: &'a TriangleUvs,
    pub lights: &'a Lights,
    pub materials: &'a Materials,
    pub atlas_tex: &'a Image!(2D, type=f32, sampled),
    pub atlas_sampler: &'a Sampler,
}

impl<'a> World<'a> {
    pub fn atlas_sample(
        &self,
        tri_id: TriangleId<AnyTriangle>,
        hit: Hit,
    ) -> Vec4 {
        let tri_uv = self.uvs.get(tri_id);

        let mut tex_uv = tri_uv.uv0
            + (tri_uv.uv1 - tri_uv.uv0) * hit.uv.x
            + (tri_uv.uv2 - tri_uv.uv0) * hit.uv.y;

        // When `uv_scale` (aka `uv_divisor`) is set, we pretend that the
        // backing texture is `1.0 / uv_scale` larger than in reality - this is
        // pretty cursed and works only on rectangular textures, which happens
        // to exactly match our needs
        if hit.uv.z < 1.0 || hit.uv.w < 1.0 {
            let tex_min = tri_uv.uv0.min(tri_uv.uv1).min(tri_uv.uv2);
            let tex_max = tri_uv.uv0.max(tri_uv.uv1).max(tri_uv.uv2);

            let tex_size = tex_max - tex_min;
            let tex_hit = (tex_uv - tex_min) / tex_size;

            tex_uv =
                tex_min + (tex_hit % hit.uv.zw()) * (tex_size / hit.uv.zw());
        }

        self.atlas_tex
            .sample_by_lod(*self.atlas_sampler, tex_uv, 0.0)
    }
}
