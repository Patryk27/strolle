use spirv_std::glam::UVec2;

use super::Bindable;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub struct Texture {
    tex_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        label: impl AsRef<str>,
        size: UVec2,
    ) -> Self {
        let label = label.as_ref();

        log::debug!("Allocating texture `{label}`; size={:?}", size);

        assert!(size.x > 0);
        assert!(size.y > 0);

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
            format: FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
        });

        let tex_view = tex.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{label}_sampler")),
            ..Default::default()
        });

        Self { tex_view, sampler }
    }

    pub fn readable(&self) -> ReadableTexture {
        ReadableTexture { parent: self }
    }

    pub fn writable(&self) -> WritableTexture {
        WritableTexture { parent: self }
    }
}

pub struct ReadableTexture<'a> {
    parent: &'a Texture,
}

impl Bindable for ReadableTexture<'_> {
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
            wgpu::BindingResource::TextureView(&self.parent.tex_view);

        let sampler_resource =
            wgpu::BindingResource::Sampler(&self.parent.sampler);

        vec![
            (tex_layout, tex_resource),
            (sampler_layout, sampler_resource),
        ]
    }
}

pub struct WritableTexture<'a> {
    parent: &'a Texture,
}

impl Bindable for WritableTexture<'_> {
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
                format: FORMAT,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        };

        let tex_resource =
            wgpu::BindingResource::TextureView(&self.parent.tex_view);

        vec![(tex_layout, tex_resource)]
    }
}
