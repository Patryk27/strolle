use crate::*;

#[derive(Copy, Clone, Default)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn hits_box_at(self, bb_min: Vec3, bb_max: Vec3) -> f32 {
        let hit_min = (bb_min - self.origin) / self.direction;
        let hit_max = (bb_max - self.origin) / self.direction;

        let x_entry = hit_min.x.min(hit_max.x);
        let y_entry = hit_min.y.min(hit_max.y);
        let z_entry = hit_min.z.min(hit_max.z);
        let x_exit = hit_min.x.max(hit_max.x);
        let y_exit = hit_min.y.max(hit_max.y);
        let z_exit = hit_min.z.max(hit_max.z);

        let latest_entry = x_entry.max(y_entry).max(z_entry);
        let earliest_exit = x_exit.min(y_exit).min(z_exit);

        if latest_entry <= earliest_exit && earliest_exit > 0.0 {
            latest_entry
        } else {
            f32::MAX
        }
    }

    pub fn hits_anything_up_to(self, world: &World, distance: f32) -> bool {
        // Check static geometry
        let mut ptr = 0;

        loop {
            let v1 = world.static_geo_index.read(ptr);
            let v2 = world.static_geo_index.read(ptr + 1);

            let info = v1.w.to_bits();
            let is_leaf = info & 1 == 1;
            let i1 = info << 16 >> 17;
            let i2 = info >> 16;

            if is_leaf {
                let tri_id = TriangleId::new_static(i1 as usize);
                let tri = world.static_geo.get(tri_id);

                if tri.casts_shadows() {
                    let hit = tri.hit(self, false);

                    if hit.t < distance {
                        let got_hit = if tri.has_uv_transparency() {
                            world.atlas_sample(tri_id.into_any(), hit).w > 0.5
                        } else {
                            true
                        };

                        if got_hit {
                            return true;
                        }
                    }
                }

                ptr = i2 as usize;
            } else {
                let at = self.hits_box_at(v1.xyz(), v2.xyz());

                if at < distance {
                    ptr = i1 as usize;
                } else {
                    ptr = i2 as usize;
                }
            }

            if ptr == 0 {
                break;
            }
        }

        // Check dynamic geometry
        let mut tri_idx = 0;

        while tri_idx < world.dynamic_geo.len() {
            let tri_id = TriangleId::new_dynamic(tri_idx);
            let tri = world.dynamic_geo.get(tri_id);

            if tri.casts_shadows() {
                let hit = tri.hit(self, false);

                if hit.t < distance {
                    let got_hit = if tri.has_uv_transparency() {
                        world.atlas_sample(tri_id.into_any(), hit).w > 0.5
                    } else {
                        true
                    };

                    if got_hit {
                        return true;
                    }
                }
            }

            tri_idx += 1;
        }

        false
    }

    pub fn trace(self, world: &World, culling: bool) -> Hit {
        let mut hit = Hit::none();

        // Check static geometry
        let mut ptr = 0;

        loop {
            let v1 = world.static_geo_index.read(ptr);
            let v2 = world.static_geo_index.read(ptr + 1);

            let info = v1.w.to_bits();
            let is_leaf = info & 1 == 1;
            let i1 = info << 16 >> 17;
            let i2 = info >> 16;

            if is_leaf {
                let tri_id = TriangleId::new_static(i1 as usize);
                let tri = world.static_geo.get(tri_id);
                let curr_hit = tri.hit(self, culling);

                if curr_hit.is_closer_than(hit) {
                    let got_hit = if tri.has_uv_transparency() {
                        world.atlas_sample(tri_id.into_any(), curr_hit).w > 0.5
                    } else {
                        true
                    };

                    if got_hit {
                        hit = curr_hit;
                        hit.tri_id = tri_id.into_any();
                    }
                }

                ptr = i2 as usize;
            } else {
                let at = self.hits_box_at(v1.xyz(), v2.xyz());

                if at < hit.t {
                    ptr = i1 as usize;
                } else {
                    ptr = i2 as usize;
                }
            }

            if ptr == 0 {
                break;
            }
        }

        // Check dynamic geometry
        let mut tri_idx = 0;

        while tri_idx < world.dynamic_geo.len() {
            let tri_id = TriangleId::new_dynamic(tri_idx);
            let tri = world.dynamic_geo.get(tri_id);
            let curr_hit = tri.hit(self, culling);

            if curr_hit.is_closer_than(hit) {
                let got_hit = if tri.has_uv_transparency() {
                    world.atlas_sample(tri_id.into_any(), curr_hit).w > 0.5
                } else {
                    true
                };

                if got_hit {
                    hit = curr_hit;
                    hit.tri_id = tri_id.into_any();
                }
            }

            tri_idx += 1;
        }

        hit
    }

    pub fn shade(mut self, color: &mut Vec4, world: &World) {
        const ST_FIRST_HIT: usize = 0;
        const ST_REFLECTED: usize = 1;
        const ST_TRANSPARENT: usize = 2;

        let mut state = ST_FIRST_HIT;
        let mut state_vars = Mat4::default();

        loop {
            let culling = state == ST_FIRST_HIT || state == ST_TRANSPARENT;

            let hit = self.trace(world, culling);
            let hit_mat;
            let hit_color;

            if hit.is_some() {
                hit_mat = world.materials.get(hit.mat_id);
                hit_color = hit_mat.radiance(world, hit);
            } else {
                hit_mat = Material::none();
                hit_color = Default::default();
            }

            match state {
                ST_FIRST_HIT => {
                    *color = hit_color.extend(1.0);

                    if hit_mat.reflectivity() > 0.0 {
                        state = ST_REFLECTED;

                        state_vars.x_axis = hit_mat
                            .reflectivity_color()
                            .extend(hit_mat.reflectivity());

                        // We preemtively store `self.direction`, because we
                        // might need it if `hit.alpha < 1.0`
                        state_vars.z_axis = self.direction.extend(0.0);

                        let reflection_dir = {
                            let camera_dir = -hit.ray.direction();

                            hit.normal * hit.normal.dot(camera_dir) * 2.0
                                - camera_dir
                        };

                        self.origin = hit.point;
                        self.direction = reflection_dir.normalize();
                    } else if hit.alpha < 1.0 {
                        state = ST_TRANSPARENT;
                        state_vars.x_axis = vec4(hit.alpha, 0.0, 0.0, 0.0);

                        self.origin = hit.point + 0.1 * self.direction;
                    } else {
                        break;
                    }
                }

                ST_REFLECTED => {
                    *color += (hit_color
                        * state_vars.x_axis.xyz()
                        * state_vars.x_axis.w)
                        .extend(0.0);

                    break;
                }

                ST_TRANSPARENT => {
                    let alpha = state_vars.x_axis.x;

                    *color = (color.truncate() * alpha
                        + hit_color * (1.0 - alpha))
                        .extend(1.0);

                    break;
                }

                _ => {
                    // unreachable
                    break;
                }
            }
        }
    }
}
