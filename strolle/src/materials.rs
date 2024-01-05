use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Index;

use bevy::asset::Handle;
use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::pbr::StandardMaterial;
use bevy::prelude::World;
use bevy::render::render_resource::{BufferVec, IntoBinding};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::Image as BevyImage;
use glam::Vec4;
use wgpu::BufferUsages;

use crate::utils::{color_to_vec4, Allocator};
use crate::{gpu, ImageHandle, Images, MaterialHandle};

#[derive(Resource)]
pub struct Materials {
    allocator: Allocator,
    buffer: BufferVec<gpu::Material>,
    index: HashMap<MaterialHandle, gpu::MaterialId>,
    materials: Vec<StandardMaterial>,
}

impl Materials {
    pub fn add(&mut self, handle: MaterialHandle, material: StandardMaterial) {
        match self.index.entry(handle) {
            Entry::Occupied(entry) => {
                let material_id = *entry.get();

                self.materials[material_id.get() as usize] = material;
            }

            Entry::Vacant(entry) => {
                let material_id =
                    if let Some(material_id) = self.allocator.take(1) {
                        material_id.start
                    } else {
                        self.materials.push(material);
                        self.materials.len() - 1
                    };

                entry.insert(gpu::MaterialId::new(material_id as u32));
            }
        }
    }

    pub fn has(&self, handle: MaterialHandle) -> bool {
        self.index.contains_key(&handle)
    }

    pub fn remove(&mut self, handle: MaterialHandle) {
        let Some(id) = self.index.remove(&handle) else {
            return;
        };

        let id = id.get() as usize;

        self.allocator.give(id..id);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn lookup(&self, handle: MaterialHandle) -> Option<gpu::MaterialId> {
        self.index.get(&handle).copied()
    }

    pub fn refresh(&mut self, images: &Images) {
        let buffer = self.buffer.values_mut();

        buffer.clear();

        buffer.extend(
            self.materials
                .iter()
                .map(|mat| Self::prepare_one(images, mat)),
        );
    }

    fn prepare_one(images: &Images, mat: &StandardMaterial) -> gpu::Material {
        let lookup_tex = |handle: Option<&Handle<BevyImage>>| -> Vec4 {
            handle
                .and_then(|handle| images.lookup(ImageHandle::new(handle.id())))
                .unwrap_or_default()
        };

        gpu::Material {
            base_color: color_to_vec4(mat.base_color),
            base_color_texture: lookup_tex(mat.base_color_texture.as_ref()),
            emissive: color_to_vec4(mat.emissive),
            emissive_texture: lookup_tex(mat.emissive_texture.as_ref()),
            roughness: mat.perceptual_roughness.powf(2.0),
            metallic: mat.metallic,
            reflectance: mat.reflectance,
            ior: mat.ior,
            normal_map_texture: lookup_tex(mat.normal_map_texture.as_ref()),
        }
    }

    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.buffer.write_buffer(device, queue);
    }

    pub fn bind(&self) -> impl IntoBinding {
        self.buffer
            .buffer()
            .expect("buffer not ready: materials")
            .as_entire_buffer_binding()
    }
}

impl FromWorld for Materials {
    fn from_world(_: &mut World) -> Self {
        Self {
            allocator: Default::default(),
            buffer: BufferVec::new(BufferUsages::STORAGE),
            index: Default::default(),
            materials: Default::default(),
        }
    }
}

impl Index<gpu::MaterialId> for Materials {
    type Output = StandardMaterial;

    fn index(&self, index: gpu::MaterialId) -> &Self::Output {
        &self.materials[index.get() as usize]
    }
}
