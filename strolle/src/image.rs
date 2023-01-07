use std::ops::Deref;

pub trait ImageTexture {
    fn get(&self) -> &wgpu::TextureView;
}

impl<T> ImageTexture for T
where
    T: Deref<Target = wgpu::TextureView>,
{
    fn get(&self) -> &wgpu::TextureView {
        self
    }
}

pub trait ImageSampler {
    fn get(&self) -> &wgpu::Sampler;
}

impl<T> ImageSampler for T
where
    T: Deref<Target = wgpu::Sampler>,
{
    fn get(&self) -> &wgpu::Sampler {
        self
    }
}
