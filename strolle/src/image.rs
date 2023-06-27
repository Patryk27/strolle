use crate::Params;

#[derive(Debug)]
pub struct Image<P>
where
    P: Params,
{
    pub(crate) data: ImageData<P>,
    pub(crate) texture_descriptor: wgpu::TextureDescriptor<'static>,

    // TODO propagate sampler's addressing modes to the shader so that we know
    //      whether the texture should be repeated, etc.
    pub(crate) _sampler_descriptor: wgpu::SamplerDescriptor<'static>,
}

impl<P> Image<P>
where
    P: Params,
{
    pub fn new(
        data: ImageData<P>,
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
pub enum ImageData<P>
where
    P: Params,
{
    Raw {
        data: Vec<u8>,
    },
    Texture {
        texture: P::ImageTexture,
        is_dynamic: bool,
    },
}
