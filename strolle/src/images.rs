use std::collections::HashMap;
use std::mem;

use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::prelude::World;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use derivative::Derivative;
use glam::{vec4, Vec4};
use guillotiere::{size2, Allocation, AtlasAllocator};
use log::warn;

use crate::{Image, ImageData, ImageHandle, Texture};

#[derive(Derivative, Resource)]
#[derivative(Debug)]
pub struct Images {
    #[derivative(Debug = "ignore")]
    atlas: AtlasAllocator,
    atlas_texture: Texture,
    atlas_changes: Vec<AtlasChange>,
    images: HashMap<ImageHandle, Allocation>,
    dynamic_textures: Vec<(Texture, Allocation)>,
}

impl Images {
    const ATLAS_WIDTH: u32 = 8192;
    const ATLAS_HEIGHT: u32 = 8192;

    pub fn add(&mut self, handle: ImageHandle, image: Image) {
        let size = size2(
            image.texture_descriptor.size.width as i32,
            image.texture_descriptor.size.height as i32,
        );

        let alloc = if let Some(alloc) = self.images.get(&handle) {
            if size == alloc.rectangle.size() {
                Some(*alloc)
            } else {
                self.atlas.deallocate(alloc.id);
                self.atlas.allocate(size)
            }
        } else {
            self.atlas.allocate(size)
        };

        let Some(alloc) = alloc else {
            // TODO allocate new atlas, up to 16 (Metal's limit)
            warn!(
                "Cannot add image `{:?}` - no more space in the atlas",
                handle
            );
            return;
        };

        self.images.insert(handle, alloc);

        match image.data {
            data @ (ImageData::Raw { .. }
            | ImageData::Texture {
                is_dynamic: false, ..
            }) => {
                self.atlas_changes.push(AtlasChange::Set {
                    x: alloc.rectangle.min.x as u32,
                    y: alloc.rectangle.min.y as u32,
                    w: alloc.rectangle.width() as u32,
                    h: alloc.rectangle.height() as u32,
                    data,
                });
            }

            ImageData::Texture {
                texture,
                is_dynamic: true,
            } => {
                self.dynamic_textures.push((texture, alloc));
            }
        }
    }

    pub fn remove(&mut self, handle: ImageHandle) {
        let Some(alloc) = self.images.remove(&handle) else {
            return;
        };

        self.atlas.deallocate(alloc.id);
    }

    pub fn lookup(&self, handle: ImageHandle) -> Option<Vec4> {
        self.images.get(&handle).map(|alloc| {
            vec4(
                alloc.rectangle.min.x as f32 / (Self::ATLAS_WIDTH as f32),
                alloc.rectangle.min.y as f32 / (Self::ATLAS_HEIGHT as f32),
                alloc.rectangle.width() as f32 / (Self::ATLAS_WIDTH as f32),
                alloc.rectangle.height() as f32 / (Self::ATLAS_HEIGHT as f32),
            )
        })
    }

    pub fn lookup_opt(&self, handle: Option<ImageHandle>) -> Option<Vec4> {
        self.lookup(handle?)
    }

    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        // let mut encoder = None;

        for change in mem::take(&mut self.atlas_changes) {
            match change {
                AtlasChange::Set { x, y, w, h, data } => {
                    let size = wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    };

                    match data {
                        ImageData::Raw { data } => {
                            // queue.write_texture(
                            //     wgpu::ImageCopyTexture {
                            //         texture: self.atlas_texture.tex(),
                            //         mip_level: 0,
                            //         origin: wgpu::Origin3d { x, y, z: 0 },
                            //         aspect: wgpu::TextureAspect::All,
                            //     },
                            //     &data,
                            //     wgpu::ImageDataLayout {
                            //         offset: 0,
                            //         bytes_per_row: Some(w * 4),
                            //         rows_per_image: None,
                            //     },
                            //     wgpu::Extent3d {
                            //         width: w,
                            //         height: h,
                            //         depth_or_array_layers: 1,
                            //     },
                            // );
                        }

                        ImageData::Texture { texture, .. } => {
                            // let encoder = encoder.get_or_insert_with(|| {
                            //     device.create_command_encoder(
                            //         &wgpu::CommandEncoderDescriptor {
                            //             label: Some("strolle_atlas"),
                            //         },
                            //     )
                            // });

                            // encoder.copy_texture_to_texture(
                            //     texture.as_image_copy(),
                            //     wgpu::ImageCopyTexture {
                            //         texture: self.atlas_texture.tex(),
                            //         mip_level: 0,
                            //         origin: wgpu::Origin3d { x, y, z: 0 },
                            //         aspect: wgpu::TextureAspect::All,
                            //     },
                            //     size,
                            // );
                        }
                    }
                }
            }
        }

        for (texture, alloc) in &self.dynamic_textures {
            // let encoder = encoder.get_or_insert_with(|| {
            //     device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            //         label: Some("strolle_atlas"),
            //     })
            // });

            // encoder.copy_texture_to_texture(
            //     texture.as_image_copy(),
            //     wgpu::ImageCopyTexture {
            //         texture: self.atlas_texture.tex(),
            //         mip_level: 0,
            //         origin: wgpu::Origin3d {
            //             x: alloc.rectangle.min.x as u32,
            //             y: alloc.rectangle.min.y as u32,
            //             z: 0,
            //         },
            //         aspect: wgpu::TextureAspect::All,
            //     },
            //     wgpu::Extent3d {
            //         width: alloc.rectangle.size().width as u32,
            //         height: alloc.rectangle.size().height as u32,
            //         depth_or_array_layers: 1,
            //     },
            // )
        }

        // if let Some(encoder) = encoder {
        //     queue.submit([encoder.finish()]);
        // }
    }
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let atlas = AtlasAllocator::new(size2(
            Self::ATLAS_WIDTH as i32,
            Self::ATLAS_HEIGHT as i32,
        ));

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("strolle_atlas_texture"),
            size: wgpu::Extent3d {
                width: Self::ATLAS_WIDTH,
                height: Self::ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Self {
            atlas,
            atlas_texture,
            atlas_changes: Default::default(),
            images: Default::default(),
            dynamic_textures: Default::default(),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
enum AtlasChange {
    Set {
        x: u32,
        y: u32,
        w: u32,
        h: u32,

        #[derivative(Debug = "ignore")]
        data: ImageData,
    },
}
