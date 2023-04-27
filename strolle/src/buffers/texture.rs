use log::info;
use spirv_std::glam::UVec2;

use super::Bindable;

#[derive(Debug)]
pub struct Texture {
    format: wgpu::TextureFormat,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: UVec2,
        format: wgpu::TextureFormat,
    ) -> Self {
        let label = label.as_ref();

        info!("Allocating texture `{label}`; size={size:?}");

        assert!(size.x > 0);
        assert!(size.y > 0);

        let usage = if format == wgpu::TextureFormat::Depth32Float {
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
        } else {
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
        };

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{label}_tex")),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = tex.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{label}_sampler")),
            ..Default::default()
        });

        Self {
            format,
            view,
            sampler,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn as_ro_sampled_bind(&self) -> impl Bindable + '_ {
        ReadonlyTextureBinder { parent: self }
    }

    pub fn as_rw_storage_bind(&self) -> impl Bindable + '_ {
        WritableTextureBinder { parent: self }
    }
}

pub struct ReadonlyTextureBinder<'a> {
    parent: &'a Texture,
}

impl Bindable for ReadonlyTextureBinder<'_> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let tex_layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT
                | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float {
                    filterable: false,
                },
            },
            count: None,
        };

        let sampler_layout = wgpu::BindGroupLayoutEntry {
            binding: binding + 1,
            visibility: wgpu::ShaderStages::FRAGMENT
                | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Sampler(
                wgpu::SamplerBindingType::NonFiltering,
            ),
            count: None,
        };

        let tex_resource =
            wgpu::BindingResource::TextureView(&self.parent.view);

        let sampler_resource =
            wgpu::BindingResource::Sampler(&self.parent.sampler);

        vec![
            (tex_layout, tex_resource),
            (sampler_layout, sampler_resource),
        ]
    }
}

pub struct WritableTextureBinder<'a> {
    parent: &'a Texture,
}

impl Bindable for WritableTextureBinder<'_> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let tex_layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT
                | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::ReadWrite,
                format: self.parent.format,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        };

        let tex_resource =
            wgpu::BindingResource::TextureView(&self.parent.view);

        vec![(tex_layout, tex_resource)]
    }
}
