use std::collections::HashMap;
use std::mem;
use std::num::NonZeroU32;

use derivative::Derivative;
use glam::{uvec2, vec4, Vec4};
use guillotiere::{size2, Allocation, AtlasAllocator};
use log::warn;

use crate::{Bindable, Params, Texture};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Images<P>
where
    P: Params,
{
    #[derivative(Debug = "ignore")]
    atlas: AtlasAllocator,
    atlas_texture: Texture,
    atlas_changes: Vec<AtlasChange>,
    images: HashMap<P::ImageHandle, Allocation>,
}

impl<P> Images<P>
where
    P: Params,
{
    const ATLAS_WIDTH: u32 = 4096;
    const ATLAS_HEIGHT: u32 = 4096;

    pub fn new(device: &wgpu::Device) -> Self {
        let atlas = AtlasAllocator::new(size2(
            Self::ATLAS_WIDTH as i32,
            Self::ATLAS_HEIGHT as i32,
        ));

        let atlas_texture = Texture::builder("atlas")
            .with_size(uvec2(Self::ATLAS_WIDTH, Self::ATLAS_HEIGHT))
            .with_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .add_usage(wgpu::TextureUsages::COPY_DST)
            .build(device);

        Self {
            atlas,
            atlas_texture,
            atlas_changes: Default::default(),
            images: Default::default(),
        }
    }

    pub fn add(
        &mut self,
        image_handle: P::ImageHandle,
        image_data: Vec<u8>,
        image_texture: wgpu::TextureDescriptor,
        _image_sampler: wgpu::SamplerDescriptor,
    ) {
        assert_eq!(image_texture.mip_level_count, 1);
        assert_eq!(image_texture.sample_count, 1);
        assert_eq!(image_texture.dimension, wgpu::TextureDimension::D2);

        // TODO we should convert textures to a common format
        assert!([
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ]
        .contains(&image_texture.format));

        // TODO propagate sampler's addressing modes to the shader so that we
        //      know whether the texture should be repeated, etc.

        let image_size = size2(
            image_texture.size.width as i32,
            image_texture.size.height as i32,
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

        self.atlas_changes.push(AtlasChange::Add {
            x: image_alloc.rectangle.min.x as u32,
            y: image_alloc.rectangle.min.y as u32,
            w: image_alloc.rectangle.width() as u32,
            h: image_alloc.rectangle.height() as u32,
            data: image_data,
        });
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

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        for change in mem::take(&mut self.atlas_changes) {
            match change {
                AtlasChange::Add { x, y, w, h, data } => {
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
            }
        }
    }

    pub fn bind_sampled(&self) -> impl Bindable + '_ {
        self.atlas_texture.bind_sampled()
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
enum AtlasChange {
    Add {
        x: u32,
        y: u32,
        w: u32,
        h: u32,

        #[derivative(Debug = "ignore")]
        data: Vec<u8>,
    },
}
