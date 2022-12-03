use strolle_shader_common::STATIC_GEOMETRY_INDEX_SIZE;

use super::*;

pub fn serialize(fbvh: RopedBvh) -> (StaticGeometryIndex, usize) {
    let mut out = Vec::new();

    for node in fbvh {
        let v1;
        let v2;

        match node {
            RopedBvhNode::Leaf { triangle, goto_id } => {
                let goto_ptr = goto_id.map(|id| id * 2).unwrap_or_default();

                let info = 1
                    | ((triangle.get() as u32) << 1)
                    | ((goto_ptr as u32) << 16);

                v1 = vec4(0.0, 0.0, 0.0, f32::from_bits(info));
                v2 = vec4(0.0, 0.0, 0.0, 0.0);
            }

            RopedBvhNode::NonLeaf {
                bb,
                on_hit_goto_id,
                on_miss_goto_id,
            } => {
                let on_hit_goto_ptr =
                    on_hit_goto_id.map(|id| id * 2).unwrap_or_default();

                let on_miss_goto_ptr =
                    on_miss_goto_id.map(|id| id * 2).unwrap_or_default();

                let info = 0
                    | ((on_hit_goto_ptr as u32) << 1)
                    | ((on_miss_goto_ptr as u32) << 16);

                v1 = bb.min().extend(f32::from_bits(info));
                v2 = bb.max().extend(0.0);
            }
        }

        out.push(v1);
        out.push(v2);
    }

    let out_len = out.len();

    // ----

    while out.len() < STATIC_GEOMETRY_INDEX_SIZE {
        out.push(vec4(0.0, 0.0, 0.0, 0.0));
    }

    let out = out.try_into().unwrap_or_else(|out: Vec<_>| {
        panic!(
            "ayy ayy the geometry index is too large -- produced {} items",
            out.len()
        );
    });

    (StaticGeometryIndex::new(out), out_len)
}
