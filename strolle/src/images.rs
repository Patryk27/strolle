use std::collections::HashMap;
use std::mem;
use std::num::NonZeroU32;

use derivative::Derivative;
use glam::{uvec2, vec4, Vec4};
use guillotiere::{size2, Allocation, AtlasAllocator};
use log::warn;

use crate::{Bindable, Image, ImageData, Params, Texture};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Images<P>
where
    P: Params,
{
    #[derivative(Debug = "ignore")]
    atlas: AtlasAllocator,
    atlas_texture: Texture,
    atlas_changes: Vec<AtlasChange<P>>,
    images: HashMap<P::ImageHandle, Allocation>,
    dynamic_textures: Vec<(P::ImageTexture, Allocation)>,
}

impl<P> Images<P>
where
    P: Params,
{
    const ATLAS_WIDTH: u32 = 8192;
    const ATLAS_HEIGHT: u32 = 8192;

    pub fn new(device: &wgpu::Device) -> Self {
        let atlas = AtlasAllocator::new(size2(
            Self::ATLAS_WIDTH as i32,
            Self::ATLAS_HEIGHT as i32,
        ));

        let atlas_texture = Texture::builder("atlas")
            .with_size(uvec2(Self::ATLAS_WIDTH, Self::ATLAS_HEIGHT))
            .with_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .with_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .with_usage(wgpu::TextureUsages::COPY_DST)
            .build(device);

        Self {
            atlas,
            atlas_texture,
            atlas_changes: Default::default(),
            images: Default::default(),
            dynamic_textures: Default::default(),
        }
    }

    pub fn add(&mut self, image_handle: P::ImageHandle, image: Image<P>) {
        let image_size = size2(
            image.texture_descriptor.size.width as i32,
            image.texture_descriptor.size.height as i32,
        );

        let image_alloc =
            if let Some(image_alloc) = self.images.get(&image_handle) {
                if image_size == image_alloc.rectangle.size() {
                    Some(*image_alloc)
                } else {
                    self.atlas.deallocate(image_alloc.id);
                    self.atlas.allocate(image_size)
                }
            } else {
                self.atlas.allocate(image_size)
            };

        let Some(image_alloc) = image_alloc else {
            // TODO allocate new atlas, up to 16 (Metal's limit)
            warn!("Cannot add image `{:?}` - no more space in the atlas", image_handle);
            return;
        };

        self.images.insert(image_handle, image_alloc);

        match image.data {
            data @ (ImageData::Raw { .. }
            | ImageData::Texture {
                is_dynamic: false, ..
            }) => {
                self.atlas_changes.push(AtlasChange::Set {
                    x: image_alloc.rectangle.min.x as u32,
                    y: image_alloc.rectangle.min.y as u32,
                    w: image_alloc.rectangle.width() as u32,
                    h: image_alloc.rectangle.height() as u32,
                    data,
                });
            }

            ImageData::Texture {
                texture,
                is_dynamic: true,
            } => {
                self.dynamic_textures.push((texture, image_alloc));
            }
        }
    }

    pub fn remove(&mut self, image_handle: &P::ImageHandle) {
        let Some(image_alloc) = self.images.remove(image_handle) else { return };

        self.atlas.deallocate(image_alloc.id);
    }

    pub fn lookup(&self, image_handle: &P::ImageHandle) -> Option<Vec4> {
        self.images.get(image_handle).map(|alloc| {
            vec4(
                alloc.rectangle.min.x as f32 / (Self::ATLAS_WIDTH as f32),
                alloc.rectangle.min.y as f32 / (Self::ATLAS_HEIGHT as f32),
                alloc.rectangle.width() as f32 / (Self::ATLAS_WIDTH as f32),
                alloc.rectangle.height() as f32 / (Self::ATLAS_HEIGHT as f32),
            )
        })
    }

    pub fn lookup_opt(
        &self,
        image_handle: Option<&P::ImageHandle>,
    ) -> Option<Vec4> {
        self.lookup(image_handle?)
    }

    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = None;

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
                            queue.write_texture(
                                wgpu::ImageCopyTexture {
                                    texture: self.atlas_texture.tex(),
                                    mip_level: 0,
                                    origin: wgpu::Origin3d { x, y, z: 0 },
                                    aspect: wgpu::TextureAspect::All,
                                },
                                &data,
                                wgpu::ImageDataLayout {
                                    offset: 0,
                                    bytes_per_row: NonZeroU32::new(w * 4),
                                    rows_per_image: None,
                                },
                                wgpu::Extent3d {
                                    width: w,
                                    height: h,
                                    depth_or_array_layers: 1,
                                },
                            );
                        }

                        ImageData::Texture { texture, .. } => {
                            let encoder = encoder.get_or_insert_with(|| {
                                device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor {
                                        label: Some("strolle_atlas"),
                                    },
                                )
                            });

                            encoder.copy_texture_to_texture(
                                texture.as_image_copy(),
                                wgpu::ImageCopyTexture {
                                    texture: self.atlas_texture.tex(),
                                    mip_level: 0,
                                    origin: wgpu::Origin3d { x, y, z: 0 },
                                    aspect: wgpu::TextureAspect::All,
                                },
                                size,
                            );
                        }
                    }
                }
            }
        }

        for (texture, alloc) in &self.dynamic_textures {
            let encoder = encoder.get_or_insert_with(|| {
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("strolle_atlas"),
                })
            });

            encoder.copy_texture_to_texture(
                texture.as_image_copy(),
                wgpu::ImageCopyTexture {
                    texture: self.atlas_texture.tex(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: alloc.rectangle.min.x as u32,
                        y: alloc.rectangle.min.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: alloc.rectangle.size().width as u32,
                    height: alloc.rectangle.size().height as u32,
                    depth_or_array_layers: 1,
                },
            )
        }

        if let Some(encoder) = encoder {
            queue.submit([encoder.finish()]);
        }
    }

    pub fn bind_sampled(&self) -> impl Bindable + '_ {
        self.atlas_texture.bind_sampled()
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
enum AtlasChange<P>
where
    P: Params,
{
    Set {
        x: u32,
        y: u32,
        w: u32,
        h: u32,

        #[derivative(Debug = "ignore")]
        data: ImageData<P>,
    },
}
