use bevy::asset::AssetId;
use bevy::render::render_resource::Texture;
use bevy::render::texture::Image as BevyImage;

#[derive(Debug)]
pub struct Image {
    pub(crate) data: ImageData,
    pub(crate) texture_descriptor: wgpu::TextureDescriptor<'static>,

    // TODO propagate sampler's addressing modes to the shader so that we know
    //      whether the texture should be repeated, etc.
    pub(crate) _sampler_descriptor: wgpu::SamplerDescriptor<'static>,
}

impl Image {
    pub fn new(
        data: ImageData,
        texture_descriptor: wgpu::TextureDescriptor<'static>,
        sampler_descriptor: wgpu::SamplerDescriptor<'static>,
    ) -> Self {
        assert_eq!(texture_descriptor.dimension, wgpu::TextureDimension::D2);

        Self {
            data,
            texture_descriptor,
            _sampler_descriptor: sampler_descriptor,
        }
    }
}

#[derive(Debug)]
pub enum ImageData {
    Raw { data: Vec<u8> },
    Texture { texture: Texture, is_dynamic: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageHandle(AssetId<BevyImage>);

impl ImageHandle {
    pub fn new(asset: AssetId<BevyImage>) -> Self {
        Self(asset)
    }

    pub fn get(&self) -> AssetId<BevyImage> {
        self.0
    }
}
