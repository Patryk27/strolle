use std::collections::HashMap;
use std::num::NonZeroU32;

use crate::buffers::Bindable;
use crate::{ImageSampler, ImageTexture, Params};

pub struct Images<P>
where
    P: Params,
{
    textures: Vec<P::ImageTexture>,
    samplers: Vec<P::ImageSampler>,
    index: HashMap<P::ImageHandle, u32>,
    null_texture: wgpu::TextureView,
    null_sampler: wgpu::Sampler,
}

impl<P> Images<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        let null_texture = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("strolle_null_texture"),
                size: wgpu::Extent3d::default(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
            })
            .create_view(&Default::default());

        let null_sampler = device.create_sampler(&Default::default());

        Self {
            textures: Default::default(),
            samplers: Default::default(),
            index: Default::default(),
            null_texture,
            null_sampler,
        }
    }

    pub fn add(
        &mut self,
        image_handle: P::ImageHandle,
        image_texture: P::ImageTexture,
        image_sampler: P::ImageSampler,
    ) {
        let image_id = self.textures.len();

        log::trace!("Image added: {:?} ({})", image_handle, image_id);

        self.textures.push(image_texture);
        self.samplers.push(image_sampler);
        self.index.insert(image_handle, image_id as u32);
    }

    pub fn remove(&mut self, image_handle: &P::ImageHandle) {
        let Some(image_id) = self.index.remove(image_handle) else { return };

        log::trace!("Image removed: {:?} ({})", image_handle, image_id);

        self.textures.remove(image_id as usize);
        self.samplers.remove(image_id as usize);
    }

    pub fn lookup(&self, image_handle: &P::ImageHandle) -> Option<u32> {
        self.index.get(image_handle).copied()
    }

    pub fn binder(&self) -> ImagesBinder {
        ImagesBinder {
            textures: self
                .textures
                .iter()
                .map(|texture| texture.get())
                .collect(),
            samplers: self
                .samplers
                .iter()
                .map(|sampler| sampler.get())
                .collect(),
            null_texture: &self.null_texture,
            null_sampler: &self.null_sampler,
        }
    }
}

pub struct ImagesBinder<'a> {
    textures: Vec<&'a wgpu::TextureView>,
    samplers: Vec<&'a wgpu::Sampler>,
    null_texture: &'a wgpu::TextureView,
    null_sampler: &'a wgpu::Sampler,
}

impl Bindable for ImagesBinder<'_> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let count = NonZeroU32::new(self.textures.len() as u32);

        let textures_layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float {
                    filterable: false,
                },
            },
            count,
        };

        let samplers_layout = wgpu::BindGroupLayoutEntry {
            binding: binding + 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Sampler(
                wgpu::SamplerBindingType::NonFiltering,
            ),
            count,
        };

        let (textures_resource, samplers_resource) = if count.is_none() {
            // Even if there are no textures, we still have to bind *something*
            // to the pipeline - so let's bind an empty texture & empty sampler.
            //
            // Note that even though what we're binding is `TextureView`, not
            // `TextureViewArray` (and the same for samplers), the shader can
            // still continue to refer to them through `images: &[Image!(...)]`;
            // somewhat magically it all works (and seems to be a common pattern
            // in cases like these).
            (
                wgpu::BindingResource::TextureView(self.null_texture),
                wgpu::BindingResource::Sampler(self.null_sampler),
            )
        } else {
            (
                wgpu::BindingResource::TextureViewArray(&self.textures),
                wgpu::BindingResource::SamplerArray(&self.samplers),
            )
        };

        vec![
            (textures_layout, textures_resource),
            (samplers_layout, samplers_resource),
        ]
    }
}
